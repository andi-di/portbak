use chrono::{DateTime, Datelike, Duration, Local, NaiveTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

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

    pub(super) fn seconds(self) -> i64 {
        match self {
            Self::Seconds => 1,
            Self::Minutes => 60,
            Self::Hours => 3_600,
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ScheduleSettings {
    pub mode: Schedule,
    pub hour: u32,
    pub minute: u32,
    pub weekdays: [bool; 7],
    pub interval_value: u32,
    pub interval_unit: IntervalUnit,
}

impl Default for ScheduleSettings {
    fn default() -> Self {
        Self {
            mode: Schedule::Interval,
            hour: 18,
            minute: 0,
            weekdays: [true; 7],
            interval_value: 60,
            interval_unit: IntervalUnit::Minutes,
        }
    }
}

impl ScheduleSettings {
    pub fn interval_seconds(&self) -> u32 {
        self.interval_value.clamp(1, self.interval_unit.maximum())
            * self.interval_unit.seconds() as u32
    }

    pub fn set_interval_seconds(&mut self, seconds: u32) {
        self.interval_value = seconds.clamp(1, IntervalUnit::Seconds.maximum());
        self.interval_unit = IntervalUnit::Seconds;
    }

    /// Weekday schedules need at least one selected day to ever run.
    pub fn has_runnable_schedule(&self) -> bool {
        self.mode != Schedule::Weekdays || self.weekdays.contains(&true)
    }

    pub fn next_after(&self, now: DateTime<Utc>) -> DateTime<Utc> {
        self.next_after_in(now, &Local)
    }

    pub(super) fn next_after_in<T: TimeZone>(&self, now: DateTime<Utc>, zone: &T) -> DateTime<Utc> {
        if self.mode == Schedule::Interval {
            return now + Duration::seconds(i64::from(self.interval_seconds()));
        }
        let date = now.with_timezone(zone).date_naive();
        for day in 0..9 {
            let d = date + Duration::days(day);
            if self.mode == Schedule::Weekdays
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
