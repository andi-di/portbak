use super::*;
use std::{collections::HashSet, ffi::OsString};

#[derive(Clone)]
pub struct Backup {
    pub record: Record,
    pub available: bool,
    pub zip: bool,
}

pub fn list(job: &Job) -> Result<Vec<Backup>> {
    let target = fs::canonicalize(&job.target).context("Destination folder is unavailable")?;
    let _lock = lock_destination(&target)?;
    let manifest = read_manifest(&target, job.id)?;
    let mut backups = Vec::new();
    for record in manifest.records.into_iter().rev() {
        let path = target.join(&record.name);
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => Some(metadata),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        };
        let zip = record.name.ends_with(".zip");
        let available =
            metadata.is_some_and(|m| !is_link(&m) && if zip { m.is_file() } else { m.is_dir() });
        backups.push(Backup {
            record,
            available,
            zip,
        });
    }
    Ok(backups)
}

pub fn run(job: &Job, name: &str, progress: &Sender<String>) -> Result<Outcome> {
    restore_inner(job, name, progress, || Ok(()))
}

fn restore_inner(
    job: &Job,
    name: &str,
    progress: &Sender<String>,
    after_install: impl FnOnce() -> Result<()>,
) -> Result<Outcome> {
    anyhow::ensure!(
        !job.source.as_os_str().is_empty(),
        "Please select a source folder"
    );
    let metadata = fs::symlink_metadata(&job.source).context("Source folder is unavailable")?;
    anyhow::ensure!(
        metadata.is_dir() && !is_link(&metadata),
        "Restore source must be an ordinary folder, not a link"
    );
    let source = fs::canonicalize(&job.source)?;
    let parent = source
        .parent()
        .context("Cannot restore into a filesystem root")?;
    let target = fs::canonicalize(&job.target).context("Destination folder is unavailable")?;
    anyhow::ensure!(
        !source.starts_with(&target) && !target.starts_with(&source),
        "Source and destination folders must not overlap"
    );
    let _lock = lock_destination(&target)?;
    let manifest = read_manifest(&target, job.id)?;
    let record = manifest
        .records
        .iter()
        .find(|record| record.name == name)
        .context("Selected backup is no longer in this job's history")?;
    let backup = target.join(name);
    let metadata = fs::symlink_metadata(&backup).context("Selected backup is unavailable")?;
    anyhow::ensure!(!is_link(&metadata), "Selected backup must not be a link");
    let stage = Temp(create_workspace(parent, ".portbak-restore-")?);
    let _ = progress.send("Preparing and verifying selected backup …".into());
    if name.ends_with(".zip") {
        anyhow::ensure!(metadata.is_file(), "Selected ZIP backup is not a file");
        extract_zip(&backup, &stage.0, progress)?;
    } else {
        anyhow::ensure!(
            metadata.is_dir(),
            "Selected folder backup is not a directory"
        );
        copy_directory(&backup, &stage.0, progress)?;
    }
    verify(&stage.0, &record.hash)?;
    let previous = create_workspace(parent, ".portbak-previous-")?;
    let mut moved = Vec::new();
    let mut installed = Vec::new();
    let result = (|| {
        let _ = progress.send("Moving current source contents aside …".into());
        move_contents(&source, &previous, &mut moved)?;
        let _ = progress.send("Installing selected backup …".into());
        move_contents(&stage.0, &source, &mut installed)?;
        after_install()?;
        verify(&source, &record.hash)
    })();
    if let Err(error) = result {
        let recovery = rollback(&source, &stage.0, &previous, &moved, &installed);
        return match recovery {
            Ok(()) => {
                fs::remove_dir(&previous).context("Original contents restored, but recovery folder could not be removed")?;
                Err(error.context("Restore failed; original source contents restored"))
            }
            Err(recovery) => Err(anyhow::anyhow!(
                "Restore failed: {error:#}. Rollback incomplete: {recovery:#}. Original data remains at {} and {}. Do not delete the recovery folder.",
                source.display(), previous.display()
            )),
        };
    }
    let _ = progress.send("Removing replaced source contents …".into());
    let message = match remove_contents(&previous).and_then(|()| Ok(fs::remove_dir(&previous)?)) {
        Ok(()) => "Restore complete · automatic backups disabled".to_owned(),
        Err(error) => format!(
            "Restore complete · old contents could not be fully removed at {}: {error:#}",
            previous.display()
        ),
    };
    Ok(Outcome {
        message,
        created: false,
    })
}

fn create_workspace(parent: &Path, prefix: &str) -> Result<PathBuf> {
    let path = parent.join(format!("{prefix}{}", uuid::Uuid::new_v4()));
    fs::create_dir(&path).context("Cannot create restore workspace beside source folder")?;
    Ok(path)
}

fn verify(root: &Path, expected: &str) -> Result<()> {
    let (entries, skipped) = inventory(root)?;
    anyhow::ensure!(
        skipped == 0,
        "Restore contents contain links or special files"
    );
    anyhow::ensure!(
        digest(&entries)? == expected,
        "Selected backup contents do not match the saved fingerprint"
    );
    Ok(())
}

fn copy_directory(source: &Path, target: &Path, progress: &Sender<String>) -> Result<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let metadata = fs::symlink_metadata(entry.path())?;
        anyhow::ensure!(!is_link(&metadata), "Folder backup contains a link");
        let output = target.join(entry.file_name());
        if metadata.is_dir() {
            fs::create_dir(&output)?;
            copy_directory(&entry.path(), &output, progress)?;
        } else {
            anyhow::ensure!(metadata.is_file(), "Folder backup contains a special file");
            let _ = progress.send(format!("Preparing restore: {}", entry.path().display()));
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&output)?;
            std::io::copy(&mut File::open(entry.path())?, &mut file)?;
            file.sync_all()?;
            #[cfg(unix)]
            fs::set_permissions(&output, metadata.permissions())?;
        }
    }
    Ok(())
}

fn extract_zip(source: &Path, target: &Path, progress: &Sender<String>) -> Result<()> {
    let mut archive = zip::ZipArchive::new(File::open(source)?)?;
    let mut names = HashSet::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = std::str::from_utf8(entry.name_raw())
            .context("ZIP name is not valid UTF-8")?
            .to_owned();
        anyhow::ensure!(
            !name.contains(['\\', ':']) && !name.starts_with('/'),
            "Unsafe ZIP path"
        );
        let trimmed = name.trim_end_matches('/');
        anyhow::ensure!(
            !trimmed.is_empty()
                && trimmed
                    .split('/')
                    .all(|part| !part.is_empty() && part != "." && part != ".."),
            "Unsafe ZIP path"
        );
        anyhow::ensure!(names.insert(trimmed.to_owned()), "Duplicate ZIP entry");
        let mode = entry.unix_mode().unwrap_or(0) & 0o170000;
        anyhow::ensure!(
            mode == 0
                || (entry.is_dir() && mode == 0o040000)
                || (!entry.is_dir() && mode == 0o100000),
            "ZIP contains a link or special file"
        );
        let output = target.join(trimmed);
        if entry.is_dir() {
            fs::create_dir_all(output)?;
        } else {
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent)?;
            }
            let _ = progress.send(format!("Extracting restore: {name}"));
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(output)?;
            std::io::copy(&mut entry, &mut file)?;
            file.sync_all()?;
        }
    }
    Ok(())
}

fn move_contents(source: &Path, target: &Path, moved: &mut Vec<OsString>) -> Result<()> {
    for entry in fs::read_dir(source)? {
        let name = entry?.file_name();
        match fs::symlink_metadata(target.join(&name)) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
            Ok(_) => bail!("Restore destination changed during replacement"),
        }
        fs::rename(source.join(&name), target.join(&name))?;
        moved.push(name);
    }
    Ok(())
}

fn rollback(
    source: &Path,
    stage: &Path,
    previous: &Path,
    moved: &[OsString],
    installed: &[OsString],
) -> Result<()> {
    let mut errors = Vec::new();
    for name in installed.iter().rev() {
        if let Err(error) = fs::rename(source.join(name), stage.join(name)) {
            errors.push(error.to_string());
        }
    }
    for name in moved.iter().rev() {
        let result = (|| -> Result<()> {
            anyhow::ensure!(
                fs::symlink_metadata(source.join(name))
                    .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound),
                "Cannot overwrite a changed source entry during rollback"
            );
            fs::rename(previous.join(name), source.join(name))?;
            Ok(())
        })();
        if let Err(error) = result {
            errors.push(error.to_string());
        }
    }
    anyhow::ensure!(errors.is_empty(), "{}", errors.join("; "));
    Ok(())
}

fn remove_contents(root: &Path) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        let metadata = fs::symlink_metadata(&path)?;
        if is_link(&metadata) {
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                if metadata.file_attributes() & 0x10 != 0 {
                    fs::remove_dir(&path)?;
                } else {
                    fs::remove_file(&path)?;
                }
            }
            #[cfg(not(windows))]
            fs::remove_file(&path)?;
        } else if metadata.is_dir() {
            remove_contents(&path)?;
            fs::remove_dir(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(zip: bool) -> (tempfile::TempDir, Job, String) {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("source");
        let target = directory.path().join("backups");
        fs::create_dir_all(source.join("nested/empty")).unwrap();
        fs::create_dir(&target).unwrap();
        fs::write(source.join("nested/file"), "saved").unwrap();
        fs::write(source.join(".hidden"), "hidden").unwrap();
        let job = Job {
            source,
            target,
            zip,
            ..Job::default()
        };
        super::super::run(&job, &std::sync::mpsc::channel().0).unwrap();
        let name = list(&job).unwrap()[0].record.name.clone();
        (directory, job, name)
    }

    fn change_source(job: &Job) {
        fs::remove_dir_all(job.source.join("nested")).unwrap();
        fs::write(job.source.join(".hidden"), "changed").unwrap();
        fs::create_dir_all(job.source.join("additional/directory")).unwrap();
        fs::write(job.source.join("additional/directory/file"), "remove").unwrap();
    }

    #[test]
    fn restore_replaces_entire_contents_and_preserves_history() {
        for zip in [false, true] {
            let (directory, job, name) = fixture(zip);
            let before = fs::read(job.target.join(format!("manifest-{}.json", job.id))).unwrap();
            change_source(&job);
            let result = run(&job, &name, &std::sync::mpsc::channel().0).unwrap();
            assert!(!result.created);
            assert_eq!(
                fs::read_to_string(job.source.join("nested/file")).unwrap(),
                "saved"
            );
            assert_eq!(
                fs::read_to_string(job.source.join(".hidden")).unwrap(),
                "hidden"
            );
            assert!(job.source.join("nested/empty").is_dir());
            assert!(!job.source.join("additional").exists());
            assert_eq!(
                before,
                fs::read(job.target.join(format!("manifest-{}.json", job.id))).unwrap()
            );
            assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
            let listed = list(&job).unwrap();
            assert!(listed[0].available);
            assert_eq!(listed[0].zip, zip);
        }
    }

    #[test]
    fn damaged_and_missing_backups_leave_source_untouched() {
        for zip in [false, true] {
            let (_directory, job, name) = fixture(zip);
            change_source(&job);
            let original = inventory(&job.source).unwrap().0;
            let backup = job.target.join(&name);
            if zip {
                fs::write(&backup, "invalid zip").unwrap();
            } else {
                fs::write(backup.join("nested/file"), "corrupt").unwrap();
            }
            assert!(run(&job, &name, &std::sync::mpsc::channel().0).is_err());
            assert_eq!(original, inventory(&job.source).unwrap().0);
            if zip {
                fs::remove_file(backup).unwrap();
            } else {
                fs::remove_dir_all(backup).unwrap();
            }
            assert!(!list(&job).unwrap()[0].available);
            assert!(run(&job, &name, &std::sync::mpsc::channel().0).is_err());
            assert_eq!(original, inventory(&job.source).unwrap().0);
        }
    }

    #[test]
    fn replacement_failure_rolls_back_original_contents() {
        for zip in [false, true] {
            let (directory, job, name) = fixture(zip);
            change_source(&job);
            let original = inventory(&job.source).unwrap().0;
            let error = restore_inner(&job, &name, &std::sync::mpsc::channel().0, || {
                bail!("Injected install failure")
            })
            .err()
            .unwrap();
            assert!(error
                .to_string()
                .contains("original source contents restored"));
            assert_eq!(original, inventory(&job.source).unwrap().0);
            assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
        }
    }

    #[test]
    fn failed_rollback_retains_original_data_and_reports_path() {
        let (directory, job, name) = fixture(false);
        change_source(&job);
        let error = restore_inner(&job, &name, &std::sync::mpsc::channel().0, || {
            fs::create_dir(job.source.join("additional"))?;
            bail!("Injected external change")
        })
        .err()
        .unwrap();
        let previous = fs::read_dir(directory.path())
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| {
                p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(".portbak-previous-")
            })
            .unwrap();
        assert!(previous.join("additional/directory/file").is_file());
        assert!(error.to_string().contains(previous.to_str().unwrap()));
    }

    #[test]
    fn foreign_history_overlaps_and_locked_destination_are_rejected() {
        let (_directory, mut job, name) = fixture(false);
        let id = job.id;
        job.id = uuid::Uuid::new_v4();
        assert!(run(&job, &name, &std::sync::mpsc::channel().0).is_err());
        job.id = id;
        let source = job.source.clone();
        job.source = job.target.clone();
        assert!(run(&job, &name, &std::sync::mpsc::channel().0).is_err());
        job.source = source;
        let lock = lock_destination(&job.target).unwrap();
        assert!(run(&job, &name, &std::sync::mpsc::channel().0).is_err());
        drop(lock);
        assert!(run(&job, "../outside", &std::sync::mpsc::channel().0).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn restore_never_follows_links() {
        let (directory, job, name) = fixture(false);
        let outside = directory.path().join("outside");
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("keep"), "safe").unwrap();
        std::os::unix::fs::symlink(&outside, job.source.join("link")).unwrap();
        run(&job, &name, &std::sync::mpsc::channel().0).unwrap();
        assert_eq!(fs::read_to_string(outside.join("keep")).unwrap(), "safe");
        assert!(!job.source.join("link").exists());
        std::os::unix::fs::symlink(&outside, job.target.join(&name).join("link")).unwrap();
        assert!(run(&job, &name, &std::sync::mpsc::channel().0).is_err());
        let mut linked = job.clone();
        linked.source = directory.path().join("linked-source");
        std::os::unix::fs::symlink(&job.source, &linked.source).unwrap();
        assert!(run(&linked, &name, &std::sync::mpsc::channel().0).is_err());
    }

    #[test]
    fn zip_links_are_rejected() {
        let (_directory, job, name) = fixture(true);
        let original = inventory(&job.source).unwrap().0;
        let mut zip = ZipWriter::new(File::create(job.target.join(&name)).unwrap());
        zip.add_symlink("link", "../outside", SimpleFileOptions::default())
            .unwrap();
        zip.finish().unwrap();
        let error = run(&job, &name, &std::sync::mpsc::channel().0)
            .err()
            .unwrap();
        assert!(error.to_string().contains("link or special file"));
        assert_eq!(original, inventory(&job.source).unwrap().0);
    }

    #[cfg(unix)]
    #[test]
    fn folder_special_files_are_rejected() {
        let (_directory, job, name) = fixture(false);
        let original = inventory(&job.source).unwrap().0;
        let _socket =
            std::os::unix::net::UnixDatagram::bind(job.target.join(&name).join("socket")).unwrap();
        assert!(run(&job, &name, &std::sync::mpsc::channel().0).is_err());
        assert_eq!(original, inventory(&job.source).unwrap().0);
    }

    #[cfg(windows)]
    #[test]
    fn locked_source_file_preserves_original_contents() {
        use std::os::windows::fs::OpenOptionsExt;
        let (_directory, job, name) = fixture(false);
        change_source(&job);
        let original = inventory(&job.source).unwrap().0;
        let lock = OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(job.source.join(".hidden"))
            .unwrap();
        assert!(run(&job, &name, &std::sync::mpsc::channel().0).is_err());
        drop(lock);
        assert_eq!(original, inventory(&job.source).unwrap().0);
    }

    #[test]
    fn unsafe_zip_entries_are_rejected_before_source_changes() {
        for names in [
            vec!["../escape"],
            vec!["/absolute"],
            vec!["C:/absolute"],
            vec!["a\\b"],
            vec!["same", "same/"],
        ] {
            let (_directory, job, name) = fixture(true);
            let original = inventory(&job.source).unwrap().0;
            let mut zip = ZipWriter::new(File::create(job.target.join(&name)).unwrap());
            for name in names {
                if name.ends_with('/') {
                    zip.add_directory(name, SimpleFileOptions::default())
                        .unwrap();
                } else {
                    zip.start_file(name, SimpleFileOptions::default()).unwrap();
                    zip.write_all(b"unsafe").unwrap();
                }
            }
            zip.finish().unwrap();
            assert!(run(&job, &name, &std::sync::mpsc::channel().0).is_err());
            assert_eq!(original, inventory(&job.source).unwrap().0);
        }
    }
}
