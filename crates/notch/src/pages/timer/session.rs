//! Phase times use Unix milliseconds so sessions survive restarts.

pub use text::{compact_text, next_wake, page_clock_text};

mod storage;
#[cfg(test)]
mod tests;
mod text;

pub const SESSIONS_PER_ROUND: u32 = 4;

pub const EXTENSION_MINUTES: u64 = 5;

pub const LENGTH_STEP_MINUTES: u64 = 5;

const MIN_FOCUS_MINUTES: u64 = 5;

const MAX_FOCUS_MINUTES: u64 = 120;

const MILLISECONDS_PER_MINUTE: u64 = 60_000;

pub const SECONDS_SHOWN_BELOW_MINUTES: u64 = 10;

/// Wake a little after a boundary so the clock has surely crossed it.
const WAKE_SLACK_MILLISECONDS: u64 = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Focus,
    ShortBreak,
    LongBreak,
}

impl Phase {
    fn name(self) -> &'static str {
        match self {
            Phase::Focus => "focus",
            Phase::ShortBreak => "short",
            Phase::LongBreak => "long",
        }
    }

    fn from_name(name: &str) -> Option<Self> {
        match name {
            "focus" => Some(Phase::Focus),
            "short" => Some(Phase::ShortBreak),
            "long" => Some(Phase::LongBreak),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Idle,
    /// `total` is the length of the phase so far, extensions included, in milliseconds.
    Running {
        phase: Phase,
        total: u64,
        ends_at: u64,
    },
    Paused {
        phase: Phase,
        total: u64,
        remaining: u64,
    },
    Finished {
        phase: Phase,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lengths {
    pub focus: u64,
    pub short_break: u64,
    pub long_break: u64,
}

impl Default for Lengths {
    fn default() -> Self {
        Self { focus: 25, short_break: 5, long_break: 15 }
    }
}

impl Lengths {
    pub fn minutes(&self, phase: Phase) -> u64 {
        match phase {
            Phase::Focus => self.focus,
            Phase::ShortBreak => self.short_break,
            Phase::LongBreak => self.long_break,
        }
    }
}

pub struct Timer {
    pub state: State,
    pub completed: u32,
    pub lengths: Lengths,
}

impl Timer {
    pub fn new() -> Self {
        Self { state: State::Idle, completed: 0, lengths: Lengths::default() }
    }

    pub fn session_number(&self) -> u32 {
        self.completed % SESSIONS_PER_ROUND + 1
    }

    pub fn break_after_focus(&self) -> Phase {
        let into_round = self.completed % SESSIONS_PER_ROUND;
        if self.completed > 0 && into_round == 0 { Phase::LongBreak } else { Phase::ShortBreak }
    }

    pub fn next_phase(&self) -> Option<Phase> {
        match self.state {
            State::Finished { phase: Phase::Focus } => Some(self.break_after_focus()),
            State::Finished { .. } => Some(Phase::Focus),
            _ => None,
        }
    }

    pub fn start_focus(&mut self, now: u64) {
        if self.state == State::Idle {
            self.begin(Phase::Focus, now);
        }
    }

    pub fn start_next(&mut self, now: u64) {
        if let Some(phase) = self.next_phase() {
            self.begin(phase, now);
        }
    }

    pub fn skip(&mut self) {
        if matches!(self.state, State::Finished { .. }) {
            self.state = State::Idle;
        }
    }

    pub fn pause(&mut self, now: u64) {
        if let State::Running { phase, total, ends_at } = self.state {
            self.state = State::Paused { phase, total, remaining: ends_at.saturating_sub(now) };
        }
    }

    pub fn resume(&mut self, now: u64) {
        if let State::Paused { phase, total, remaining } = self.state {
            self.state = State::Running { phase, total, ends_at: now + remaining };
        }
    }

    pub fn extend(&mut self) {
        let extra = EXTENSION_MINUTES * MILLISECONDS_PER_MINUTE;
        match &mut self.state {
            State::Running { total, ends_at, .. } => {
                *total += extra;
                *ends_at += extra;
            }
            State::Paused { total, remaining, .. } => {
                *total += extra;
                *remaining += extra;
            }
            _ => {}
        }
    }

    /// Abandons a running or paused phase; an unfinished focus session does not count.
    pub fn stop(&mut self) {
        if matches!(self.state, State::Running { .. } | State::Paused { .. }) {
            self.state = State::Idle;
        }
    }

    pub fn adjust_focus_length(&mut self, steps: i64) {
        if self.state != State::Idle {
            return;
        }
        let minutes = self.lengths.focus as i64 + steps * LENGTH_STEP_MINUTES as i64;
        self.lengths.focus = minutes.clamp(MIN_FOCUS_MINUTES as i64, MAX_FOCUS_MINUTES as i64) as u64;
    }

    pub fn tick(&mut self, now: u64) -> bool {
        let State::Running { phase, ends_at, .. } = self.state else { return false };
        if now < ends_at {
            return false;
        }
        if phase == Phase::Focus {
            self.completed += 1;
        }
        self.state = State::Finished { phase };
        true
    }

    pub fn remaining(&self, now: u64) -> Option<u64> {
        match self.state {
            State::Running { ends_at, .. } => Some(ends_at.saturating_sub(now)),
            State::Paused { remaining, .. } => Some(remaining),
            _ => None,
        }
    }

    pub fn elapsed_fraction(&self, now: u64) -> f32 {
        let (total, remaining) = match self.state {
            State::Running { total, ends_at, .. } => (total, ends_at.saturating_sub(now)),
            State::Paused { total, remaining, .. } => (total, remaining),
            State::Finished { .. } => return 1.0,
            State::Idle => return 0.0,
        };
        if total == 0 { 1.0 } else { (1.0 - remaining as f32 / total as f32).clamp(0.0, 1.0) }
    }

    pub fn phase(&self) -> Option<Phase> {
        match self.state {
            State::Running { phase, .. } | State::Paused { phase, .. } | State::Finished { phase } => Some(phase),
            State::Idle => None,
        }
    }

    fn begin(&mut self, phase: Phase, now: u64) {
        let total = self.lengths.minutes(phase) * MILLISECONDS_PER_MINUTE;
        self.state = State::Running { phase, total, ends_at: now + total };
        if phase == Phase::Focus && self.completed >= SESSIONS_PER_ROUND {
            self.completed = 0;
        }
    }

    // Expired sessions restore as finished.
}
