use chrono::{DateTime, Datelike, Duration, Local, NaiveTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

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
    pub fn label(self) -> &'static str {
        match self {
            Self::Seconds => "Seconds",
            Self::Minutes => "Minutes",
            Self::Hours => "Hours",
        }
    }

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
    pub schedule: Schedule,
    pub hour: u32,
    pub minute: u32,
    pub weekdays: [bool; 7],
    pub interval_value: u32,
    pub interval_unit: IntervalUnit,
    pub rolling: bool,
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
            name: "New job".into(),
            source: PathBuf::new(),
            target: PathBuf::new(),
            enabled: false,
            zip: false,
            schedule: Schedule::Interval,
            hour: 18,
            minute: 0,
            weekdays: [true; 7],
            interval_value: 60,
            interval_unit: IntervalUnit::Minutes,
            rolling: false,
            max_count: None,
            max_bytes: None,
            next_run: None,
            last_success: None,
            status: "Not checked yet".into(),
        }
    }
}

impl Job {
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
            return now
                + Duration::seconds(
                    i64::from(self.interval_value.clamp(1, self.interval_unit.maximum()))
                        * self.interval_unit.seconds(),
                );
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
pub struct Config {
    pub version: u32,
    pub theme: Theme,
    /// Id of the chosen skin; unknown ids fall back to the default skin.
    pub skin: String,
    pub sidebar_width: f32,
    pub jobs: Vec<Job>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            theme: Theme::System,
            skin: "retro".into(),
            sidebar_width: 290.0,
            jobs: vec![],
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
        assert_eq!(saved.sidebar_width, 290.0);
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
