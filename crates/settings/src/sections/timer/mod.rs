//! The Timer page: how the focus timer looks in the notch, the three session lengths and one round drawn to scale.

mod pictures;

use store::Settings;

use crate::{
    field::stored_whole_number,
    i18n::tr,
    layout::{Group, Item, Page},
    stepper::{self, REGULAR},
};

// The keys `crates/notch/src/pages/timer/session/storage.rs` reads; a test compares them.
const FOCUS_KEY: &str = "timer.focus_minutes";
const SHORT_BREAK_KEY: &str = "timer.short_break_minutes";
const LONG_BREAK_KEY: &str = "timer.long_break_minutes";

const FOCUS_RANGE: std::ops::RangeInclusive<u32> = 5..=120;
const BREAK_RANGE: std::ops::RangeInclusive<u32> = 1..=120;
/// The notch's timer page changes the focus length in steps of 5 too.
const FOCUS_STEP: u32 = 5;
const BREAK_STEP: u32 = 1;
/// Focus sessions in one round; the long break follows the last (`SESSIONS_PER_ROUND` in the notch).
const SESSIONS_PER_ROUND: u32 = 4;

/// The three lengths in minutes, read as the notch reads them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lengths {
    pub focus: u32,
    pub short_break: u32,
    pub long_break: u32,
}

impl Lengths {
    /// A missing, unreadable or zero value is the default; the notch keeps focus within 5 to 120 and breaks as stored.
    pub fn read(settings: &Settings) -> Self {
        Self {
            focus: stored_whole_number(settings, FOCUS_KEY, 25).clamp(*FOCUS_RANGE.start(), *FOCUS_RANGE.end()),
            short_break: stored_whole_number(settings, SHORT_BREAK_KEY, 5),
            long_break: stored_whole_number(settings, LONG_BREAK_KEY, 15),
        }
    }

    /// Four focus sessions, a short break between each two and the long break at the end.
    pub fn round_minutes(self) -> u32 {
        SESSIONS_PER_ROUND * self.focus + (SESSIONS_PER_ROUND - 1) * self.short_break + self.long_break
    }
}

pub fn page() -> Page {
    Page::plain(
        tr!("timer.title"),
        vec![
            Group::titled(tr!("timer.group_looks"), vec![Item::picture_row(pictures::pills)])
                .description(tr!("timer.group_looks_note")),
            Group::titled(
                tr!("timer.group_lengths"),
                vec![
                    Item::new(tr!("timer.focus"), stepper::minutes(FOCUS_KEY, 25, FOCUS_RANGE, FOCUS_STEP, REGULAR))
                        .description(tr!("timer.focus_note")),
                    Item::new(
                        tr!("timer.short_break"),
                        stepper::minutes(SHORT_BREAK_KEY, 5, BREAK_RANGE, BREAK_STEP, REGULAR),
                    )
                    .description(tr!("timer.short_break_note")),
                    Item::new(
                        tr!("timer.long_break"),
                        stepper::minutes(LONG_BREAK_KEY, 15, BREAK_RANGE, BREAK_STEP, REGULAR),
                    )
                    .description(tr!("timer.long_break_note")),
                ],
            )
            .description(tr!("timer.group_lengths_note")),
            Group::titled(tr!("timer.group_round"), vec![Item::picture_row(pictures::round)]).live_description(|| {
                tr!("timer.round_total", minutes = Lengths::read(&Settings::load()).round_minutes()).into()
            }),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings_with(text: &str, name: &str) -> Settings {
        let path = std::env::temp_dir().join(format!("notch-settings-timer-{name}-{}.txt", std::process::id()));
        std::fs::write(&path, text).expect("write settings");
        let settings = Settings::at(Some(path.clone()));
        let _ = std::fs::remove_file(&path);
        settings
    }

    #[test]
    fn the_keys_are_the_ones_the_notch_reads() {
        let source = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../notch/src/pages/timer/session/storage.rs"
        ))
        .expect("notch timer storage source");
        for key in [FOCUS_KEY, SHORT_BREAK_KEY, LONG_BREAK_KEY] {
            assert!(source.contains(&format!("\"{key}\"")), "{key} missing in storage.rs");
        }
    }

    #[test]
    fn a_round_of_the_default_lengths_lasts_130_minutes() {
        assert_eq!(Lengths::read(&settings_with("", "default")).round_minutes(), 130);
    }

    #[test]
    fn a_round_follows_the_stored_lengths() {
        let settings = settings_with(
            "timer.focus_minutes=50\ntimer.short_break_minutes=10\ntimer.long_break_minutes=30\n",
            "stored",
        );
        assert_eq!(Lengths::read(&settings).round_minutes(), 4 * 50 + 3 * 10 + 30);
    }

    #[test]
    fn lengths_are_read_like_the_notch_reads_them() {
        let settings = settings_with(
            "timer.focus_minutes=300\ntimer.short_break_minutes=0\ntimer.long_break_minutes=abc\n",
            "odd",
        );
        assert_eq!(Lengths::read(&settings), Lengths { focus: 120, short_break: 5, long_break: 15 });
    }

    #[test]
    fn the_notch_keeps_focus_in_the_same_range_and_rounds_of_four() {
        let source =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../notch/src/pages/timer/session.rs"))
                .expect("notch timer session source");
        assert!(source.contains(&format!("SESSIONS_PER_ROUND: u32 = {SESSIONS_PER_ROUND};")));
        assert!(source.contains(&format!("MIN_FOCUS_MINUTES: u64 = {};", FOCUS_RANGE.start())));
        assert!(source.contains(&format!("MAX_FOCUS_MINUTES: u64 = {};", FOCUS_RANGE.end())));
        assert!(source.contains(&format!("LENGTH_STEP_MINUTES: u64 = {FOCUS_STEP};")));
    }
}
