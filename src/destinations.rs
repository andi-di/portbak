use crate::model::Job;
use std::path::PathBuf;

pub(crate) fn automatic_path(job: &Job, app_dir: &std::path::Path) -> anyhow::Result<PathBuf> {
    let source = std::fs::canonicalize(&job.source)?;
    let name = &job.name;
    let name: String = name
        .chars()
        .map(|c| {
            if c.is_control() || "<>:\"/\\|?*".contains(c) {
                '_'
            } else {
                c
            }
        })
        .take(100)
        .collect();
    let name = name.trim().trim_end_matches('.');
    let folder = if name.is_empty() {
        job.id.to_string()
    } else {
        format!("{}-{name}", job.id)
    };
    let target = std::fs::canonicalize(app_dir)?.join("backups").join(folder);
    anyhow::ensure!(
        !target.starts_with(&source),
        "Destination must be outside the source folder"
    );
    Ok(target)
}

pub fn create_for(job: &mut Job, app_dir: &std::path::Path) -> anyhow::Result<()> {
    let target = automatic_path(job, app_dir)?;
    std::fs::create_dir_all(&target)?;
    job.target = target;
    Ok(())
}

pub fn update(job: &mut Job, previous: &Job, app_dir: &std::path::Path) -> anyhow::Result<()> {
    if job.target != previous.target {
        job.auto_destination = false;
    }
    if !job.auto_destination || !job.source.is_dir() {
        return Ok(());
    }
    if !previous.auto_destination || job.source != previous.source {
        if let Err(error) = create_for(job, app_dir) {
            job.auto_destination = previous.auto_destination;
            return Err(error);
        }
        return Ok(());
    }
    let backups = std::fs::canonicalize(app_dir)?.join("backups");
    let owned_name = job.target.file_name().is_some_and(|name| {
        let name = name.to_string_lossy();
        name == job.id.to_string() || name.starts_with(&format!("{}-", job.id))
    });
    if !job.target.as_os_str().is_empty()
        && (job.target.parent() != Some(backups.as_path()) || !owned_name)
    {
        return Ok(());
    }
    let target = automatic_path(job, app_dir)?;
    if target == job.target {
        return Ok(());
    }
    if job.target.is_dir() {
        anyhow::ensure!(!target.exists(), "Renamed destination already exists");
        std::fs::rename(&job.target, &target)?;
    } else {
        std::fs::create_dir_all(&target)?;
    }
    job.target = target;
    Ok(())
}

pub(crate) fn portable_path(job: &Job) -> Option<PathBuf> {
    if !job.auto_destination {
        return None;
    }
    let name = job.target.file_name()?;
    let text = name.to_string_lossy();
    let owned = text == job.id.to_string() || text.starts_with(&format!("{}-", job.id));
    let parent = job.target.parent()?;
    if owned
        && (parent == std::path::Path::new("backups")
            || (job.target.is_absolute()
                && parent.file_name() == Some(std::ffi::OsStr::new("backups"))))
    {
        Some(PathBuf::from("backups").join(name))
    } else {
        None
    }
}
