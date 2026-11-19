//! What `pause.rs` and `snooze.rs` share: the stored countdown text, the lengths and the clock helpers.

pub(crate) const TEN_MINUTES_SECONDS: i64 = 600;
pub(crate) const THIRTY_MINUTES_SECONDS: i64 = 1800;
pub(crate) const ONE_HOUR_SECONDS: i64 = 3600;

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

/// Defines a decoded countdown enum (`Off`, `Until`, `Manual`) with the parse, text and remaining-time methods
/// every countdown key shares; the caller adds its own name for "is on" and its choice list.
macro_rules! countdown_state {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum $name {
            Off,
            Until(i64),
            Manual,
        }

        impl $name {
            /// Unknown text and a time not after `now_unix` read as `Off`.
            pub fn parse(text: Option<&str>, now_unix: i64) -> $name {
                match text.map(str::trim) {
                    Some("manual") => $name::Manual,
                    Some(value) => match value.parse::<i64>() {
                        Ok(end_unix) if end_unix > now_unix => $name::Until(end_unix),
                        _ => $name::Off,
                    },
                    None => $name::Off,
                }
            }

            pub fn as_text(self) -> String {
                match self {
                    $name::Off => String::new(),
                    $name::Manual => "manual".to_owned(),
                    $name::Until(end_unix) => end_unix.to_string(),
                }
            }

            /// Milliseconds until a timed value ends, at least 1; `None` for `Off` and `Manual`.
            pub fn remaining_milliseconds(self, now_unix: i64) -> Option<u32> {
                match self {
                    $name::Until(end_unix) => {
                        let milliseconds = (end_unix - now_unix).saturating_mul(1000).max(1);
                        Some(u32::try_from(milliseconds).unwrap_or(u32::MAX))
                    }
                    $name::Off | $name::Manual => None,
                }
            }
        }
    };
}
