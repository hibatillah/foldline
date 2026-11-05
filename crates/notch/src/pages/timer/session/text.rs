//! The words the timer shows and when it next needs to wake.

use super::{MILLISECONDS_PER_MINUTE, SECONDS_SHOWN_BELOW_MINUTES, WAKE_SLACK_MILLISECONDS};

pub(super) const MILLISECONDS_PER_SECOND: u64 = 1000;

pub fn compact_text(remaining: u64) -> String {
    if remaining > SECONDS_SHOWN_BELOW_MINUTES * MILLISECONDS_PER_MINUTE {
        format!("{}m", remaining / MILLISECONDS_PER_MINUTE)
    } else {
        clock_text(remaining)
    }
}

/// The time left as "m:ss", counting a started second as a whole one so zero only shows when it is over.
pub fn clock_text(remaining: u64) -> String {
    let seconds = remaining.div_ceil(MILLISECONDS_PER_SECOND);
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

/// Milliseconds until the displayed time changes or the phase ends; open views update each second.
pub fn next_wake(remaining: u64, is_open: bool) -> u64 {
    let counts_seconds = is_open || remaining <= SECONDS_SHOWN_BELOW_MINUTES * MILLISECONDS_PER_MINUTE;
    let unit = if counts_seconds { MILLISECONDS_PER_SECOND } else { MILLISECONDS_PER_MINUTE };
    let past_boundary = remaining % unit;
    let until_boundary = if past_boundary == 0 { unit } else { past_boundary };
    until_boundary.min(remaining) + WAKE_SLACK_MILLISECONDS
}
