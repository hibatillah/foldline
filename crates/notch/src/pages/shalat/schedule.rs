//! Today's prayer times, when the page announces one and when it next wakes.

use super::times::Prayer;
use super::{
    BADGE_MINUTES, Config, REFRESH_TIMER, ShalatPage, badge_text, badge_width, clock_text, short_badge_text, times,
};
use crate::i18n::tr;
use crate::{
    gfx::Fonts,
    page::Context,
    ui::{self},
};
use salah::{DateTime, Local, NaiveDate, Timelike, Utc};

pub(super) const MESSAGE_MILLISECONDS: u32 = 9000;
/// A prayer time that passed this recently still gets its message, since the timer fires on a minute boundary.
pub(super) const ANNOUNCE_SECONDS: i64 = 120;
pub(super) const MAX_WAIT_MILLISECONDS: i64 = 12 * 60 * 60 * 1000;

/// Today's times and what comes next, worked out at `now`.
#[derive(Clone, Debug)]
pub(super) struct Schedule {
    pub(super) today: [DateTime<Utc>; 6],
    pub(super) next: (Prayer, DateTime<Utc>),
}

pub(super) fn compute(config: &Config, now: DateTime<Utc>, date: NaiveDate) -> Option<Schedule> {
    let place = config.place.as_ref()?;
    let next = times::next_after(now, date, place, &config.rules)?;
    Some(Schedule { today: times::day_times(date, place, &config.rules)?, next })
}

/// Milliseconds until the page needs to look again; `None` when nothing needs a timer.
pub(super) fn wait_milliseconds(
    seconds_to_next: i64,
    is_shown: bool,
    shows_badge: bool,
    shows_message: bool,
    milliseconds_into_minute: i64,
) -> Option<i64> {
    let to_minute = 60_000 - milliseconds_into_minute + 50;
    let to_next = seconds_to_next.max(0) * 1000 + 1000;
    let to_badge_window = (seconds_to_next - BADGE_MINUTES * 60).max(0) * 1000;
    let wait = if is_shown {
        to_minute
    } else if shows_badge || shows_message {
        if seconds_to_next <= BADGE_MINUTES * 60 { to_minute.min(to_next) } else { to_badge_window.max(1000) }
    } else {
        return None;
    };
    Some(wait.clamp(250, MAX_WAIT_MILLISECONDS))
}

impl ShalatPage {
    /// Recalculates for the present moment and, when a prayer has just started, shows its message.
    pub(super) fn refresh(&mut self, context: &mut Context) {
        let now = Utc::now();
        let date = now.with_timezone(&Local).date_naive();
        self.schedule = compute(&self.config, now, date);
        self.badge = None;
        self.badge_short = None;
        if let Some(schedule) = &self.schedule {
            let seconds = (schedule.next.1 - now).num_seconds();
            if self.config.shows_badge {
                self.set_badge(context.fonts(), schedule.next.0, seconds);
            }
            self.announce_if_due(now, date, context);
        }
        self.arm_timer(context, now);
        context.request_repaint();
    }

    /// Shows the badge as wide as the text it shows now; the host re-lays out when the width changes.
    pub(super) fn set_badge(&mut self, fonts: &Fonts, prayer: Prayer, seconds: i64) {
        let Some(text) = badge_text(prayer, seconds) else { return };
        self.badge_width = badge_width(fonts, &text);
        self.badge = Some(text);
        self.badge_short = short_badge_text(prayer, seconds);
        if let Some(short) = &self.badge_short {
            self.badge_short_width = badge_width(fonts, short);
        }
    }

    pub(super) fn announce_if_due(&mut self, now: DateTime<Utc>, date: NaiveDate, context: &mut Context) {
        let Some(place) = self.config.place.as_ref() else { return };
        let Some((prayer, started)) = times::previous_before(now, date, place, &self.config.rules) else { return };
        let is_fresh = (now - started).num_seconds() <= ANNOUNCE_SECONDS;
        if !self.config.shows_message || !prayer.is_prayer() || !is_fresh || self.last_announced == Some(started) {
            return;
        }
        self.last_announced = Some(started);
        self.alert_title = tr!("shalat.alert_title", prayer = prayer.name()).into_owned();
        self.alert_value = clock_text(started);
        self.alert_width = ui::message_width(context.fonts(), &self.alert_title, Some(&self.alert_value));
        self.alert.show(context, MESSAGE_MILLISECONDS);
    }

    pub(super) fn arm_timer(&self, context: &mut Context, now: DateTime<Utc>) {
        let Some(schedule) = &self.schedule else {
            context.kill_timer(REFRESH_TIMER);
            return;
        };
        let into_minute = i64::from(now.with_timezone(&Local).second()) * 1000;
        let seconds_to_next = (schedule.next.1 - now).num_seconds();
        match wait_milliseconds(
            seconds_to_next,
            self.is_shown,
            self.config.shows_badge,
            self.config.shows_message,
            into_minute,
        ) {
            Some(wait) => context.set_timer(REFRESH_TIMER, wait as u32),
            None => context.kill_timer(REFRESH_TIMER),
        }
    }
}
