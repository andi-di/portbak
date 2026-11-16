use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::{fs, io::Write, path::Path};

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
