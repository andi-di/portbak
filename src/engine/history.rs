use super::*;

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
    backups.sort_by_key(|backup| std::cmp::Reverse(backup.record.created));
    Ok(backups)
}

pub fn set_protected(job: &Job, name: &str, protected: bool) -> Result<()> {
    let target = fs::canonicalize(&job.target).context("Destination folder is unavailable")?;
    let _lock = lock_destination(&target)?;
    let mut manifest = read_manifest(&target, job.id)?;
    let record = manifest
        .records
        .iter_mut()
        .find(|record| record.name == name)
        .context("Selected backup is no longer in this job's history")?;
    record.protected = protected;
    save(&target.join(format!("manifest-{}.json", job.id)), &manifest)
}

pub fn warnings(job: &Job, records: &[Record]) -> Vec<String> {
    let protected_count = records.iter().filter(|record| record.protected).count();
    if !job.rolling || protected_count == 0 {
        return Vec::new();
    }
    let protected_bytes = records
        .iter()
        .filter(|record| record.protected)
        .fold(0u128, |total, record| total + u128::from(record.bytes));
    let mut warnings = Vec::new();
    if let Some(limit) = job.max_count {
        let free = limit.saturating_sub(protected_count);
        if free <= 3 {
            warnings.push(format!("Low retention capacity: {free} places remain ({protected_count} protected, limit {limit})."));
        }
    }
    if let Some(limit) = job.max_bytes {
        let free = u128::from(limit).saturating_sub(protected_bytes);
        if free * 100 <= u128::from(limit) * 20 {
            let percent = if limit == 0 {
                0.0
            } else {
                free as f64 * 100.0 / limit as f64
            };
            warnings.push(format!("Low retention size budget: {free} bytes ({percent:.1}%) remain after protected backups; limit {limit} bytes."));
        }
    }
    let total_bytes = records
        .iter()
        .fold(0u128, |total, record| total + u128::from(record.bytes));
    if job.max_count.is_some_and(|limit| records.len() > limit)
        || job
            .max_bytes
            .is_some_and(|limit| total_bytes > u128::from(limit))
    {
        warnings.push("Current history exceeds retention limits; protected backups and the newest version are kept.".into());
    }
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let path = job.target.join(format!("manifest-{}.json", job.id));
        save(&path, &manifest).unwrap();
        (directory, job, manifest, path)
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
}
