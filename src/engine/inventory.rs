use super::filesystem::is_link;
use anyhow::{Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{self, Read},
    path::Path,
};

/// One file or folder of a backup; its serialized form defines the backup fingerprint.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(super) struct Entry {
    pub(super) path: String,
    pub(super) hash: String,
    pub(super) size: u64,
    pub(super) directory: bool,
}

pub(super) fn hash_reader(mut reader: impl Read) -> Result<String> {
    let mut hasher = Sha256::new();
    io::copy(&mut reader, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

pub(super) fn inventory(root: &Path) -> Result<(Vec<Entry>, usize)> {
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

pub(super) fn digest(entries: &[Entry]) -> Result<String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(entries)?)
    ))
}
