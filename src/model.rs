use chrono::{DateTime, Datelike, Duration, Local, NaiveTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum Schedule {
    Daily,
    Weekdays,
    Interval,
}
#[derive(Clone, Serialize, Deserialize)]
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
    pub interval_minutes: u32,
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
            schedule: Schedule::Daily,
            hour: 18,
            minute: 0,
            weekdays: [true; 7],
            interval_minutes: 60,
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
            return now + Duration::minutes(self.interval_minutes.max(1).into());
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
#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub version: u32,
    pub theme: String,
    pub jobs: Vec<Job>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            theme: "System".into(),
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
    fn interval_is_future() {
        let j = Job {
            schedule: Schedule::Interval,
            interval_minutes: 5,
            ..Job::default()
        };
        let n = Utc::now();
        assert_eq!(j.next_after(n), n + Duration::minutes(5));
    }
    #[test]
    fn calendar_is_future() {
        let j = Job::default();
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
    fn dst_missing_hour_is_advanced_and_repeated_hour_runs_once() {
        let zone = chrono_tz::Europe::Berlin;
        let j = Job {
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
