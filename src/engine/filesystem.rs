use crate::model::Job;
use anyhow::{Context, Result};
use std::{
    fs::{self, File, Metadata, OpenOptions},
    io,
    path::{Path, PathBuf},
};

/// Removes a partially written file or folder when dropped.
pub(super) struct TempPath(pub(super) PathBuf);

impl Drop for TempPath {
    fn drop(&mut self) {
        let _ = if self.0.is_dir() {
            fs::remove_dir_all(&self.0)
        } else {
            fs::remove_file(&self.0)
        };
    }
}

pub(super) fn destination(job: &Job) -> Result<PathBuf> {
    fs::canonicalize(&job.target).context("Destination folder is unavailable")
}

pub(super) fn ensure_disjoint(source: &Path, target: &Path) -> Result<()> {
    anyhow::ensure!(
        !source.starts_with(target) && !target.starts_with(source),
        "Source and destination folders must not overlap"
    );
    Ok(())
}

pub(super) fn is_zip_backup(name: &str) -> bool {
    name.ends_with(".zip")
}

/// Copies into a fresh file only; an existing output means the workspace was tampered with.
pub(super) fn copy_file(from: &Path, to: &Path) -> Result<()> {
    let mut output = OpenOptions::new().write(true).create_new(true).open(to)?;
    io::copy(&mut File::open(from)?, &mut output)?;
    output.sync_all()?;
    #[cfg(unix)]
    fs::set_permissions(to, fs::metadata(from)?.permissions())?;
    Ok(())
}

pub(super) fn remove_backup(path: &Path) -> Result<()> {
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

pub(super) fn metadata_if_exists(path: &Path) -> Result<Option<Metadata>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

pub(super) fn is_link(metadata: &Metadata) -> bool {
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

pub(super) fn disk_size(path: &Path) -> Result<u64> {
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
