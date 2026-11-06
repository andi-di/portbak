use super::{
    copy_file, destination, digest, ensure_disjoint, inventory, is_link, is_zip_backup,
    lock_destination, metadata_if_exists, read_manifest, report, Outcome, TempPath,
};
use crate::model::Job;
use anyhow::{bail, Context, Result};
use std::{
    collections::HashSet,
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
    sync::mpsc::Sender,
};

pub fn run(job: &Job, name: &str, progress: &Sender<String>) -> Result<Outcome> {
    restore_inner(job, name, progress, || Ok(()))
}

fn restore_inner(
    job: &Job,
    name: &str,
    progress: &Sender<String>,
    after_install: impl FnOnce() -> Result<()>,
) -> Result<Outcome> {
    let source = restorable_source(job)?;
    let parent = source
        .parent()
        .context("Cannot restore into a filesystem root")?;
    let target = destination(job)?;
    ensure_disjoint(&source, &target)?;
    let _lock = lock_destination(&target)?;
    let manifest = read_manifest(&target, job.id)?;
    let record = manifest
        .records
        .iter()
        .find(|record| record.name == name)
        .context("Selected backup is no longer in this job's history")?;
    let stage = TempPath(create_workspace(parent, ".portbak-restore-")?);
    report(progress, "Preparing and verifying selected backup …");
    stage_backup(&target, name, &stage.0, progress)?;
    verify(&stage.0, &record.hash)?;
    let previous = create_workspace(parent, ".portbak-previous-")?;
    replace_contents(&source, &stage.0, &previous, progress, || {
        after_install()?;
        verify(&source, &record.hash)
    })?;
    report(progress, "Removing replaced source contents …");
    Ok(Outcome {
        message: discard_previous(&previous),
        created: false,
    })
}

fn restorable_source(job: &Job) -> Result<PathBuf> {
    anyhow::ensure!(
        !job.source.as_os_str().is_empty(),
        "Please select a source folder"
    );
    let metadata = fs::symlink_metadata(&job.source).context("Source folder is unavailable")?;
    anyhow::ensure!(
        metadata.is_dir() && !is_link(&metadata),
        "Restore source must be an ordinary folder, not a link"
    );
    Ok(fs::canonicalize(&job.source)?)
}

/// Workspaces sit beside the source so every later move is a same-volume rename.
fn create_workspace(parent: &Path, prefix: &str) -> Result<PathBuf> {
    let path = parent.join(format!("{prefix}{}", uuid::Uuid::new_v4()));
    fs::create_dir(&path).context("Cannot create restore workspace beside source folder")?;
    Ok(path)
}

fn stage_backup(target: &Path, name: &str, stage: &Path, progress: &Sender<String>) -> Result<()> {
    let backup = target.join(name);
    let metadata = fs::symlink_metadata(&backup).context("Selected backup is unavailable")?;
    anyhow::ensure!(!is_link(&metadata), "Selected backup must not be a link");
    if is_zip_backup(name) {
        anyhow::ensure!(metadata.is_file(), "Selected ZIP backup is not a file");
        extract_zip(&backup, stage, progress)
    } else {
        anyhow::ensure!(
            metadata.is_dir(),
            "Selected folder backup is not a directory"
        );
        copy_folder_backup(&backup, stage, progress)
    }
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

fn copy_folder_backup(source: &Path, target: &Path, progress: &Sender<String>) -> Result<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        anyhow::ensure!(!is_link(&metadata), "Folder backup contains a link");
        let output = target.join(entry.file_name());
        if metadata.is_dir() {
            fs::create_dir(&output)?;
            copy_folder_backup(&path, &output, progress)?;
        } else {
            anyhow::ensure!(metadata.is_file(), "Folder backup contains a special file");
            report(progress, format!("Preparing restore: {}", path.display()));
            copy_file(&path, &output)?;
        }
    }
    Ok(())
}

fn extract_zip(source: &Path, target: &Path, progress: &Sender<String>) -> Result<()> {
    let mut archive = zip::ZipArchive::new(File::open(source)?)?;
    let mut names = HashSet::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = safe_zip_name(entry.name_raw())?;
        anyhow::ensure!(names.insert(name.clone()), "Duplicate ZIP entry");
        let file_type = entry.unix_mode().unwrap_or(0) & 0o170000;
        let expected_type = if entry.is_dir() { 0o040000 } else { 0o100000 };
        anyhow::ensure!(
            file_type == 0 || file_type == expected_type,
            "ZIP contains a link or special file"
        );
        let output = target.join(&name);
        if entry.is_dir() {
            fs::create_dir_all(output)?;
            continue;
        }
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        report(progress, format!("Extracting restore: {name}"));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output)?;
        io::copy(&mut entry, &mut file)?;
        file.sync_all()?;
    }
    Ok(())
}

/// Accepts only relative `/`-separated names, so extraction can never leave the stage.
fn safe_zip_name(raw: &[u8]) -> Result<String> {
    let name = std::str::from_utf8(raw).context("ZIP name is not valid UTF-8")?;
    let trimmed = name.trim_end_matches('/');
    let safe = !name.contains(['\\', ':'])
        && !name.starts_with('/')
        && !trimmed.is_empty()
        && trimmed
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..");
    anyhow::ensure!(safe, "Unsafe ZIP path");
    Ok(trimmed.to_owned())
}

/// Swaps the staged backup into the source; any failure moves the original contents back.
fn replace_contents(
    source: &Path,
    stage: &Path,
    previous: &Path,
    progress: &Sender<String>,
    check_installed: impl FnOnce() -> Result<()>,
) -> Result<()> {
    let mut moved = Vec::new();
    let mut installed = Vec::new();
    let result = (|| {
        report(progress, "Moving current source contents aside …");
        move_contents(source, previous, &mut moved)?;
        report(progress, "Installing selected backup …");
        move_contents(stage, source, &mut installed)?;
        check_installed()
    })();
    let Err(error) = result else {
        return Ok(());
    };
    match rollback(source, stage, previous, &moved, &installed) {
        Ok(()) => {
            fs::remove_dir(previous).with_context(|| {
                format!(
                    "Restore failed: {error:#}. Original contents restored, \
                     but the empty recovery folder {} could not be removed",
                    previous.display()
                )
            })?;
            Err(error.context("Restore failed; original source contents restored"))
        }
        Err(recovery) => bail!(
            "Restore failed: {error:#}. Rollback incomplete: {recovery:#}. \
             Original data remains at {} and {}. Do not delete the recovery folder.",
            source.display(),
            previous.display()
        ),
    }
}

/// Records each moved name so a failure can undo exactly what happened.
fn move_contents(source: &Path, target: &Path, moved: &mut Vec<OsString>) -> Result<()> {
    for entry in fs::read_dir(source)? {
        let name = entry?.file_name();
        move_entry(source, target, &name)?;
        moved.push(name);
    }
    Ok(())
}

/// Never overwrites: an existing destination entry means something changed during the restore.
fn move_entry(from: &Path, to: &Path, name: &OsString) -> Result<()> {
    anyhow::ensure!(
        metadata_if_exists(&to.join(name))?.is_none(),
        "Restore destination changed during replacement"
    );
    fs::rename(from.join(name), to.join(name))?;
    Ok(())
}

fn rollback(
    source: &Path,
    stage: &Path,
    previous: &Path,
    moved: &[OsString],
    installed: &[OsString],
) -> Result<()> {
    let installed = installed
        .iter()
        .rev()
        .map(|name| move_entry(source, stage, name));
    let originals = moved
        .iter()
        .rev()
        .map(|name| move_entry(previous, source, name));
    let errors: Vec<String> = installed
        .chain(originals)
        .filter_map(|result| result.err().map(|error| format!("{error:#}")))
        .collect();
    anyhow::ensure!(errors.is_empty(), "{}", errors.join("; "));
    Ok(())
}

fn discard_previous(previous: &Path) -> String {
    match remove_contents(previous).and_then(|()| Ok(fs::remove_dir(previous)?)) {
        Ok(()) => "Restore complete · automatic backups disabled".to_owned(),
        Err(error) => format!(
            "Restore complete · old contents could not be fully removed at {}: {error:#}",
            previous.display()
        ),
    }
}

/// Deletes links themselves, never their targets.
fn remove_contents(root: &Path) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        let metadata = fs::symlink_metadata(&path)?;
        if is_link(&metadata) {
            remove_link(&path, &metadata)?;
        } else if metadata.is_dir() {
            remove_contents(&path)?;
            fs::remove_dir(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

#[cfg_attr(not(windows), allow(unused_variables))]
fn remove_link(path: &Path, metadata: &fs::Metadata) -> Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x10;
        // Directory links and junctions are removed like empty folders on Windows.
        if metadata.file_attributes() & FILE_ATTRIBUTE_DIRECTORY != 0 {
            return Ok(fs::remove_dir(path)?);
        }
    }
    Ok(fs::remove_file(path)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{discard_progress, history::list, manifest_path};
    use std::io::Write;
    use zip::{write::SimpleFileOptions, ZipWriter};

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
        crate::engine::run(&job, &discard_progress()).unwrap();
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
            let before = fs::read(manifest_path(&job.target, job.id)).unwrap();
            change_source(&job);
            let result = run(&job, &name, &discard_progress()).unwrap();
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
                fs::read(manifest_path(&job.target, job.id)).unwrap()
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
            assert!(run(&job, &name, &discard_progress()).is_err());
            assert_eq!(original, inventory(&job.source).unwrap().0);
            if zip {
                fs::remove_file(backup).unwrap();
            } else {
                fs::remove_dir_all(backup).unwrap();
            }
            assert!(!list(&job).unwrap()[0].available);
            assert!(run(&job, &name, &discard_progress()).is_err());
            assert_eq!(original, inventory(&job.source).unwrap().0);
        }
    }

    #[test]
    fn replacement_failure_rolls_back_original_contents() {
        for zip in [false, true] {
            let (directory, job, name) = fixture(zip);
            change_source(&job);
            let original = inventory(&job.source).unwrap().0;
            let error = restore_inner(&job, &name, &discard_progress(), || {
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
        let error = restore_inner(&job, &name, &discard_progress(), || {
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
        // Restore reports canonical paths, including Windows prefixes and resolved aliases.
        let reported_path = fs::canonicalize(&previous).unwrap();
        assert!(
            error.to_string().contains(reported_path.to_str().unwrap()),
            "Recovery path missing from restore error: {error:#}"
        );
    }

    #[test]
    fn foreign_history_overlaps_and_locked_destination_are_rejected() {
        let (_directory, mut job, name) = fixture(false);
        let id = job.id;
        job.id = uuid::Uuid::new_v4();
        assert!(run(&job, &name, &discard_progress()).is_err());
        job.id = id;
        let source = job.source.clone();
        job.source = job.target.clone();
        assert!(run(&job, &name, &discard_progress()).is_err());
        job.source = source;
        let lock = lock_destination(&job.target).unwrap();
        assert!(run(&job, &name, &discard_progress()).is_err());
        drop(lock);
        assert!(run(&job, "../outside", &discard_progress()).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn restore_never_follows_links() {
        let (directory, job, name) = fixture(false);
        let outside = directory.path().join("outside");
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("keep"), "safe").unwrap();
        std::os::unix::fs::symlink(&outside, job.source.join("link")).unwrap();
        run(&job, &name, &discard_progress()).unwrap();
        assert_eq!(fs::read_to_string(outside.join("keep")).unwrap(), "safe");
        assert!(!job.source.join("link").exists());
        std::os::unix::fs::symlink(&outside, job.target.join(&name).join("link")).unwrap();
        assert!(run(&job, &name, &discard_progress()).is_err());
        let mut linked = job.clone();
        linked.source = directory.path().join("linked-source");
        std::os::unix::fs::symlink(&job.source, &linked.source).unwrap();
        assert!(run(&linked, &name, &discard_progress()).is_err());
    }

    #[test]
    fn zip_links_are_rejected() {
        let (_directory, job, name) = fixture(true);
        let original = inventory(&job.source).unwrap().0;
        let mut zip = ZipWriter::new(File::create(job.target.join(&name)).unwrap());
        zip.add_symlink("link", "../outside", SimpleFileOptions::default())
            .unwrap();
        zip.finish().unwrap();
        let error = run(&job, &name, &discard_progress()).err().unwrap();
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
        assert!(run(&job, &name, &discard_progress()).is_err());
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
        assert!(run(&job, &name, &discard_progress()).is_err());
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
            assert!(run(&job, &name, &discard_progress()).is_err());
            assert_eq!(original, inventory(&job.source).unwrap().0);
        }
    }
}
