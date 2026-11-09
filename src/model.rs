use chrono::{DateTime, Datelike, Duration, Local, NaiveTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

pub const MAX_JOB_NAME_LENGTH: usize = 50;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Schedule {
    Daily,
    Weekdays,
    Interval,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntervalUnit {
    Seconds,
    #[default]
    Minutes,
    Hours,
}

impl IntervalUnit {
    pub fn maximum(self) -> u32 {
        match self {
            Self::Seconds => 31_536_000,
            Self::Minutes => 525_600,
            Self::Hours => 8_760,
        }
    }

    fn seconds(self) -> i64 {
        match self {
            Self::Seconds => 1,
            Self::Minutes => 60,
            Self::Hours => 3_600,
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Job {
    pub id: Uuid,
    pub name: String,
    pub source: PathBuf,
    pub target: PathBuf,
    pub enabled: bool,
    pub zip: bool,
    pub auto_destination: bool,
    pub check_on_enable: bool,
    pub schedule: Schedule,
    pub hour: u32,
    pub minute: u32,
    pub weekdays: [bool; 7],
    pub interval_value: u32,
    pub interval_unit: IntervalUnit,
    pub rolling: bool,
    pub count_protected: bool,
    pub max_count: Option<usize>,
    pub max_bytes: Option<u64>,
    pub next_run: Option<DateTime<Utc>>,
    pub last_success: Option<DateTime<Utc>>,
    pub status: String,
}

impl Default for Job {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            source: PathBuf::new(),
            target: PathBuf::new(),
            enabled: false,
            zip: false,
            auto_destination: false,
            check_on_enable: false,
            schedule: Schedule::Interval,
            hour: 18,
            minute: 0,
            weekdays: [true; 7],
            interval_value: 60,
            interval_unit: IntervalUnit::Minutes,
            rolling: false,
            count_protected: false,
            max_count: None,
            max_bytes: None,
            next_run: None,
            last_success: None,
            status: "Not checked yet".into(),
        }
    }
}

impl Job {
    /// The name shown in lists and headings; unnamed jobs stay recognizable.
    pub fn display_name(&self) -> &str {
        if self.name.trim().is_empty() {
            "Unnamed job"
        } else {
            &self.name
        }
    }

    pub fn interval_seconds(&self) -> u32 {
        self.interval_value.clamp(1, self.interval_unit.maximum())
            * self.interval_unit.seconds() as u32
    }

    pub fn set_interval_seconds(&mut self, seconds: u32) {
        self.interval_value = seconds.clamp(1, IntervalUnit::Seconds.maximum());
        self.interval_unit = IntervalUnit::Seconds;
    }

    fn auto_destination_path(&self, app_dir: &std::path::Path) -> anyhow::Result<PathBuf> {
        let source = std::fs::canonicalize(&self.source)?;
        let name = &self.name;
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
            self.id.to_string()
        } else {
            format!("{}-{name}", self.id)
        };
        let target = std::fs::canonicalize(app_dir)?.join("backups").join(folder);
        anyhow::ensure!(
            !target.starts_with(&source),
            "Destination must be outside the source folder"
        );
        Ok(target)
    }

    pub fn set_auto_destination(&mut self, app_dir: &std::path::Path) -> anyhow::Result<()> {
        let target = self.auto_destination_path(app_dir)?;
        std::fs::create_dir_all(&target)?;
        self.target = target;
        Ok(())
    }

    pub fn update_auto_destination(
        &mut self,
        previous: &Job,
        app_dir: &std::path::Path,
    ) -> anyhow::Result<()> {
        if self.target != previous.target {
            self.auto_destination = false;
        }
        if !self.auto_destination || !self.source.is_dir() {
            return Ok(());
        }
        if !previous.auto_destination || self.source != previous.source {
            if let Err(error) = self.set_auto_destination(app_dir) {
                self.auto_destination = previous.auto_destination;
                return Err(error);
            }
            return Ok(());
        }
        let backups = std::fs::canonicalize(app_dir)?.join("backups");
        let owned_name = self.target.file_name().is_some_and(|name| {
            let name = name.to_string_lossy();
            name == self.id.to_string() || name.starts_with(&format!("{}-", self.id))
        });
        if !self.target.as_os_str().is_empty()
            && (self.target.parent() != Some(backups.as_path()) || !owned_name)
        {
            return Ok(());
        }
        let target = self.auto_destination_path(app_dir)?;
        if target == self.target {
            return Ok(());
        }
        if self.target.is_dir() {
            anyhow::ensure!(!target.exists(), "Renamed destination already exists");
            std::fs::rename(&self.target, &target)?;
        } else {
            std::fs::create_dir_all(&target)?;
        }
        self.target = target;
        Ok(())
    }

    fn portable_destination(&self) -> Option<PathBuf> {
        if !self.auto_destination {
            return None;
        }
        let name = self.target.file_name()?;
        let text = name.to_string_lossy();
        let owned = text == self.id.to_string() || text.starts_with(&format!("{}-", self.id));
        let parent = self.target.parent()?;
        if owned
            && (parent == std::path::Path::new("backups")
                || (self.target.is_absolute()
                    && parent.file_name() == Some(std::ffi::OsStr::new("backups"))))
        {
            Some(PathBuf::from("backups").join(name))
        } else {
            None
        }
    }

    /// Weekday schedules need at least one selected day to ever run.
    pub fn has_runnable_schedule(&self) -> bool {
        self.schedule != Schedule::Weekdays || self.weekdays.contains(&true)
    }

    pub fn reschedule(&mut self, now: DateTime<Utc>) {
        self.next_run = self.enabled.then(|| self.next_after(now));
    }

    pub fn countdown(&self, now: DateTime<Utc>) -> Option<String> {
        if !self.enabled {
            return None;
        }
        self.next_run.map(|next| {
            let millis = (next - now).num_milliseconds().max(0);
            let seconds = (millis + 999) / 1000;
            let days = seconds / 86400;
            let time = format!(
                "{:02}:{:02}:{:02}",
                seconds / 3600 % 24,
                seconds / 60 % 60,
                seconds % 60
            );
            if days > 0 {
                format!("{days}d {time}")
            } else {
                time
            }
        })
    }

    pub fn take_due(&mut self, now: DateTime<Utc>) -> bool {
        if !self.enabled {
            return false;
        }
        let due = self.next_run.is_some_and(|t| t <= now);
        if self.next_run.is_none() || due {
            self.next_run = Some(self.next_after(now));
        }
        due
    }

    pub fn next_after(&self, now: DateTime<Utc>) -> DateTime<Utc> {
        self.next_after_in(now, &Local)
    }

    fn next_after_in<T: TimeZone>(&self, now: DateTime<Utc>, zone: &T) -> DateTime<Utc> {
        if self.schedule == Schedule::Interval {
            return now + Duration::seconds(i64::from(self.interval_seconds()));
        }
        let date = now.with_timezone(zone).date_naive();
        for day in 0..9 {
            let d = date + Duration::days(day);
            if self.schedule == Schedule::Weekdays
                && !self.weekdays[d.weekday().num_days_from_monday() as usize]
            {
                continue;
            }
            let naive = d.and_time(
                NaiveTime::from_hms_opt(self.hour.min(23), self.minute.min(59), 0).unwrap(),
            );
            // A missing DST time is advanced to the first valid local minute.
            for offset in 0..181 {
                if let Some(t) = zone
                    .from_local_datetime(&(naive + Duration::minutes(offset)))
                    .earliest()
                {
                    let t = t.with_timezone(&Utc);
                    if t > now {
                        return t;
                    }
                    break;
                }
            }
        }
        now + Duration::days(1)
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

impl Theme {
    #[cfg(test)]
    pub const ALL: [Self; 3] = [Self::System, Self::Light, Self::Dark];
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct JobDefaults {
    pub auto_destination: bool,
    pub check_on_enable: bool,
    pub zip: bool,
    pub schedule: Schedule,
    pub hour: u32,
    pub minute: u32,
    pub weekdays: [bool; 7],
    pub interval_value: u32,
    pub interval_unit: IntervalUnit,
    pub rolling: bool,
    pub count_protected: bool,
    pub count_enabled: bool,
    pub max_count: usize,
    pub size_enabled: bool,
    pub max_bytes: u64,
}

impl Default for JobDefaults {
    fn default() -> Self {
        Self {
            auto_destination: true,
            check_on_enable: false,
            zip: true,
            schedule: Schedule::Interval,
            hour: 18,
            minute: 0,
            weekdays: [true; 7],
            interval_value: 10,
            interval_unit: IntervalUnit::Minutes,
            rolling: true,
            count_protected: false,
            count_enabled: true,
            max_count: 20,
            size_enabled: false,
            max_bytes: 10 * 1024 * 1024 * 1024,
        }
    }
}

impl JobDefaults {
    pub fn new_job(&self) -> Job {
        Job {
            auto_destination: self.auto_destination,
            check_on_enable: self.check_on_enable,
            zip: self.zip,
            schedule: self.schedule,
            hour: self.hour,
            minute: self.minute,
            weekdays: self.weekdays,
            interval_value: self.interval_value,
            interval_unit: self.interval_unit,
            rolling: self.rolling,
            count_protected: self.count_protected,
            max_count: self.count_enabled.then_some(self.max_count),
            max_bytes: self.size_enabled.then_some(self.max_bytes),
            ..Job::default()
        }
    }

    pub fn update_schedule(&mut self, job: &Job) {
        self.schedule = job.schedule;
        self.hour = job.hour;
        self.minute = job.minute;
        self.weekdays = job.weekdays;
        self.interval_value = job.interval_value;
        self.interval_unit = job.interval_unit;
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub version: u32,
    pub theme: Theme,
    /// Id of the chosen skin; unknown ids fall back to the default skin.
    pub skin: String,
    pub job_defaults: JobDefaults,
    pub sidebar_width: f32,
    pub jobs: Vec<Job>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            theme: Theme::System,
            skin: "retro".into(),
            job_defaults: JobDefaults::default(),
            sidebar_width: 300.0,
            jobs: vec![],
        }
    }
}

impl Config {
    /// Keep runtime paths absolute, but save automatic destinations beside the portable app.
    pub fn for_storage(&self) -> Self {
        let mut saved = self.clone();
        for job in &mut saved.jobs {
            if let Some(relative) = job.portable_destination() {
                job.target = relative;
            }
        }
        saved
    }

    pub fn resolve_destinations(&mut self, app_dir: &std::path::Path) {
        let app_dir = std::fs::canonicalize(app_dir).unwrap_or_else(|_| app_dir.to_path_buf());
        for job in &mut self.jobs {
            if let Some(relative) = job.portable_destination() {
                job.target = app_dir.join(relative);
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Record {
    pub name: String,
    pub hash: String,
    pub bytes: u64,
    pub created: DateTime<Utc>,
    #[serde(default)]
    pub protected: bool,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Manifest {
    pub version: u32,
    pub job_id: Option<Uuid>,
    pub records: Vec<Record>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_names_display_as_unnamed() {
        for (name, shown) in [
            ("", "Unnamed job"),
            ("  ", "Unnamed job"),
            ("Photos", "Photos"),
        ] {
            let job = Job {
                name: name.into(),
                ..Job::default()
            };
            assert_eq!(job.display_name(), shown);
        }
    }

    #[test]
    fn protected_budget_option_defaults_off_and_is_inherited_independently() {
        let mut config: Config = serde_json::from_str(r#"{"jobs":[{}]}"#).unwrap();
        assert!(!config.job_defaults.count_protected);
        assert!(!config.jobs[0].count_protected);
        config.job_defaults.count_protected = true;
        let job = config.job_defaults.new_job();
        assert!(job.count_protected);
        config.jobs.push(job);
        config.job_defaults.count_protected = false;
        let saved: Config = serde_json::from_str(&serde_json::to_string(&config).unwrap()).unwrap();
        assert!(saved.jobs[1].count_protected);
        assert!(!saved.job_defaults.count_protected);
    }

    #[test]
    fn legacy_source_named_destinations_migrate_to_job_names_without_losing_backups() {
        let root = tempfile::tempdir().unwrap();
        let app = root.path().join("app");
        let source = root.path().join("source");
        std::fs::create_dir(&app).unwrap();
        std::fs::create_dir(&source).unwrap();
        let mut job = Job {
            source,
            name: "Project".into(),
            ..JobDefaults::default().new_job()
        };
        job.target = std::fs::canonicalize(&app)
            .unwrap()
            .join("backups")
            .join(format!("{}-source", job.id));
        std::fs::create_dir_all(&job.target).unwrap();
        std::fs::write(job.target.join("backup"), "keep").unwrap();
        let mut value = serde_json::to_value(&job).unwrap();
        value["destination_name"] = serde_json::json!("SourceFolder");
        let mut job: Job = serde_json::from_value(value).unwrap();
        let previous = job.clone();
        job.update_auto_destination(&previous, &app).unwrap();
        assert_eq!(
            job.target.file_name().unwrap(),
            format!("{}-Project", job.id).as_str()
        );
        assert_eq!(
            std::fs::read_to_string(job.target.join("backup")).unwrap(),
            "keep"
        );
        assert!(!previous.target.exists());
    }

    #[test]
    fn auto_updates_can_be_reenabled_after_manual_destination_override() {
        let root = tempfile::tempdir().unwrap();
        let app = root.path().join("app");
        let source = root.path().join("source");
        let manual = root.path().join("manual");
        for path in [&app, &source, &manual] {
            std::fs::create_dir(path).unwrap();
        }
        std::fs::write(manual.join("keep"), "manual data").unwrap();
        let mut job = Job {
            source,
            target: manual.clone(),
            auto_destination: false,
            ..Job::default()
        };
        let previous = job.clone();
        job.auto_destination = true;
        job.update_auto_destination(&previous, &app).unwrap();
        assert!(job.auto_destination);
        assert_ne!(job.target, manual);
        assert_eq!(
            std::fs::read_to_string(manual.join("keep")).unwrap(),
            "manual data"
        );
        let previous = job.clone();
        job.target = manual.clone();
        job.update_auto_destination(&previous, &app).unwrap();
        assert!(!job.auto_destination);
        let previous = job.clone();
        job.auto_destination = true;
        job.update_auto_destination(&previous, &app).unwrap();
        assert!(job.auto_destination);
        let previous = job.clone();
        job.name = "Renamed again".into();
        job.update_auto_destination(&previous, &app).unwrap();
        assert_eq!(
            job.target.file_name().unwrap(),
            format!("{}-Renamed again", job.id).as_str()
        );
    }

    #[test]
    fn job_named_destination_follows_renames_and_preserves_contents() {
        let root = tempfile::tempdir().unwrap();
        let app = root.path().join("app");
        let source = root.path().join("source");
        std::fs::create_dir(&app).unwrap();
        std::fs::create_dir(&source).unwrap();
        let mut job = Job {
            name: "Project".into(),
            source,
            ..JobDefaults::default().new_job()
        };
        job.set_auto_destination(&app).unwrap();
        assert_eq!(
            job.target.file_name().unwrap(),
            format!("{}-Project", job.id).as_str()
        );
        std::fs::write(job.target.join("saved-backup"), "keep").unwrap();
        let previous = job.clone();
        job.name = "Renamed".into();
        job.update_auto_destination(&previous, &app).unwrap();
        assert!(!previous.target.exists());
        assert_eq!(
            std::fs::read_to_string(job.target.join("saved-backup")).unwrap(),
            "keep"
        );
        assert_eq!(
            job.target.file_name().unwrap(),
            format!("{}-Renamed", job.id).as_str()
        );
        let saved: Job = serde_json::from_str(&serde_json::to_string(&job).unwrap()).unwrap();
        assert_eq!(saved.target, job.target);
        let previous = job.clone();
        job.name.clear();
        job.update_auto_destination(&previous, &app).unwrap();
        assert_eq!(job.target.file_name().unwrap(), job.id.to_string().as_str());
        assert_eq!(
            std::fs::read_to_string(job.target.join("saved-backup")).unwrap(),
            "keep"
        );
        let previous = job.clone();
        job.target = root.path().join("manual");
        job.name = "Another name".into();
        job.update_auto_destination(&previous, &app).unwrap();
        assert_eq!(job.target, root.path().join("manual"));
        assert!(!job.auto_destination);
        let previous = job.clone();
        job.name = "Still manual".into();
        job.update_auto_destination(&previous, &app).unwrap();
        assert_eq!(job.target, previous.target);
    }

    #[test]
    fn destination_rename_refuses_collisions_and_sanitizes_names() {
        let root = tempfile::tempdir().unwrap();
        let app = root.path().join("app");
        let source = root.path().join("source");
        std::fs::create_dir(&app).unwrap();
        std::fs::create_dir(&source).unwrap();
        let mut job = Job {
            name: "Old".into(),
            source,
            ..JobDefaults::default().new_job()
        };
        job.set_auto_destination(&app).unwrap();
        let previous = job.clone();
        job.name = "a/b:*?".into();
        let target = job.auto_destination_path(&app).unwrap();
        assert_eq!(
            target.file_name().unwrap(),
            format!("{}-a_b___", job.id).as_str()
        );
        std::fs::create_dir(&target).unwrap();
        assert!(job.update_auto_destination(&previous, &app).is_err());
        assert_eq!(job.target, previous.target);
        assert!(previous.target.is_dir());
    }

    #[test]
    fn mixed_interval_duration_preserves_seconds_and_enforces_bounds() {
        let mut job = JobDefaults::default().new_job();
        assert_eq!(job.interval_seconds(), 600);
        job.set_interval_seconds(3600 + 2 * 60 + 3);
        assert_eq!(job.interval_seconds(), 3723);
        let now = Utc::now();
        assert_eq!(job.next_after(now), now + Duration::seconds(3723));
        let saved: Job = serde_json::from_str(&serde_json::to_string(&job).unwrap()).unwrap();
        assert_eq!(saved.interval_seconds(), 3723);
        job.set_interval_seconds(0);
        assert_eq!(job.interval_seconds(), 1);
        job.set_interval_seconds(u32::MAX);
        assert_eq!(job.interval_seconds(), 31_536_000);
    }

    #[test]
    fn new_jobs_use_saved_defaults_without_changing_existing_jobs() {
        let mut config: Config = serde_json::from_str(r#"{"version":1,"jobs":[{}]}"#).unwrap();
        let existing = config.jobs[0].clone();
        let first = config.job_defaults.new_job();
        assert!(first.auto_destination && first.zip && first.rolling);
        assert_eq!(first.schedule, Schedule::Interval);
        assert_eq!(first.interval_value, 10);
        assert_eq!(first.interval_unit, IntervalUnit::Minutes);
        assert_eq!(first.max_count, Some(20));
        assert_eq!(first.max_bytes, None);
        assert!(!first.enabled);
        config.job_defaults.zip = false;
        config.job_defaults.interval_value = 25;
        config.job_defaults.count_enabled = false;
        config.job_defaults.max_count = 230;
        config.job_defaults.max_bytes = 20 * 1024 * 1024 * 1024;
        let saved: Config = serde_json::from_str(&serde_json::to_string(&config).unwrap()).unwrap();
        assert!(saved == config);
        let second = saved.job_defaults.new_job();
        assert!(!second.zip);
        assert_eq!(second.interval_value, 25);
        assert_eq!(second.max_count, None);
        assert_ne!(first.id, second.id);
        assert!(saved.jobs[0] == existing);
        assert!(!existing.zip && !existing.rolling && !existing.auto_destination);
    }

    #[test]
    fn auto_destination_uses_job_uuid_and_job_name_and_allows_override() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("My source");
        let app = root.path().join("app");
        std::fs::create_dir(&source).unwrap();
        std::fs::create_dir(&app).unwrap();
        let mut job = Job {
            source: source.clone(),
            ..JobDefaults::default().new_job()
        };
        job.set_auto_destination(&app).unwrap();
        let expected = std::fs::canonicalize(&app)
            .unwrap()
            .join("backups")
            .join(job.id.to_string());
        assert_eq!(job.target, expected);
        assert!(expected.is_dir());
        let previous = job.clone();
        job.name = "Renamed job".into();
        job.update_auto_destination(&previous, &app).unwrap();
        assert_ne!(job.target, previous.target);
        job.target = root.path().join("manual");
        let saved: Job = serde_json::from_str(&serde_json::to_string(&job).unwrap()).unwrap();
        assert_eq!(saved.target, job.target);
        job.source = app.clone();
        assert!(job.set_auto_destination(&app).is_err());
    }

    #[test]
    fn default_job_is_disabled_interval() {
        let job = Job::default();
        assert!(!job.enabled);
        assert_eq!(job.schedule, Schedule::Interval);
        assert_eq!(job.interval_value, 60);
        assert!(job.next_run.is_none());
    }

    #[test]
    fn activation_starts_a_fresh_interval() {
        let now = Utc::now();
        let mut job = Job {
            enabled: true,
            ..Job::default()
        };
        job.reschedule(now);
        assert_eq!(job.next_run, Some(now + Duration::minutes(60)));
        job.enabled = false;
        job.reschedule(now);
        assert!(job.next_run.is_none());
        assert!(!job.take_due(now + Duration::days(1)));
        job.enabled = true;
        job.reschedule(now + Duration::days(1));
        assert_eq!(
            job.next_run,
            Some(now + Duration::days(1) + Duration::minutes(60))
        );
    }

    #[test]
    fn countdown_uses_due_time_without_mutating_it() {
        let now = Utc::now();
        let mut job = Job {
            enabled: true,
            next_run: Some(now + Duration::days(1) + Duration::seconds(1)),
            ..Job::default()
        };
        assert_eq!(job.countdown(now).as_deref(), Some("1d 00:00:01"));
        assert_eq!(
            job.countdown(now + Duration::milliseconds(500)).as_deref(),
            Some("1d 00:00:01")
        );
        assert_eq!(
            job.countdown(now + Duration::seconds(2)).as_deref(),
            Some("23:59:59")
        );
        assert_eq!(
            job.countdown(now + Duration::days(2)).as_deref(),
            Some("00:00:00")
        );
        assert_eq!(
            job.next_run,
            Some(now + Duration::days(1) + Duration::seconds(1))
        );
        job.enabled = false;
        assert!(job.countdown(now).is_none());
        job.enabled = true;
        job.next_run = None;
        assert!(job.countdown(now).is_none());
    }

    #[test]
    fn interval_is_future() {
        let j = Job {
            schedule: Schedule::Interval,
            interval_value: 5,
            ..Job::default()
        };
        let n = Utc::now();
        assert_eq!(j.next_after(n), n + Duration::minutes(5));
    }

    #[test]
    fn interval_units_schedule_and_roundtrip() {
        let now = Utc::now();
        for (unit, seconds) in [
            (IntervalUnit::Seconds, 5),
            (IntervalUnit::Minutes, 300),
            (IntervalUnit::Hours, 18_000),
        ] {
            let job = Job {
                interval_value: 5,
                interval_unit: unit,
                enabled: true,
                ..Job::default()
            };
            let data = serde_json::to_vec(&job).unwrap();
            let mut saved: Job = serde_json::from_slice(&data).unwrap();
            assert_eq!(saved.interval_unit, unit);
            assert_eq!(saved.interval_value, 5);
            saved.reschedule(now);
            assert_eq!(saved.next_run, Some(now + Duration::seconds(seconds)));
            assert!(!saved.take_due(now + Duration::seconds(seconds) - Duration::milliseconds(1)));
            assert!(saved.take_due(now + Duration::seconds(seconds)));
        }
    }

    #[test]
    fn interval_bounds_and_unit_changes() {
        let now = Utc::now();
        let mut job = Job {
            interval_value: 60,
            ..Job::default()
        };
        job.interval_unit = IntervalUnit::Hours;
        assert_eq!(job.interval_value, 60);
        assert_eq!(job.next_after(now), now + Duration::hours(60));
        for unit in [
            IntervalUnit::Seconds,
            IntervalUnit::Minutes,
            IntervalUnit::Hours,
        ] {
            job.interval_unit = unit;
            job.interval_value = 0;
            assert_eq!(job.next_after(now), now + Duration::seconds(unit.seconds()));
            job.interval_value = u32::MAX;
            assert_eq!(job.next_after(now), now + Duration::days(365));
        }
    }

    #[test]
    fn calendar_is_future() {
        let j = Job {
            schedule: Schedule::Daily,
            ..Job::default()
        };
        let n = Utc::now();
        assert!(j.next_after(n) > n);
    }

    #[test]
    fn weekday_selection() {
        let j = Job {
            schedule: Schedule::Weekdays,
            weekdays: [true, false, false, false, false, false, false],
            ..Job::default()
        };
        let t = j.next_after(Utc::now());
        assert_eq!(t.with_timezone(&Local).weekday(), chrono::Weekday::Mon);
    }

    #[test]
    fn overdue_is_coalesced_and_disabled_does_not_run() {
        let now = Utc::now();
        let mut j = Job {
            enabled: true,
            next_run: Some(now - Duration::days(10)),
            ..Job::default()
        };
        assert!(j.take_due(now));
        assert!(!j.take_due(now));
        assert!(j.next_run.unwrap() > now);
        j.enabled = false;
        j.next_run = Some(now - Duration::days(1));
        assert!(!j.take_due(now));
    }

    #[test]
    fn new_schedule_is_planned_without_immediate_run() {
        let mut j = Job {
            enabled: true,
            ..Job::default()
        };
        let now = Utc::now();
        assert!(!j.take_due(now));
        assert!(j.next_run.unwrap() > now);
    }

    #[test]
    fn theme_keeps_its_saved_names() {
        let config = Config {
            theme: Theme::Dark,
            ..Config::default()
        };
        let value = serde_json::to_value(&config).unwrap();
        assert_eq!(value["theme"], "Dark");
        let saved: Config = serde_json::from_value(value).unwrap();
        assert_eq!(saved.theme, Theme::Dark);
    }

    #[test]
    fn modern_skin_and_brightness_survive_config_roundtrip() {
        for theme in Theme::ALL {
            let config = Config {
                skin: "modern".into(),
                theme,
                ..Config::default()
            };
            let saved: Config =
                serde_json::from_str(&serde_json::to_string(&config).unwrap()).unwrap();
            assert_eq!(saved.skin, "modern");
            assert_eq!(saved.theme, theme);
        }
    }

    #[test]
    fn configs_without_a_skin_load_the_default_skin() {
        let saved: Config =
            serde_json::from_str(r#"{"version":1,"theme":"Light","jobs":[]}"#).unwrap();
        assert_eq!(saved.skin, "retro");
        assert_eq!(saved.sidebar_width, 300.0);
        assert_eq!(saved.theme, Theme::Light);
    }

    #[test]
    fn weekday_schedule_needs_a_selected_day() {
        let mut job = Job {
            schedule: Schedule::Weekdays,
            weekdays: [false; 7],
            ..Job::default()
        };
        assert!(!job.has_runnable_schedule());
        job.weekdays[6] = true;
        assert!(job.has_runnable_schedule());
        job.weekdays = [false; 7];
        job.schedule = Schedule::Daily;
        assert!(job.has_runnable_schedule());
    }

    #[test]
    fn dst_missing_hour_is_advanced_and_repeated_hour_runs_once() {
        let zone = chrono_tz::Europe::Berlin;
        let j = Job {
            schedule: Schedule::Daily,
            hour: 2,
            minute: 30,
            ..Job::default()
        };
        let before = Utc.with_ymd_and_hms(2026, 3, 29, 0, 0, 0).unwrap();
        assert_eq!(
            j.next_after_in(before, &zone),
            Utc.with_ymd_and_hms(2026, 3, 29, 1, 0, 0).unwrap()
        );
        let before = Utc.with_ymd_and_hms(2026, 10, 25, 0, 0, 0).unwrap();
        let first = j.next_after_in(before, &zone);
        assert_eq!(first, Utc.with_ymd_and_hms(2026, 10, 25, 0, 30, 0).unwrap());
        assert_eq!(
            j.next_after_in(first, &zone),
            Utc.with_ymd_and_hms(2026, 10, 26, 1, 30, 0).unwrap()
        );
    }
}
