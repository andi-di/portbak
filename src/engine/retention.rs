use super::{counted, filesystem::remove_backup};
use crate::{
    model::{Job, Manifest, Record},
    persistence::save,
};
use anyhow::Result;
use std::path::Path;

/// Warn once three or fewer unprotected places remain.
const LOW_FREE_PLACES: usize = 3;
/// Warn once 20 % or less of the size budget remains unprotected.
const LOW_FREE_PERCENT: u128 = 20;

/// Deletes the oldest unprotected backups until the limits hold; the newest is always kept.
pub(super) fn prune(
    root: &Path,
    manifest_path: &Path,
    manifest: &mut Manifest,
    job: &Job,
) -> Result<()> {
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

pub(super) fn exceeds_limits(job: &Job, records: &[Record]) -> bool {
    let counted = records
        .iter()
        .filter(|record| job.count_protected || !record.protected);
    let (count, total) = counted.fold((0usize, 0u128), |(count, total), record| {
        (count + 1, total + u128::from(record.bytes))
    });
    job.max_count.is_some_and(|limit| count > limit.max(1))
        || job.max_bytes.is_some_and(|limit| total > u128::from(limit))
}

/// Explains how much room protected backups leave for rolling retention.
pub fn warnings(job: &Job, records: &[Record]) -> Vec<String> {
    let protected: Vec<&Record> = records.iter().filter(|record| record.protected).collect();
    if !job.rolling || !job.count_protected || protected.is_empty() {
        return Vec::new();
    }
    let mut warnings = Vec::new();
    if let Some(limit) = job.max_count {
        let free = limit.saturating_sub(protected.len());
        if free <= LOW_FREE_PLACES {
            let verb = if free == 1 { "remains" } else { "remain" };
            warnings.push(format!(
                "Low retention capacity: {} {verb} ({} protected, limit {limit}).",
                counted(free, "place", "places"),
                protected.len()
            ));
        }
    }
    if let Some(limit) = job.max_bytes {
        let limit = u128::from(limit);
        let protected_bytes: u128 = protected
            .iter()
            .map(|record| u128::from(record.bytes))
            .sum();
        let free = limit.saturating_sub(protected_bytes);
        if free * 100 <= limit * LOW_FREE_PERCENT {
            let percent = if limit == 0 {
                0.0
            } else {
                free as f64 * 100.0 / limit as f64
            };
            warnings.push(format!(
                "Low retention size budget: {free} bytes ({percent:.1}%) remain \
                 after protected backups; limit {limit} bytes."
            ));
        }
    }
    if exceeds_limits(job, records) {
        warnings.push(
            "Current history exceeds retention limits; \
             protected backups and the newest version are kept."
                .into(),
        );
    }
    warnings
}
