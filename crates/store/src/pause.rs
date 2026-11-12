//! The pause value the notch and the settings window share through one settings key.

/// Settings key holding `""` (not paused), `manual`, or the Unix second the pause ends.
pub const PAUSED_UNTIL_KEY: &str = "notch.paused_until";

/// One of the ways the user can start a pause.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauseChoice {
    TenMinutes,
    ThirtyMinutes,
    OneHour,
    UntilResumed,
}

/// The decoded value of [`PAUSED_UNTIL_KEY`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pause {
    Off,
    Until(i64),
    Manual,
}

impl PauseChoice {
    pub const ALL: [PauseChoice; 4] =
        [PauseChoice::TenMinutes, PauseChoice::ThirtyMinutes, PauseChoice::OneHour, PauseChoice::UntilResumed];

    /// `None` for `UntilResumed`.
    pub fn seconds(self) -> Option<i64> {
        match self {
            PauseChoice::TenMinutes => Some(600),
            PauseChoice::ThirtyMinutes => Some(1800),
            PauseChoice::OneHour => Some(3600),
            PauseChoice::UntilResumed => None,
        }
    }

    pub fn pause_from(self, now_unix: i64) -> Pause {
        match self.seconds() {
            Some(seconds) => Pause::Until(now_unix + seconds),
            None => Pause::Manual,
        }
    }
}

impl Pause {
    /// Unknown text and a time not after `now_unix` read as `Off`.
    pub fn parse(text: Option<&str>, now_unix: i64) -> Pause {
        match text.map(str::trim) {
            Some("manual") => Pause::Manual,
            Some(value) => match value.parse::<i64>() {
                Ok(end_unix) if end_unix > now_unix => Pause::Until(end_unix),
                _ => Pause::Off,
            },
            None => Pause::Off,
        }
    }

    pub fn as_text(self) -> String {
        match self {
            Pause::Off => String::new(),
            Pause::Manual => "manual".to_owned(),
            Pause::Until(end_unix) => end_unix.to_string(),
        }
    }

    pub fn is_paused(self) -> bool {
        self != Pause::Off
    }

    /// Milliseconds until a timed pause ends, at least 1; `None` for `Off` and `Manual`.
    pub fn remaining_milliseconds(self, now_unix: i64) -> Option<u32> {
        match self {
            Pause::Until(end_unix) => {
                let milliseconds = (end_unix - now_unix).saturating_mul(1000).max(1);
                Some(u32::try_from(milliseconds).unwrap_or(u32::MAX))
            }
            Pause::Off | Pause::Manual => None,
        }
    }
}

/// "14:30" for `unix` seen at `utc_offset_seconds` east of UTC.
pub fn clock_text(unix: i64, utc_offset_seconds: i32) -> String {
    let local_seconds = (unix + i64::from(utc_offset_seconds)).rem_euclid(86_400);
    format!("{:02}:{:02}", local_seconds / 3600, local_seconds % 3600 / 60)
}

pub fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_choice_ends_after_its_duration_or_never() {
        assert_eq!(PauseChoice::TenMinutes.pause_from(1_000), Pause::Until(1_600));
        assert_eq!(PauseChoice::ThirtyMinutes.pause_from(1_000), Pause::Until(2_800));
        assert_eq!(PauseChoice::OneHour.pause_from(1_000), Pause::Until(4_600));
        assert_eq!(PauseChoice::UntilResumed.pause_from(1_000), Pause::Manual);
    }

    #[test]
    fn the_stored_text_round_trips() {
        for pause in [Pause::Off, Pause::Manual, Pause::Until(4_600)] {
            assert_eq!(Pause::parse(Some(&pause.as_text()), 1_000), pause);
        }
        assert_eq!(Pause::Off.as_text(), "");
        assert_eq!(Pause::Manual.as_text(), "manual");
    }

    #[test]
    fn a_missing_unknown_or_past_value_is_not_a_pause() {
        for text in [None, Some(""), Some("yesterday"), Some("-5"), Some("999"), Some("1000")] {
            assert_eq!(Pause::parse(text, 1_000), Pause::Off, "{text:?}");
        }
    }

    #[test]
    fn only_a_timed_pause_has_time_left() {
        assert_eq!(Pause::Until(1_600).remaining_milliseconds(1_000), Some(600_000));
        assert_eq!(Pause::Until(1_000).remaining_milliseconds(1_000), Some(1));
        assert_eq!(Pause::Manual.remaining_milliseconds(1_000), None);
        assert_eq!(Pause::Off.remaining_milliseconds(1_000), None);
    }

    #[test]
    fn the_clock_text_is_local_hours_and_minutes() {
        assert_eq!(clock_text(52_200, 0), "14:30");
        assert_eq!(clock_text(52_200, 7 * 3600), "21:30");
        assert_eq!(clock_text(52_200, 9 * 3600 + 1800), "00:00");
    }
}
