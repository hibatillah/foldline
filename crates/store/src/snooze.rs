//! The snooze value the notch and the settings window share through one settings key.

use crate::countdown::{ONE_HOUR_SECONDS, TEN_MINUTES_SECONDS, THIRTY_MINUTES_SECONDS};
pub use crate::countdown::{clock_text, now_unix};

/// Settings key holding `""` (not snoozed), `manual`, or the Unix second the snooze ends.
pub const SNOOZED_UNTIL_KEY: &str = "notch.snoozed_until";

/// One of the ways the user can start a snooze.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnoozeChoice {
    TenMinutes,
    ThirtyMinutes,
    OneHour,
    UntilTurnedOff,
}

countdown_state! {
    /// The decoded value of [`SNOOZED_UNTIL_KEY`].
    Snooze
}

impl SnoozeChoice {
    pub const ALL: [SnoozeChoice; 4] =
        [SnoozeChoice::TenMinutes, SnoozeChoice::ThirtyMinutes, SnoozeChoice::OneHour, SnoozeChoice::UntilTurnedOff];

    /// `None` for `UntilTurnedOff`.
    pub fn seconds(self) -> Option<i64> {
        match self {
            SnoozeChoice::TenMinutes => Some(TEN_MINUTES_SECONDS),
            SnoozeChoice::ThirtyMinutes => Some(THIRTY_MINUTES_SECONDS),
            SnoozeChoice::OneHour => Some(ONE_HOUR_SECONDS),
            SnoozeChoice::UntilTurnedOff => None,
        }
    }

    pub fn snooze_from(self, now_unix: i64) -> Snooze {
        match self.seconds() {
            Some(seconds) => Snooze::Until(now_unix + seconds),
            None => Snooze::Manual,
        }
    }
}

impl Snooze {
    pub fn is_active(self) -> bool {
        self != Snooze::Off
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_choice_ends_after_its_duration_or_never() {
        assert_eq!(SnoozeChoice::TenMinutes.snooze_from(1_000), Snooze::Until(1_600));
        assert_eq!(SnoozeChoice::ThirtyMinutes.snooze_from(1_000), Snooze::Until(2_800));
        assert_eq!(SnoozeChoice::OneHour.snooze_from(1_000), Snooze::Until(4_600));
        assert_eq!(SnoozeChoice::UntilTurnedOff.snooze_from(1_000), Snooze::Manual);
    }

    #[test]
    fn the_stored_text_round_trips() {
        for snooze in [Snooze::Off, Snooze::Manual, Snooze::Until(4_600)] {
            assert_eq!(Snooze::parse(Some(&snooze.as_text()), 1_000), snooze);
        }
        assert_eq!(Snooze::Off.as_text(), "");
        assert_eq!(Snooze::Manual.as_text(), "manual");
    }

    #[test]
    fn a_missing_unknown_or_past_value_is_not_a_snooze() {
        for text in [None, Some(""), Some("yesterday"), Some("-5"), Some("999"), Some("1000")] {
            assert_eq!(Snooze::parse(text, 1_000), Snooze::Off, "{text:?}");
        }
    }

    #[test]
    fn only_a_timed_snooze_has_time_left() {
        assert_eq!(Snooze::Until(1_600).remaining_milliseconds(1_000), Some(600_000));
        assert_eq!(Snooze::Until(1_000).remaining_milliseconds(1_000), Some(1));
        assert_eq!(Snooze::Manual.remaining_milliseconds(1_000), None);
        assert_eq!(Snooze::Off.remaining_milliseconds(1_000), None);
    }

    #[test]
    fn a_snooze_is_active_unless_it_is_off() {
        assert!(Snooze::Manual.is_active());
        assert!(Snooze::Until(5).is_active());
        assert!(!Snooze::Off.is_active());
    }

    #[test]
    fn the_snooze_is_stored_under_its_own_key_and_leaves_the_pause_alone() {
        let path = std::env::temp_dir().join(format!("notch-store-snooze-{}", std::process::id())).join("settings.txt");
        let mut settings = crate::Settings::at(Some(path.clone()));
        settings.set_string(crate::pause::PAUSED_UNTIL_KEY, "manual");
        settings.set_string(SNOOZED_UNTIL_KEY, &SnoozeChoice::OneHour.snooze_from(1_000).as_text());

        let reloaded = crate::Settings::at(Some(path.clone()));
        assert_eq!(Snooze::parse(reloaded.get_string(SNOOZED_UNTIL_KEY), 1_000), Snooze::Until(4_600));
        assert_eq!(reloaded.get_string(crate::pause::PAUSED_UNTIL_KEY), Some("manual"));
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }
}
