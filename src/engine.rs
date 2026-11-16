pub mod history;
pub mod restore;

mod filesystem;
mod inventory;
pub mod retention;
mod storage;

use self::{
    filesystem::{
        copy_file, destination, disk_size, ensure_disjoint, metadata_if_exists, TempPath,
    },
    inventory::{digest, hash_reader, inventory, Entry},
    retention::prune,
    storage::{lock_destination, manifest_path, read_manifest},
};
use crate::{
    model::{Job, Manifest, Record},
    persistence::save,
};
use anyhow::{Context, Result};
use chrono::{Local, Utc};
use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    sync::mpsc::Sender,
};
use uuid::Uuid;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

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
    let (source, root) = backup_folders(job)?;
    let _lock = lock_destination(&root)?;
    let manifest_path = manifest_path(&root, job.id);
    let mut manifest = read_manifest(&root, job.id)?;
    report(progress, "Hashing all source contents …");
    let (entries, skipped) = inventory(&source)?;
    let hash = digest(&entries)?;
    if latest_matches(&root, &manifest, &hash) {
        return Ok(Outcome {
            message: format!("Unchanged · {skipped} skipped"),
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

/// Progress is best effort: the receiver may already be gone.
fn report(progress: &Sender<String>, message: impl Into<String>) {
    let _ = progress.send(message.into());
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

/// `count` followed by its noun in singular or plural form, e.g. `1 file` or `3 files`.
pub(crate) fn counted(count: usize, singular: &str, plural: &str) -> String {
    format!("{count} {}", if count == 1 { singular } else { plural })
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
        counted(entry_count, "file/folder", "files/folders"),
        format!("{skipped} skipped"),
    ];
    let latest = manifest
        .records
        .last()
        .filter(|record| job.count_protected || !record.protected)
        .map_or(0, |record| record.bytes);
    if job.rolling && job.max_bytes.is_some_and(|limit| latest > limit) {
        parts.push("Latest backup exceeds size limit".into());
    }
    if let Some(error) = cleanup_error {
        parts.push(format!("Cleanup failed: {error:#}"));
    }
    parts.extend(retention::warnings(job, &manifest.records));
    parts.join(" · ")
}

#[cfg(test)]
fn discard_progress() -> Sender<String> {
    std::sync::mpsc::channel().0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::load;
    use anyhow::bail;
    use fs2::FileExt;
    use std::io::Read;

    #[test]
    fn counts_use_singular_only_for_one() {
        assert_eq!(counted(0, "place", "places"), "0 places");
        assert_eq!(counted(1, "file/folder", "files/folders"), "1 file/folder");
        assert_eq!(counted(2, "place", "places"), "2 places");
    }

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
        assert_eq!(
            load::<crate::model::Config>(&p).unwrap().version,
            crate::model::CONFIG_VERSION
        );
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
