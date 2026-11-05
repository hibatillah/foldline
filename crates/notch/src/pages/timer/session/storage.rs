//! Saving and loading the timer's state and lengths.

use super::{Lengths, MAX_FOCUS_MINUTES, MIN_FOCUS_MINUTES};
use crate::host::Settings;
use crate::pages::timer::Phase;
use crate::pages::timer::State;
use crate::pages::timer::Timer;

pub(super) const STATE_KEY: &str = "timer.state";
pub(super) const COMPLETED_KEY: &str = "timer.completed";
pub(super) const FOCUS_MINUTES_KEY: &str = "timer.focus_minutes";
pub(super) const SHORT_BREAK_MINUTES_KEY: &str = "timer.short_break_minutes";
pub(super) const LONG_BREAK_MINUTES_KEY: &str = "timer.long_break_minutes";

pub(super) fn format_state(state: State) -> String {
    match state {
        State::Idle => "idle".to_owned(),
        State::Running { phase, total, ends_at } => format!("running {} {total} {ends_at}", phase.name()),
        State::Paused { phase, total, remaining } => format!("paused {} {total} {remaining}", phase.name()),
        State::Finished { phase } => format!("finished {}", phase.name()),
    }
}

pub(super) fn parse_state(text: &str) -> Option<State> {
    let mut words = text.split_whitespace();
    let kind = words.next()?;
    if kind == "idle" {
        return Some(State::Idle);
    }
    let phase = Phase::from_name(words.next()?)?;
    if kind == "finished" {
        return Some(State::Finished { phase });
    }
    let total: u64 = words.next()?.parse().ok()?;
    let time: u64 = words.next()?.parse().ok()?;
    match kind {
        "running" => Some(State::Running { phase, total, ends_at: time }),
        "paused" => Some(State::Paused { phase, total, remaining: time }),
        _ => None,
    }
}

impl Lengths {
    /// A missing, unreadable or zero value keeps the length from `fallback`.
    pub(super) fn load(settings: &Settings, fallback: Lengths) -> Self {
        let read = |key: &str, fallback: u64| {
            settings
                .get_string(key)
                .and_then(|text| text.parse().ok())
                .filter(|minutes| *minutes > 0)
                .unwrap_or(fallback)
        };
        Self {
            focus: read(FOCUS_MINUTES_KEY, fallback.focus).clamp(MIN_FOCUS_MINUTES, MAX_FOCUS_MINUTES),
            short_break: read(SHORT_BREAK_MINUTES_KEY, fallback.short_break),
            long_break: read(LONG_BREAK_MINUTES_KEY, fallback.long_break),
        }
    }
}

impl Timer {
    /// Restores the timer saved by `save`; a phase that ran out while the notch was closed comes back finished.
    pub fn load(settings: &Settings, now: u64) -> Self {
        let mut timer = Self {
            state: settings.get_string(STATE_KEY).and_then(parse_state).unwrap_or(State::Idle),
            completed: settings.get_string(COMPLETED_KEY).and_then(|text| text.parse().ok()).unwrap_or(0),
            lengths: Lengths::load(settings, Lengths::default()),
        };
        timer.tick(now);
        timer
    }

    /// Re-reads the phase lengths; a running phase keeps the length it started with.
    pub fn reload_lengths(&mut self, settings: &Settings) {
        self.lengths = Lengths::load(settings, self.lengths);
    }

    pub fn save(&self, settings: &mut Settings) {
        settings.set_string(STATE_KEY, &format_state(self.state));
        settings.set_string(COMPLETED_KEY, &self.completed.to_string());
        settings.set_string(FOCUS_MINUTES_KEY, &self.lengths.focus.to_string());
    }
}
