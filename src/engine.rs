pub mod history;
pub mod restore;

use crate::model::{Job, Manifest, Record};
use anyhow::{bail, Context, Result};
use chrono::{Local, Utc};
use fs2::FileExt;
use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, Metadata, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::mpsc::Sender,
};
use uuid::Uuid;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

const MANIFEST_VERSION: u32 = 1;

pub struct Outcome {
    pub message: String,
    pub created: bool,
}

/// One file or folder of a backup; its serialized form defines the backup fingerprint.
#[derive(Clone, Debug, PartialEq, Serialize)]
struct Entry {
    path: String,
    hash: String,
    size: u64,
    directory: bool,
}

/// Removes a partially written file or folder when dropped.
struct TempPath(PathBuf);

impl Drop for TempPath {
    fn drop(&mut self) {
        let _ = if self.0.is_dir() {
            fs::remove_dir_all(&self.0)
        } else {
            fs::remove_file(&self.0)
        };
    }
}

pub fn save<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let data = serde_json::to_vec_pretty(value)?;
    atomicwrites::AtomicFile::new(path, atomicwrites::AllowOverwrite)
        .write(|file| {
            file.write_all(&data)?;
            file.sync_all()
        })
        .map_err(|error| anyhow::anyhow!("Failed to save: {error}"))
}

pub fn load<T: DeserializeOwned>(path: &Path) -> Result<T> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

pub fn manifest_path(root: &Path, id: Uuid) -> PathBuf {
    root.join(format!("manifest-{id}.json"))
}

pub fn run(job: &Job, progress: &Sender<String>) -> Result<Outcome> {
    run_inner(job, progress, || Ok(()))
}

fn run_inner(
    job: &Job,
    progress: &Sender<String>,
    before_verify: impl FnOnce() -> Result<()>,
) -> Result<Outcome> {
    let (source, root) = backup_folders(job)?;
    let _lock = lock_destination(&root)?;
    let manifest_path = manifest_path(&root, job.id);
    let mut manifest = read_manifest(&root, job.id)?;
    report(progress, "Hashing all source contents …");
    let (entries, skipped) = inventory(&source)?;
    let hash = digest(&entries)?;
    if latest_matches(&root, &manifest, &hash) {
        return Ok(Outcome {
            message: format!("Unchanged · {skipped} links/special files skipped"),
            created: false,
        });
    }
    let name = unique_backup_name(&root, &source, job.zip)?;
    let temp = TempPath(root.join(format!(".partial-{}", Uuid::new_v4())));
    if job.zip {
        write_zip(&source, &entries, &temp.0, progress)?;
    } else {
        write_folder(&source, &entries, &temp.0, progress)?;
    }
    before_verify()?;
    report(progress, "Checking source again …");
    anyhow::ensure!(
        inventory(&source)?.0 == entries,
        "Source changed during backup; please check again"
    );
    let size = disk_size(&temp.0)?;
    fs::rename(&temp.0, root.join(&name))?;
    manifest.records.push(Record {
        name,
        hash,
        bytes: size,
        created: Utc::now(),
        protected: false,
    });
    save(&manifest_path, &manifest)?;
    let cleanup_error = prune(&root, &manifest_path, &mut manifest, job).err();
    Ok(Outcome {
        message: completion_message(job, &manifest, entries.len(), skipped, cleanup_error),
        created: true,
    })
}

fn backup_folders(job: &Job) -> Result<(PathBuf, PathBuf)> {
    anyhow::ensure!(
        !job.source.as_os_str().is_empty() && !job.target.as_os_str().is_empty(),
        "Please select source and destination folders"
    );
    let source = fs::canonicalize(&job.source).context("Source folder is unavailable")?;
    anyhow::ensure!(source.is_dir(), "Source is not a folder");
    let target = destination(job)?;
    ensure_disjoint(&source, &target)?;
    anyhow::ensure!(target.is_dir(), "Destination is not a folder");
    Ok((source, target))
}

fn destination(job: &Job) -> Result<PathBuf> {
    fs::canonicalize(&job.target).context("Destination folder is unavailable")
}

fn ensure_disjoint(source: &Path, target: &Path) -> Result<()> {
    anyhow::ensure!(
        !source.starts_with(target) && !target.starts_with(source),
        "Source and destination folders must not overlap"
    );
    Ok(())
}

/// Progress is best effort: the receiver may already be gone.
fn report(progress: &Sender<String>, message: impl Into<String>) {
    let _ = progress.send(message.into());
}

fn is_zip_backup(name: &str) -> bool {
    name.ends_with(".zip")
}

fn latest_matches(root: &Path, manifest: &Manifest, hash: &str) -> bool {
    manifest
        .records
        .last()
        .is_some_and(|record| record.hash == hash && root.join(&record.name).exists())
}

fn unique_backup_name(root: &Path, source: &Path, zip: bool) -> Result<String> {
    let folder = source
        .file_name()
        .context("Source folder has no name")?
        .to_string_lossy();
    let base = format!("{}_{folder}", Local::now().format("%Y-%m-%d_%H-%M-%S"));
    let extension = if zip { ".zip" } else { "" };
    let mut name = format!("{base}{extension}");
    let mut suffix = 1;
    while metadata_if_exists(&root.join(&name))?.is_some() {
        name = format!("{base}_{suffix}{extension}");
        suffix += 1;
    }
    Ok(name)
}

fn write_zip(
    source: &Path,
    entries: &[Entry],
    output: &Path,
    progress: &Sender<String>,
) -> Result<()> {
    let mut zip = ZipWriter::new(File::create(output)?);
    let deflated = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (index, entry) in entries.iter().enumerate() {
        report(
            progress,
            format!("ZIP: {} / {} · {}", index + 1, entries.len(), entry.path),
        );
        if entry.directory {
            zip.add_directory(format!("{}/", entry.path), SimpleFileOptions::default())?;
        } else {
            zip.start_file(&entry.path, deflated)?;
            io::copy(&mut File::open(source.join(&entry.path))?, &mut zip)?;
        }
    }
    zip.finish()?.sync_all()?;
    report(progress, "Verifying ZIP contents …");
    let mut archive = ZipArchive::new(File::open(output)?)?;
    for entry in entries.iter().filter(|entry| !entry.directory) {
        anyhow::ensure!(
            hash_reader(archive.by_name(&entry.path)?)? == entry.hash,
            "File changed during backup: {}",
            entry.path
        );
    }
    Ok(())
}

fn write_folder(
    source: &Path,
    entries: &[Entry],
    output: &Path,
    progress: &Sender<String>,
) -> Result<()> {
    fs::create_dir(output)?;
    for (index, entry) in entries.iter().enumerate() {
        report(
            progress,
            format!(
                "Copying: {} / {} · {}",
                index + 1,
                entries.len(),
                entry.path
            ),
        );
        let destination = output.join(&entry.path);
        if entry.directory {
            fs::create_dir_all(destination)?;
            continue;
        }
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        copy_file(&source.join(&entry.path), &destination)?;
    }
    anyhow::ensure!(
        inventory(output)?.0 == entries,
        "Copied contents do not match the source"
    );
    Ok(())
}

/// Copies into a fresh file only; an existing output means the workspace was tampered with.
fn copy_file(from: &Path, to: &Path) -> Result<()> {
    let mut output = OpenOptions::new().write(true).create_new(true).open(to)?;
    io::copy(&mut File::open(from)?, &mut output)?;
    output.sync_all()?;
    #[cfg(unix)]
    fs::set_permissions(to, fs::metadata(from)?.permissions())?;
    Ok(())
}

fn completion_message(
    job: &Job,
    manifest: &Manifest,
    entry_count: usize,
    skipped: usize,
    cleanup_error: Option<anyhow::Error>,
) -> String {
    let mut parts = vec![
        "Backup complete".to_owned(),
        format!("{entry_count} files/folders"),
        format!("{skipped} skipped"),
    ];
    let latest = manifest.records.last().map_or(0, |record| record.bytes);
    if job.rolling && job.max_bytes.is_some_and(|limit| latest > limit) {
        parts.push("Latest backup exceeds size limit".into());
    }
    if let Some(error) = cleanup_error {
        parts.push(format!("Cleanup failed: {error:#}"));
    }
    parts.extend(history::warnings(job, &manifest.records));
    parts.join(" · ")
}

/// Deletes the oldest unprotected backups until the limits hold; the newest is always kept.
fn prune(root: &Path, manifest_path: &Path, manifest: &mut Manifest, job: &Job) -> Result<()> {
    if !job.rolling {
        return Ok(());
    }
    while manifest.records.len() > 1 && exceeds_limits(job, &manifest.records) {
        let newest = manifest.records.len() - 1;
        let Some(index) = manifest.records[..newest]
            .iter()
            .position(|record| !record.protected)
        else {
            break;
        };
        remove_backup(&root.join(&manifest.records[index].name))?;
        manifest.records.remove(index);
        save(manifest_path, manifest)?;
    }
    Ok(())
}

fn exceeds_limits(job: &Job, records: &[Record]) -> bool {
    let total: u128 = records.iter().map(|record| u128::from(record.bytes)).sum();
    job.max_count
        .is_some_and(|limit| records.len() > limit.max(1))
        || job.max_bytes.is_some_and(|limit| total > u128::from(limit))
}

fn remove_backup(path: &Path) -> Result<()> {
    let Some(metadata) = metadata_if_exists(path)? else {
        return Ok(());
    };
    anyhow::ensure!(!is_link(&metadata), "Cleanup refused to remove a link");
    if metadata.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn metadata_if_exists(path: &Path) -> Result<Option<Metadata>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn is_link(metadata: &Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        metadata.file_type().is_symlink()
            || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

fn hash_reader(mut reader: impl Read) -> Result<String> {
    let mut hasher = Sha256::new();
    io::copy(&mut reader, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

fn inventory(root: &Path) -> Result<(Vec<Entry>, usize)> {
    let mut entries = Vec::new();
    let mut skipped = 0;
    collect_entries(root, root, &mut entries, &mut skipped)?;
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok((entries, skipped))
}

/// Links and special files are counted but never followed or saved.
fn collect_entries(
    root: &Path,
    directory: &Path,
    entries: &mut Vec<Entry>,
    skipped: &mut usize,
) -> Result<()> {
    for item in fs::read_dir(directory)? {
        let path = item?.path();
        let metadata = fs::symlink_metadata(&path)?;
        if is_link(&metadata) || !(metadata.is_dir() || metadata.is_file()) {
            *skipped += 1;
            continue;
        }
        let is_directory = metadata.is_dir();
        entries.push(Entry {
            path: portable_path(path.strip_prefix(root)?)?,
            hash: if is_directory {
                String::new()
            } else {
                hash_reader(File::open(&path)?)?
            },
            size: if is_directory { 0 } else { metadata.len() },
            directory: is_directory,
        });
        if is_directory {
            collect_entries(root, &path, entries, skipped)?;
        }
    }
    Ok(())
}

/// Joins with `/` so fingerprints and ZIP names match across platforms.
fn portable_path(relative: &Path) -> Result<String> {
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
    Ok(components.join("/"))
}

fn digest(entries: &[Entry]) -> Result<String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(entries)?)
    ))
}

fn disk_size(path: &Path) -> Result<u64> {
    let metadata = fs::symlink_metadata(path)?;
    anyhow::ensure!(!is_link(&metadata), "Link found in backup destination");
    if metadata.is_file() {
        return Ok(metadata.len());
    }
    let mut total = 0;
    for entry in fs::read_dir(path)? {
        total += disk_size(&entry?.path())?;
    }
    Ok(total)
}

fn reject_metadata_link(path: &Path) -> Result<()> {
    if let Some(metadata) = metadata_if_exists(path)? {
        anyhow::ensure!(!is_link(&metadata), "Backup metadata must not be a link");
    }
    Ok(())
}

fn lock_destination(root: &Path) -> Result<File> {
    let lock_path = root.join(".lock");
    reject_metadata_link(&lock_path)?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)?;
    lock.try_lock_exclusive()
        .context("This destination is already in use")?;
    Ok(lock)
}

fn read_manifest(root: &Path, id: Uuid) -> Result<Manifest> {
    let path = manifest_path(root, id);
    reject_metadata_link(&path)?;
    let manifest = if path.exists() {
        load(&path)?
    } else {
        Manifest {
            version: MANIFEST_VERSION,
            job_id: Some(id),
            records: vec![],
        }
    };
    if manifest.version != MANIFEST_VERSION || manifest.job_id != Some(id) {
        bail!("Unknown backup manifest")
    }
    // Names are joined onto the destination, so they must stay single path components.
    let valid_name = |name: &str| {
        Path::new(name).components().count() == 1 && name.starts_with(|c: char| c.is_ascii_digit())
    };
    anyhow::ensure!(
        manifest
            .records
            .iter()
            .all(|record| valid_name(&record.name)),
        "Invalid backup name in manifest"
    );
    Ok(manifest)
}

#[cfg(test)]
fn discard_progress() -> Sender<String> {
    std::sync::mpsc::channel().0
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
        run(j, &discard_progress()).unwrap()
    }

    #[test]
    fn backups_use_selected_destination_directly() {
        let (_t, j) = setup();
        run_test(&j);
        let manifest: Manifest = load(&manifest_path(&j.target, j.id)).unwrap();
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
        let first_manifest: Manifest = load(&manifest_path(&first.target, first.id)).unwrap();
        let second_manifest: Manifest = load(&manifest_path(&second.target, second.id)).unwrap();
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
        assert!(run(&job, &discard_progress()).is_err());
        assert!(!manifest_path(&job.target, job.id).exists());
    }

    #[cfg(unix)]
    #[test]
    fn metadata_links_are_rejected() {
        for manifest in [false, true] {
            let (_t, job) = setup();
            let metadata = if manifest {
                manifest_path(&job.target, job.id)
            } else {
                job.target.join(".lock")
            };
            std::os::unix::fs::symlink(job.source.join("a"), metadata).unwrap();
            assert!(run(&job, &discard_progress()).is_err());
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
        let m: Manifest = load(&manifest_path(&root, j.id)).unwrap();
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
        let m: Manifest = load(&manifest_path(&root, j.id)).unwrap();
        assert_eq!(m.records.len(), 1);
        assert!(root.join(&m.records[0].name).exists());
        assert!(root.join("foreign").exists());
    }

    #[test]
    fn overlap_rejected() {
        let (_t, mut j) = setup();
        j.target = j.source.clone();
        assert!(run(&j, &discard_progress()).is_err());
    }

    #[test]
    fn missing_source_fails() {
        let (_t, mut j) = setup();
        j.source.push("missing");
        assert!(run(&j, &discard_progress()).is_err());
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
            let path = manifest_path(&root, j.id);
            let original = fs::read(&path).unwrap();
            let result = run_inner(&j, &discard_progress(), || {
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
        assert!(run_inner(&j, &discard_progress(), || bail!("Simulated write failure")).is_err());
        let root = j.target.clone();
        assert!(!manifest_path(&root, j.id).exists());
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
        let m: Manifest = load(&manifest_path(&j.target, j.id)).unwrap();
        assert_eq!(m.records.len(), 2);
    }

    #[test]
    fn missing_latest_is_recreated() {
        let (_t, j) = setup();
        run_test(&j);
        let root = j.target.clone();
        let m: Manifest = load(&manifest_path(&root, j.id)).unwrap();
        fs::remove_dir_all(root.join(&m.records[0].name)).unwrap();
        assert!(run_test(&j).created);
    }

    #[test]
    fn invalid_target_fails() {
        let (_t, mut j) = setup();
        j.target = j.source.join("a");
        assert!(run(&j, &discard_progress()).is_err());
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
            let m: Manifest = load(&manifest_path(&root, j.id)).unwrap();
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
