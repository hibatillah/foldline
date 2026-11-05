//! The words and numbers the page shows for a time.

use super::BADGE_MINUTES;
use super::times::Prayer;
use salah::{DateTime, Local, Timelike, Utc};

pub(super) fn clock_text(time: DateTime<Utc>) -> String {
    let local = time.with_timezone(&Local);
    format!("{:02}:{:02}", local.hour(), local.minute())
}

/// "in 1 h 12 min", "in 24 min"; rounds up so the last minute still reads "in 1 min".
pub(super) fn remaining_text(seconds: i64) -> String {
    let minutes = (seconds.max(0) + 59) / 60;
    match (minutes / 60, minutes % 60) {
        (0, minutes) => format!("in {} min", minutes.max(1)),
        (hours, 0) => format!("in {hours} h"),
        (hours, minutes) => format!("in {hours} h {minutes} min"),
    }
}

/// The collapsed mark, "Asr in 24m", while the next prayer is within `BADGE_MINUTES`; `None` otherwise.
pub(super) fn badge_text(prayer: Prayer, seconds: i64) -> Option<String> {
    (seconds > 0 && seconds <= BADGE_MINUTES * 60).then(|| format!("{} in {}m", prayer.name(), (seconds + 59) / 60))
}
