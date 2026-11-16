use super::filesystem::{is_link, metadata_if_exists};
use crate::{model::Manifest, persistence::load};
use anyhow::{bail, Context, Result};
use fs2::FileExt;
use std::{
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
};
use uuid::Uuid;

const MANIFEST_VERSION: u32 = 1;

pub(super) fn manifest_path(root: &Path, id: Uuid) -> PathBuf {
    root.join(format!("manifest-{id}.json"))
}

fn reject_metadata_link(path: &Path) -> Result<()> {
    if let Some(metadata) = metadata_if_exists(path)? {
        anyhow::ensure!(!is_link(&metadata), "Backup metadata must not be a link");
    }
    Ok(())
}

pub(super) fn lock_destination(root: &Path) -> Result<File> {
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

pub(super) fn read_manifest(root: &Path, id: Uuid) -> Result<Manifest> {
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
