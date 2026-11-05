//! The calendar's events, rules and zones.

pub(super) const WEEKDAY_CODES: [&str; 7] = ["MO", "TU", "WE", "TH", "FR", "SA", "SU"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zone {
    Floating,
    Utc,
    Offset(i64),
}

/// Turns UTC into local time; production asks Windows, tests use a fixed offset.
pub trait Clock {
    fn utc_to_local(&self, utc: i64) -> i64;
}

#[cfg(test)]
pub struct FixedClock(pub i64);

#[cfg(test)]
impl Clock for FixedClock {
    fn utc_to_local(&self, utc: i64) -> i64 {
        utc + self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    pub frequency: Frequency,
    pub interval: i64,
    pub count: Option<i64>,
    /// The last moment a repeat may start, as wall seconds in the event's zone.
    pub until: Option<i64>,
    /// Weekdays (Monday = 0) with an optional position in the month: `(2, 1)` is the second Tuesday.
    pub by_day: Vec<(i32, u32)>,
    pub by_month_day: Vec<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    pub uid: String,
    pub summary: String,
    pub location: String,
    pub description: String,
    pub url: String,
    /// Start and end as wall seconds in `zone`; the end of an all-day event is the day after its last day.
    pub start: i64,
    pub end: i64,
    pub zone: Zone,
    pub is_all_day: bool,
    pub rule: Option<Rule>,
    /// Starts (wall seconds) of repeats that were removed.
    pub exceptions: Vec<i64>,
    /// For an event that replaces one repeat of another: the start (wall seconds) it replaces.
    pub replaces: Option<i64>,
    pub is_cancelled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Occurrence {
    pub start: i64,
    pub end: i64,
    pub is_all_day: bool,
    pub summary: String,
    pub location: String,
    pub description: String,
    pub url: String,
}
