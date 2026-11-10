//! The words and numbers the page shows for a time.

use super::BADGE_MINUTES;
use super::times::Prayer;
use crate::i18n::tr;
use salah::{DateTime, Local, Timelike, Utc};

pub(super) fn clock_text(time: DateTime<Utc>) -> String {
    let local = time.with_timezone(&Local);
    format!("{:02}:{:02}", local.hour(), local.minute())
}

/// "in 1 h 12 min", "in 24 min"; rounds up so the last minute still reads "in 1 min".
pub(super) fn remaining_text(seconds: i64) -> String {
    let minutes = (seconds.max(0) + 59) / 60;
    match (minutes / 60, minutes % 60) {
        (0, minutes) => tr!("shalat.in_minutes", minutes = minutes.max(1)).into_owned(),
        (hours, 0) => tr!("shalat.in_hours", hours = hours).into_owned(),
        (hours, minutes) => tr!("shalat.in_hours_minutes", hours = hours, minutes = minutes).into_owned(),
    }
}

/// The collapsed mark, "Asr in 24m", while the next prayer is within `BADGE_MINUTES`; `None` otherwise.
pub(super) fn badge_text(prayer: Prayer, seconds: i64) -> Option<String> {
    (seconds > 0 && seconds <= BADGE_MINUTES * 60)
        .then(|| tr!("shalat.badge_text", prayer = prayer.name(), minutes = (seconds + 59) / 60).into_owned())
}

/// The collapsed mark when several share the pill, "Asr 24m".
pub(super) fn short_badge_text(prayer: Prayer, seconds: i64) -> Option<String> {
    (seconds > 0 && seconds <= BADGE_MINUTES * 60)
        .then(|| tr!("shalat.badge_short", prayer = prayer.name(), minutes = (seconds + 59) / 60).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_prayer_texts_read_in_indonesian() {
        store::language::with_override(store::language::Language::Indonesian, || {
            assert_eq!(remaining_text(24 * 60), "24 menit lagi");
            assert_eq!(remaining_text(3600 + 12 * 60), "1 jam 12 mnt lagi");
            assert_eq!(badge_text(Prayer::Asr, 24 * 60).as_deref(), Some("Ashar 24m lagi"));
            assert_eq!(short_badge_text(Prayer::Asr, 24 * 60).as_deref(), Some("Ashar 24m"));
        });
    }
}
