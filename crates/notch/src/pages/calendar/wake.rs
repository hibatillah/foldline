//! When the page wakes: every minute while shown or counting down, otherwise only for the next reminder window.
//! A hidden page keeps fetching too, or an event added later would never raise a reminder.

use super::{
    CalendarPage, FETCH_TIMER, MINUTE_TIMER,
    agenda::REMINDER_LEAD_SECONDS,
    dates::{SECONDS_PER_DAY, SECONDS_PER_MINUTE},
    ics::Occurrence,
    local_time::{local_milliseconds_into_minute, local_now},
};
use crate::page::Context;
use std::time::{Duration, Instant};

const WAKE_MARGIN_MILLISECONDS: i64 = 50;

/// Seconds until the next event's reminder window opens, or until midnight when none is left today.
pub(super) fn seconds_until_wake(occurrences: &[Occurrence], now: i64) -> i64 {
    let midnight = SECONDS_PER_DAY - now.rem_euclid(SECONDS_PER_DAY);
    occurrences
        .iter()
        .filter(|occurrence| !occurrence.is_all_day)
        .map(|occurrence| occurrence.start - REMINDER_LEAD_SECONDS - now)
        .filter(|seconds| *seconds > 0)
        .fold(midnight, i64::min)
}

/// A fetch is due when none ran yet or the last one is at least one interval old.
pub(super) fn is_fetch_due(last_fetch: Option<Instant>, now: Instant, interval_milliseconds: u32) -> bool {
    last_fetch.is_none_or(|last| now.duration_since(last) >= Duration::from_millis(u64::from(interval_milliseconds)))
}

fn milliseconds_to_next_minute() -> i64 {
    SECONDS_PER_MINUTE * 1000 - local_milliseconds_into_minute() + WAKE_MARGIN_MILLISECONDS
}

impl CalendarPage {
    pub(super) fn arm_wake_timer(&self, context: &mut Context) {
        let milliseconds = if self.is_shown || self.reminder.is_some() {
            milliseconds_to_next_minute()
        } else {
            seconds_until_wake(&self.occurrences, local_now()) * 1000 + WAKE_MARGIN_MILLISECONDS
        };
        context.set_timer(MINUTE_TIMER, milliseconds.clamp(1, i64::from(u32::MAX)) as u32);
    }

    /// Downloads now when the last download is older than the interval, e.g. after sleep or a failed attempt.
    pub(super) fn fetch_when_due(&mut self, context: &mut Context) {
        if self.link.is_some() && is_fetch_due(self.last_fetch, Instant::now(), self.fetch_interval_milliseconds) {
            self.start_fetch();
            context.set_timer(FETCH_TIMER, self.fetch_interval_milliseconds);
        }
    }

    pub(super) fn start_fetch(&mut self) {
        if let (Some(link), Some(notifier)) = (&self.link, self.notifier) {
            self.last_fetch = Some(Instant::now());
            super::fetch::start(link.clone(), self.fetched.clone(), notifier);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY_START: i64 = 20_000 * SECONDS_PER_DAY;

    fn event(start_seconds: i64, is_all_day: bool) -> Occurrence {
        Occurrence {
            start: DAY_START + start_seconds,
            end: DAY_START + start_seconds + 3600,
            is_all_day,
            summary: String::new(),
            location: String::new(),
            description: String::new(),
            url: String::new(),
        }
    }

    #[test]
    fn a_hidden_page_sleeps_until_the_next_reminder_window_opens() {
        let day = [event(9 * 3600, false), event(13 * 3600, false)];
        assert_eq!(seconds_until_wake(&day, DAY_START + 8 * 3600), 3600 - 15 * 60);
        assert_eq!(seconds_until_wake(&day, DAY_START + 10 * 3600), 3 * 3600 - 15 * 60);
    }

    #[test]
    fn with_no_reminder_left_the_page_sleeps_until_midnight() {
        let day = [event(9 * 3600, false), event(0, true)];
        assert_eq!(seconds_until_wake(&day, DAY_START + 10 * 3600), 14 * 3600);
        assert_eq!(seconds_until_wake(&[], DAY_START + 23 * 3600 + 59 * 60), 60);
    }

    #[test]
    fn an_event_whose_window_is_already_open_does_not_wake_the_page_again() {
        let day = [event(9 * 3600, false)];
        assert_eq!(seconds_until_wake(&day, DAY_START + 9 * 3600 - 5 * 60), 15 * 3600 + 5 * 60);
    }

    #[test]
    fn a_download_is_due_when_none_ran_or_one_interval_has_passed() {
        let now = Instant::now();
        assert!(is_fetch_due(None, now, 60_000));
        assert!(!is_fetch_due(Some(now), now, 60_000));
        assert!(is_fetch_due(Some(now), now + Duration::from_millis(60_000), 60_000));
        assert!(!is_fetch_due(Some(now), now + Duration::from_millis(59_999), 60_000));
    }
}
