use crate::model::{Job, Manifest, Record};
use anyhow::{bail, Context, Result};
use chrono::Utc;
use fs2::FileExt;
use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::mpsc::Sender,
};
use zip::{write::SimpleFileOptions, ZipWriter};

pub fn save<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let data = serde_json::to_vec_pretty(value)?;
    atomicwrites::AtomicFile::new(path, atomicwrites::AllowOverwrite)
        .write(|f| {
            f.write_all(&data)?;
            f.sync_all()
        })
        .map_err(|e| anyhow::anyhow!("Failed to save: {e}"))
}
pub fn load<T: DeserializeOwned>(path: &Path) -> Result<T> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
#[derive(Clone, Debug, PartialEq, Serialize)]
struct Entry {
    path: String,
    hash: String,
    size: u64,
    directory: bool,
}
fn is_link(meta: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        meta.file_type().is_symlink() || meta.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        meta.file_type().is_symlink()
    }
}
fn file_hash(path: &Path) -> Result<String> {
    let mut f = File::open(path)?;
    let mut h = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(format!("{:x}", h.finalize()))
}
fn inventory(root: &Path) -> Result<(Vec<Entry>, usize)> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<Entry>, skipped: &mut usize) -> Result<()> {
        for item in fs::read_dir(dir)? {
            let p = item?.path();
            let meta = fs::symlink_metadata(&p)?;
            if is_link(&meta) || (!meta.is_dir() && !meta.is_file()) {
                *skipped += 1;
                continue;
            }
            let relative = p.strip_prefix(root)?;
            let mut components = Vec::new();
            for component in relative.components() {
                let name = component
                    .as_os_str()
                    .to_str()
                    .context("File name is not valid UTF-8")?;
                anyhow::ensure!(
                    !name.contains('\\'),
                    "File names containing backslashes are not supported"
                );
                components.push(name);
            }
            let rel = components.join("/");
            out.push(Entry {
                path: rel,
                hash: if meta.is_file() {
                    file_hash(&p)?
                } else {
                    String::new()
                },
                size: if meta.is_file() { meta.len() } else { 0 },
                directory: meta.is_dir(),
            });
            if meta.is_dir() {
                walk(root, &p, out, skipped)?;
            }
        }
        Ok(())
    }
    let mut entries = vec![];
    let mut skipped = 0;
    walk(root, root, &mut entries, &mut skipped)?;
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok((entries, skipped))
}
fn digest(entries: &[Entry]) -> Result<String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(entries)?)
    ))
}
fn bytes(path: &Path) -> Result<u64> {
    let m = fs::symlink_metadata(path)?;
    if is_link(&m) {
        bail!("Link found in backup destination")
    }
    if m.is_file() {
        return Ok(m.len());
    }
    let mut n = 0;
    for e in fs::read_dir(path)? {
        n += bytes(&e?.path())?;
    }
    Ok(n)
}
struct Temp(PathBuf);
impl Drop for Temp {
    fn drop(&mut self) {
        if self.0.is_dir() {
            let _ = fs::remove_dir_all(&self.0);
        } else {
            let _ = fs::remove_file(&self.0);
        }
    }
}
pub struct Outcome {
    pub message: String,
    pub created: bool,
}
pub fn run(job: &Job, progress: &Sender<String>) -> Result<Outcome> {
    run_inner(job, progress, || Ok(()))
}
fn run_inner(
    job: &Job,
    progress: &Sender<String>,
    before_verify: impl FnOnce() -> Result<()>,
) -> Result<Outcome> {
    anyhow::ensure!(
        !job.source.as_os_str().is_empty() && !job.target.as_os_str().is_empty(),
        "Please select source and destination folders"
    );
    let source = fs::canonicalize(&job.source).context("Source folder is unavailable")?;
    if !source.is_dir() {
        bail!("Source is not a folder")
    }
    let target = fs::canonicalize(&job.target).context("Destination folder is unavailable")?;
    if source.starts_with(&target) || target.starts_with(&source) {
        bail!("Source and destination folders must not overlap")
    }
    anyhow::ensure!(target.is_dir(), "Destination is not a folder");
    let root = target;
    let lock_path = root.join(".lock");
    let manifest_path = root.join(format!("manifest-{}.json", job.id));
    for path in [&lock_path, &manifest_path] {
        match fs::symlink_metadata(path) {
            Ok(metadata) => {
                anyhow::ensure!(!is_link(&metadata), "Backup metadata must not be a link")
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)?;
    lock.try_lock_exclusive()
        .context("This destination is already in use")?;
    let mut manifest: Manifest = if manifest_path.exists() {
        load(&manifest_path)?
    } else {
        Manifest {
            version: 1,
            job_id: Some(job.id),
            records: vec![],
        }
    };
    if manifest.version != 1 || manifest.job_id != Some(job.id) {
        bail!("Unknown backup manifest")
    }
    for r in &manifest.records {
        if Path::new(&r.name).components().count() != 1
            || !r.name.starts_with(|c: char| c.is_ascii_digit())
        {
            bail!("Invalid backup name in manifest")
        }
    }
    let _ = progress.send("Hashing all source contents …".into());
    let (entries, skipped) = inventory(&source)?;
    let hash = digest(&entries)?;
    let unchanged = manifest
        .records
        .last()
        .map(|r| r.hash == hash && root.join(&r.name).exists())
        .unwrap_or(false);
    if unchanged {
        return Ok(Outcome {
            message: format!("Unchanged · {skipped} links/special files skipped"),
            created: false,
        });
    }
    let folder = source
        .file_name()
        .context("Source folder has no name")?
        .to_string_lossy();
    let base = format!(
        "{}_{}",
        chrono::Local::now().format("%Y-%m-%d_%H-%M-%S"),
        folder
    );
    let ext = if job.zip { ".zip" } else { "" };
    let mut name = format!("{base}{ext}");
    let mut i = 1;
    while root.join(&name).exists() {
        name = format!("{base}_{i}{ext}");
        i += 1;
    }
    let temp = Temp(root.join(format!(".partial-{}", uuid::Uuid::new_v4())));
    if job.zip {
        let mut zip = ZipWriter::new(File::create(&temp.0)?);
        for (index, e) in entries.iter().enumerate() {
            let _ = progress.send(format!(
                "ZIP: {} / {} · {}",
                index + 1,
                entries.len(),
                e.path
            ));
            if e.directory {
                zip.add_directory(format!("{}/", e.path), SimpleFileOptions::default())?;
            } else {
                zip.start_file(
                    &e.path,
                    SimpleFileOptions::default()
                        .compression_method(zip::CompressionMethod::Deflated),
                )?;
                std::io::copy(&mut File::open(source.join(&e.path))?, &mut zip)?;
            }
        }
        zip.finish()?.sync_all()?;
        let _ = progress.send("Verifying ZIP contents …".into());
        let mut archive = zip::ZipArchive::new(File::open(&temp.0)?)?;
        for e in &entries {
            if !e.directory {
                let mut f = archive.by_name(&e.path)?;
                let mut h = Sha256::new();
                let mut buf = [0; 65536];
                loop {
                    let n = f.read(&mut buf)?;
                    if n == 0 {
                        break;
                    }
                    h.update(&buf[..n]);
                }
                if format!("{:x}", h.finalize()) != e.hash {
                    bail!("File changed during backup: {}", e.path)
                }
            }
        }
    } else {
        fs::create_dir(&temp.0)?;
        for (index, e) in entries.iter().enumerate() {
            let _ = progress.send(format!(
                "Copying: {} / {} · {}",
                index + 1,
                entries.len(),
                e.path
            ));
            let dst = temp.0.join(&e.path);
            if e.directory {
                fs::create_dir_all(dst)?;
            } else {
                if let Some(p) = dst.parent() {
                    fs::create_dir_all(p)?;
                }
                let input_path = source.join(&e.path);
                let mut output = File::create(&dst)?;
                std::io::copy(&mut File::open(&input_path)?, &mut output)?;
                output.sync_all()?;
                #[cfg(unix)]
                fs::set_permissions(&dst, fs::metadata(&input_path)?.permissions())?;
            }
        }
        if inventory(&temp.0)?.0 != entries {
            bail!("Copied contents do not match the source")
        }
    }
    before_verify()?;
    let _ = progress.send("Checking source again …".into());
    if inventory(&source)?.0 != entries {
        bail!("Source changed during backup; please check again")
    }
    let size = bytes(&temp.0)?;
    fs::rename(&temp.0, root.join(&name))?;
    manifest.records.push(Record {
        name,
        hash,
        bytes: size,
        created: Utc::now(),
    });
    save(&manifest_path, &manifest)?;
    let warning = prune(&root, &manifest_path, &mut manifest, job)
        .err()
        .map(|e| format!(" · Cleanup failed: {e}"))
        .unwrap_or_default();
    let over = job.rolling && job.max_bytes.map(|n| size > n).unwrap_or(false);
    Ok(Outcome {
        message: format!(
            "Backup complete · {} files/folders · {skipped} skipped{}{}",
            entries.len(),
            if over {
                " · Latest backup exceeds size limit"
            } else {
                ""
            },
            warning
        ),
        created: true,
    })
}
fn prune(root: &Path, path: &Path, m: &mut Manifest, j: &Job) -> Result<()> {
    if !j.rolling {
        return Ok(());
    }
    while m.records.len() > 1 {
        let over_count = j
            .max_count
            .map(|n| m.records.len() > n.max(1))
            .unwrap_or(false);
        let over_size = j
            .max_bytes
            .map(|n| m.records.iter().map(|r| r.bytes).sum::<u64>() > n)
            .unwrap_or(false);
        if !over_count && !over_size {
            break;
        }
        let p = root.join(&m.records[0].name);
        if p.exists() {
            let meta = fs::symlink_metadata(&p)?;
            if is_link(&meta) {
                bail!("Cleanup refused to remove a link")
            };
            if meta.is_dir() {
                fs::remove_dir_all(&p)?;
            } else {
                fs::remove_file(&p)?;
            }
        }
        m.records.remove(0);
        save(path, m)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (tempfile::TempDir, Job) {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("source");
        let d = t.path().join("target");
        fs::create_dir(&s).unwrap();
        fs::create_dir(&d).unwrap();
        fs::write(s.join("a"), "hello").unwrap();
        let j = Job {
            source: s,
            target: d,
            ..Job::default()
        };
        (t, j)
    }
    fn run_test(j: &Job) -> Outcome {
        run(j, &std::sync::mpsc::channel().0).unwrap()
    }
    #[test]
    fn backups_use_selected_destination_directly() {
        let (_t, j) = setup();
        run_test(&j);
        let manifest: Manifest = load(&j.target.join(format!("manifest-{}.json", j.id))).unwrap();
        assert!(j.target.join(&manifest.records[0].name).join("a").is_file());
        assert!(!j.target.join(format!("backup-{}", j.id)).exists());
    }

    #[test]
    fn shared_destination_keeps_job_history_separate() {
        let (_t, first) = setup();
        let mut second = first.clone();
        second.id = uuid::Uuid::new_v4();
        second.rolling = true;
        second.max_count = Some(1);
        run_test(&first);
        run_test(&second);
        fs::write(second.source.join("a"), "changed").unwrap();
        run_test(&second);
        let first_manifest: Manifest =
            load(&first.target.join(format!("manifest-{}.json", first.id))).unwrap();
        let second_manifest: Manifest =
            load(&second.target.join(format!("manifest-{}.json", second.id))).unwrap();
        assert_eq!(first_manifest.records.len(), 1);
        assert_eq!(second_manifest.records.len(), 1);
        assert!(first.target.join(&first_manifest.records[0].name).exists());
        assert!(second
            .target
            .join(&second_manifest.records[0].name)
            .exists());
        assert_ne!(
            first_manifest.records[0].name,
            second_manifest.records[0].name
        );
    }

    #[test]
    fn destination_lock_prevents_concurrent_checks() {
        let (_t, job) = setup();
        let lock = File::create(job.target.join(".lock")).unwrap();
        lock.try_lock_exclusive().unwrap();
        assert!(run(&job, &std::sync::mpsc::channel().0).is_err());
        assert!(!job
            .target
            .join(format!("manifest-{}.json", job.id))
            .exists());
    }

    #[cfg(unix)]
    #[test]
    fn metadata_links_are_rejected() {
        for name in [".lock", "manifest"] {
            let (_t, job) = setup();
            let metadata = if name == "manifest" {
                format!("manifest-{}.json", job.id)
            } else {
                name.into()
            };
            std::os::unix::fs::symlink(job.source.join("a"), job.target.join(metadata)).unwrap();
            assert!(run(&job, &std::sync::mpsc::channel().0).is_err());
            assert_eq!(fs::read_to_string(job.source.join("a")).unwrap(), "hello");
        }
    }

    #[test]
    fn changes_and_empty_directories() {
        let (_t, j) = setup();
        assert!(run_test(&j).created);
        assert!(!run_test(&j).created);
        fs::rename(j.source.join("a"), j.source.join("b")).unwrap();
        assert!(run_test(&j).created);
        fs::create_dir(j.source.join("empty")).unwrap();
        assert!(run_test(&j).created);
        fs::remove_file(j.source.join("b")).unwrap();
        assert!(run_test(&j).created);
    }
    #[test]
    fn content_not_metadata() {
        let (_t, j) = setup();
        run_test(&j);
        fs::write(j.source.join("a"), "other").unwrap();
        assert!(run_test(&j).created);
        fs::write(j.source.join("a"), "other").unwrap();
        assert!(!run_test(&j).created);
    }
    #[test]
    fn zip_is_readable() {
        let (_t, mut j) = setup();
        j.zip = true;
        run_test(&j);
        let root = j.target.clone();
        let m: Manifest = load(&root.join(format!("manifest-{}.json", j.id))).unwrap();
        let mut z =
            zip::ZipArchive::new(File::open(root.join(&m.records[0].name)).unwrap()).unwrap();
        let mut s = String::new();
        z.by_name("a").unwrap().read_to_string(&mut s).unwrap();
        assert_eq!(s, "hello");
    }
    #[test]
    fn rolling_preserves_latest_and_foreign() {
        let (_t, mut j) = setup();
        j.rolling = true;
        j.max_count = Some(1);
        j.max_bytes = Some(1);
        run_test(&j);
        let root = j.target.clone();
        fs::write(root.join("foreign"), "safe").unwrap();
        fs::write(j.source.join("a"), "changed").unwrap();
        let o = run_test(&j);
        assert!(o.message.contains("exceeds"));
        let m: Manifest = load(&root.join(format!("manifest-{}.json", j.id))).unwrap();
        assert_eq!(m.records.len(), 1);
        assert!(root.join(&m.records[0].name).exists());
        assert!(root.join("foreign").exists());
    }
    #[test]
    fn overlap_rejected() {
        let (_t, mut j) = setup();
        j.target = j.source.clone();
        assert!(run(&j, &std::sync::mpsc::channel().0).is_err());
    }
    #[test]
    fn missing_source_fails() {
        let (_t, mut j) = setup();
        j.source.push("missing");
        assert!(run(&j, &std::sync::mpsc::channel().0).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn symlink_skipped() {
        let (_t, j) = setup();
        std::os::unix::fs::symlink(&j.source, j.source.join("loop")).unwrap();
        assert!(run_test(&j).message.contains("1 skipped"));
    }
    #[test]
    fn config_roundtrip() {
        let t = tempfile::tempdir().unwrap();
        let p = t.path().join("config");
        save(&p, &crate::model::Config::default()).unwrap();
        save(&p, &crate::model::Config::default()).unwrap();
        assert_eq!(load::<crate::model::Config>(&p).unwrap().version, 1);
    }
    #[test]
    fn changing_source_discards_backup_without_pruning() {
        for zip in [false, true] {
            let (_t, mut j) = setup();
            j.zip = zip;
            j.rolling = true;
            j.max_count = Some(1);
            run_test(&j);
            fs::write(j.source.join("a"), "version2").unwrap();
            let root = j.target.clone();
            let path = root.join(format!("manifest-{}.json", j.id));
            let original = fs::read(&path).unwrap();
            let result = run_inner(&j, &std::sync::mpsc::channel().0, || {
                fs::write(j.source.join("a"), "version3")?;
                Ok(())
            });
            assert!(result.is_err());
            assert_eq!(original, fs::read(&path).unwrap());
            assert!(!fs::read_dir(&root).unwrap().any(|e| e
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".partial-")));
            let m: Manifest = load(&path).unwrap();
            assert!(root.join(&m.records[0].name).exists());
        }
    }
    #[test]
    fn interrupted_backup_is_cleaned_up() {
        let (_t, j) = setup();
        assert!(run_inner(&j, &std::sync::mpsc::channel().0, || bail!(
            "Simulated write failure"
        ))
        .is_err());
        let root = j.target.clone();
        assert!(!root.join(format!("manifest-{}.json", j.id)).exists());
        assert_eq!(fs::read_dir(root).unwrap().count(), 1);
    }
    #[test]
    fn rolling_disabled_keeps_all() {
        let (_t, mut j) = setup();
        j.max_count = Some(1);
        j.max_bytes = Some(1);
        run_test(&j);
        fs::write(j.source.join("a"), "second").unwrap();
        run_test(&j);
        let m: Manifest = load(&j.target.join(format!("manifest-{}.json", j.id))).unwrap();
        assert_eq!(m.records.len(), 2);
    }
    #[test]
    fn missing_latest_is_recreated() {
        let (_t, j) = setup();
        run_test(&j);
        let root = j.target.clone();
        let m: Manifest = load(&root.join(format!("manifest-{}.json", j.id))).unwrap();
        fs::remove_dir_all(root.join(&m.records[0].name)).unwrap();
        assert!(run_test(&j).created);
    }
    #[test]
    fn invalid_target_fails() {
        let (_t, mut j) = setup();
        j.target = j.source.join("a");
        assert!(run(&j, &std::sync::mpsc::channel().0).is_err());
    }
    #[test]
    fn nested_copy_and_zip_empty_directory() {
        for zip in [false, true] {
            let (_t, mut j) = setup();
            j.zip = zip;
            fs::create_dir_all(j.source.join("nested/empty")).unwrap();
            fs::write(j.source.join("nested/file"), "content").unwrap();
            run_test(&j);
            let root = j.target.clone();
            let m: Manifest = load(&root.join(format!("manifest-{}.json", j.id))).unwrap();
            if zip {
                let mut z =
                    zip::ZipArchive::new(File::open(root.join(&m.records[0].name)).unwrap())
                        .unwrap();
                assert!(z.by_name("nested/empty/").unwrap().is_dir());
            } else {
                assert!(root.join(&m.records[0].name).join("nested/empty").is_dir());
            }
        }
    }
}
