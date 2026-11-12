use super::{
    counted, destination, exceeds_limits, is_link, is_zip_backup, lock_destination, manifest_path,
    metadata_if_exists, read_manifest, save,
};
use crate::model::{Job, Record};
use anyhow::{Context, Result};
use std::cmp::Reverse;

/// Warn once three or fewer unprotected places remain.
const LOW_FREE_PLACES: usize = 3;
/// Warn once 20 % or less of the size budget remains unprotected.
const LOW_FREE_PERCENT: u128 = 20;

#[derive(Clone)]
pub struct Backup {
    pub record: Record,
    pub available: bool,
    pub zip: bool,
}

/// Lists newest first without taking the destination lock: manifests are replaced atomically,
/// and locking would leave `.lock` files in every folder typed into the destination field.
pub fn list(job: &Job) -> Result<Vec<Backup>> {
    let target = destination(job)?;
    let manifest = read_manifest(&target, job.id)?;
    let mut backups = manifest
        .records
        .into_iter()
        .rev()
        .map(|record| {
            let zip = is_zip_backup(&record.name);
            let available = metadata_if_exists(&target.join(&record.name))?
                .is_some_and(|m| !is_link(&m) && if zip { m.is_file() } else { m.is_dir() });
            Ok(Backup {
                record,
                available,
                zip,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    backups.sort_by_key(|backup| Reverse(backup.record.created));
    Ok(backups)
}

/// Removes only an unlocked version owned by this job.
pub fn delete(job: &Job, name: &str) -> Result<()> {
    let target = destination(job)?;
    let _lock = lock_destination(&target)?;
    let mut manifest = read_manifest(&target, job.id)?;
    let index = manifest
        .records
        .iter()
        .position(|record| record.name == name)
        .context("Selected backup is no longer in this job's history")?;
    anyhow::ensure!(
        !manifest.records[index].protected,
        "Unlock this backup before deleting it"
    );
    super::remove_backup(&target.join(name))?;
    manifest.records.remove(index);
    save(&manifest_path(&target, job.id), &manifest)
}

pub fn set_protected(job: &Job, name: &str, protected: bool) -> Result<()> {
    let target = destination(job)?;
    let _lock = lock_destination(&target)?;
    let mut manifest = read_manifest(&target, job.id)?;
    manifest
        .records
        .iter_mut()
        .find(|record| record.name == name)
        .context("Selected backup is no longer in this job's history")?
        .protected = protected;
    save(&manifest_path(&target, job.id), &manifest)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{load, prune};
    use crate::model::Manifest;
    use chrono::Utc;
    use std::{fs, path::PathBuf};

    #[test]
    fn manual_delete_requires_unlock_and_allows_last_missing_version() {
        let (_directory, job, original, path) = fixture(vec![record(0, true, 10)]);
        let name = &original.records[0].name;
        assert!(delete(&job, name).is_err());
        assert!(job.target.join(name).exists());
        set_protected(&job, name, false).unwrap();
        delete(&job, name).unwrap();
        assert!(!job.target.join(name).exists());
        assert!(load::<Manifest>(&path).unwrap().records.is_empty());
        assert!(delete(&job, name).is_err());
        save(&path, &original).unwrap();
        set_protected(&job, name, false).unwrap();
        delete(&job, name).unwrap();
        assert!(list(&job).unwrap().is_empty());
    }

    #[test]
    fn capacity_warning_uses_singular_for_one_place() {
        let job = Job {
            rolling: true,
            count_protected: true,
            max_count: Some(3),
            max_bytes: None,
            ..Job::default()
        };
        let records = vec![record(1, true, 1), record(2, true, 1)];
        assert_eq!(
            warnings(&job, &records),
            ["Low retention capacity: 1 place remains (2 protected, limit 3)."]
        );
    }

    fn record(index: u32, protected: bool, size: u64) -> Record {
        Record {
            name: format!("2026-backup-{index}"),
            hash: index.to_string(),
            bytes: size,
            created: Utc::now(),
            protected,
        }
    }

    fn fixture(records: Vec<Record>) -> (tempfile::TempDir, Job, Manifest, PathBuf) {
        let directory = tempfile::tempdir().unwrap();
        let job = Job {
            target: directory.path().to_path_buf(),
            rolling: true,
            count_protected: true,
            max_count: Some(2),
            ..Job::default()
        };
        let manifest = Manifest {
            version: 1,
            job_id: Some(job.id),
            records,
        };
        for record in &manifest.records {
            fs::write(job.target.join(&record.name), b"saved").unwrap();
        }
        let path = manifest_path(&job.target, job.id);
        save(&path, &manifest).unwrap();
        (directory, job, manifest, path)
    }

    #[test]
    fn protected_backups_are_optional_in_both_budgets_but_never_deleted() {
        for count_protected in [false, true] {
            for size_limit in [false, true] {
                let (_directory, mut job, mut manifest, path) = fixture(vec![
                    record(0, true, 10),
                    record(1, false, 10),
                    record(2, false, 10),
                    record(3, false, 10),
                ]);
                job.count_protected = count_protected;
                if size_limit {
                    job.max_count = None;
                    job.max_bytes = Some(20);
                }
                prune(&job.target, &path, &mut manifest, &job).unwrap();
                assert!(job.target.join("2026-backup-0").exists());
                assert!(manifest.records[0].protected);
                assert_eq!(manifest.records.len(), if count_protected { 2 } else { 3 });
                assert_eq!(manifest.records.last().unwrap().name, "2026-backup-3");
                if !count_protected {
                    assert!(warnings(&job, &manifest.records).is_empty());
                }
            }
        }
    }

    #[test]
    fn protection_roundtrips_without_changing_other_fields_or_pruning() {
        let (_directory, job, original, path) =
            fixture(vec![record(0, false, 10), record(1, false, 20)]);
        let name = &original.records[0].name;
        set_protected(&job, name, true).unwrap();
        assert!(list(&job)
            .unwrap()
            .iter()
            .any(|b| b.record.name == *name && b.record.protected));
        let saved: Manifest = load(&path).unwrap();
        assert_eq!(saved.records[0].hash, original.records[0].hash);
        assert_eq!(saved.records[0].created, original.records[0].created);
        assert!(!saved.records[1].protected);
        set_protected(&job, name, false).unwrap();
        assert!(job.target.join(name).exists());
        assert_eq!(load::<Manifest>(&path).unwrap().records.len(), 2);
        let mut value = serde_json::to_value(&original.records[0]).unwrap();
        value.as_object_mut().unwrap().remove("protected");
        assert!(!serde_json::from_value::<Record>(value).unwrap().protected);
    }

    #[test]
    fn retention_skips_protected_and_keeps_newest_under_both_limits() {
        for size_limit in [false, true] {
            let (_directory, mut job, mut manifest, path) = fixture(vec![
                record(0, true, 10),
                record(1, false, 10),
                record(2, false, 10),
                record(3, false, 10),
            ]);
            if size_limit {
                job.max_count = None;
                job.max_bytes = Some(20);
            }
            prune(&job.target, &path, &mut manifest, &job).unwrap();
            assert_eq!(
                manifest
                    .records
                    .iter()
                    .map(|r| r.name.as_str())
                    .collect::<Vec<_>>(),
                ["2026-backup-0", "2026-backup-3"]
            );
            assert!(job.target.join("2026-backup-0").exists());
            assert!(!job.target.join("2026-backup-1").exists());
        }
    }

    #[test]
    fn all_protected_records_can_exceed_limits_without_deletion() {
        let (_directory, mut job, mut manifest, path) = fixture(vec![
            record(0, true, 10),
            record(1, true, 10),
            record(2, false, 10),
        ]);
        job.max_count = Some(1);
        job.max_bytes = Some(5);
        prune(&job.target, &path, &mut manifest, &job).unwrap();
        assert_eq!(manifest.records.len(), 3);
        assert!(warnings(&job, &manifest.records)
            .iter()
            .any(|warning| warning.contains("exceeds")));
        manifest.records[2].protected = true;
        prune(&job.target, &path, &mut manifest, &job).unwrap();
        assert_eq!(manifest.records.len(), 3);
    }

    #[test]
    fn warnings_use_three_places_and_twenty_percent_inclusive() {
        let (_directory, mut job, manifest, _path) = fixture(vec![record(0, true, 80)]);
        job.max_count = Some(5);
        job.max_bytes = Some(101);
        assert!(warnings(&job, &manifest.records).is_empty());
        job.max_count = Some(4);
        let messages = warnings(&job, &manifest.records);
        assert!(messages.iter().any(|message| message.contains("3 places")));
        assert!(!messages
            .iter()
            .any(|message| message.contains("size budget")));
        job.max_bytes = Some(100);
        assert!(warnings(&job, &manifest.records)
            .iter()
            .any(|message| message.contains("20.0%")));
        job.rolling = false;
        assert!(warnings(&job, &manifest.records).is_empty());
        job.rolling = true;
        assert!(warnings(&job, &[record(0, false, 100)]).is_empty());
    }

    #[test]
    fn protection_errors_preserve_manifest_and_allow_unprotecting_missing_backups() {
        let (_directory, job, original, path) = fixture(vec![record(0, true, 10)]);
        let bytes = fs::read(&path).unwrap();
        let lock = lock_destination(&job.target).unwrap();
        assert!(set_protected(&job, &original.records[0].name, false).is_err());
        assert_eq!(bytes, fs::read(&path).unwrap());
        drop(lock);
        assert!(set_protected(&job, "missing", true).is_err());
        fs::remove_file(job.target.join(&original.records[0].name)).unwrap();
        set_protected(&job, &original.records[0].name, false).unwrap();
        assert!(!load::<Manifest>(&path).unwrap().records[0].protected);
    }

    #[test]
    fn listing_neither_creates_nor_waits_for_the_lock() {
        let directory = tempfile::tempdir().unwrap();
        let job = Job {
            target: directory.path().to_path_buf(),
            ..Job::default()
        };
        assert!(list(&job).unwrap().is_empty());
        assert!(!job.target.join(".lock").exists());
        let (_directory, job, manifest, _path) = fixture(vec![record(0, false, 10)]);
        let _lock = lock_destination(&job.target).unwrap();
        assert_eq!(list(&job).unwrap()[0].record.name, manifest.records[0].name);
    }
}
