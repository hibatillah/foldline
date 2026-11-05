//! Decides when scroll input switches a page: one page per wheel notch or gesture, never a run through the list.

use std::time::Instant;

use super::motion::{GESTURE_QUIET_TIME, PAGE_SWITCH_COOLDOWN, SWIPE_SWITCH_THRESHOLD};

#[derive(Default)]
pub struct ScrollGate {
    accumulated: f32,
    last_smooth_event: Option<Instant>,
    last_switch: Option<Instant>,
    /// The current gesture already switched a page and waits for its end.
    is_spent: bool,
}

impl ScrollGate {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Whether a touchpad or precision-wheel gesture is still going, so a large delta in it is not a wheel notch.
    pub fn is_in_gesture(&self, now: Instant) -> bool {
        self.last_smooth_event.is_some_and(|last| now.saturating_duration_since(last) < GESTURE_QUIET_TIME)
    }

    fn is_cooling_down(&self, now: Instant) -> bool {
        self.last_switch.is_some_and(|last| now.saturating_duration_since(last) < PAGE_SWITCH_COOLDOWN)
    }

    /// The page step (`1` next, `-1` previous) this scroll event causes, if any.
    ///
    /// A wheel notch (`is_smooth` false) switches one page unless the cooldown runs. Smooth input accumulates, switches
    /// once when the sum crosses the threshold, then is ignored until a quiet gap ends the gesture.
    pub fn feed(&mut self, now: Instant, amount: i32, is_smooth: bool) -> Option<i32> {
        if amount == 0 {
            return None;
        }
        if !is_smooth {
            if self.is_cooling_down(now) {
                return None;
            }
            self.last_switch = Some(now);
            return Some(amount.signum());
        }
        if !self.is_in_gesture(now) {
            self.accumulated = 0.0;
            self.is_spent = false;
        }
        self.last_smooth_event = Some(now);
        if self.is_spent || self.is_cooling_down(now) {
            self.accumulated = 0.0;
            return None;
        }
        self.accumulated += amount as f32;
        if self.accumulated.abs() < SWIPE_SWITCH_THRESHOLD {
            return None;
        }
        let step = self.accumulated.signum() as i32;
        self.accumulated = 0.0;
        self.is_spent = true;
        self.last_switch = Some(now);
        Some(step)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn after(start: Instant, milliseconds: u64) -> Instant {
        start + Duration::from_millis(milliseconds)
    }

    #[test]
    fn a_wheel_notch_moves_one_page_and_the_next_notch_waits_for_the_cooldown() {
        let (mut gate, start) = (ScrollGate::default(), Instant::now());
        assert_eq!(gate.feed(start, 120, false), Some(1));
        assert_eq!(gate.feed(after(start, 100), 120, false), None);
        assert_eq!(gate.feed(after(start, 299), -120, false), None);
        assert_eq!(gate.feed(after(start, 300), -120, false), Some(-1));
    }

    #[test]
    fn a_touchpad_gesture_accumulates_and_switches_once_when_it_crosses_the_threshold() {
        let (mut gate, start) = (ScrollGate::default(), Instant::now());
        assert_eq!(gate.feed(start, 40, true), None);
        assert_eq!(gate.feed(after(start, 16), 50, true), None);
        assert_eq!(gate.feed(after(start, 32), 40, true), Some(1));
        for tick in 3..40 {
            assert_eq!(gate.feed(after(start, tick * 16), 60, true), None, "tick {tick}");
        }
    }

    #[test]
    fn a_new_gesture_after_the_quiet_gap_and_the_cooldown_can_switch_again() {
        let (mut gate, start) = (ScrollGate::default(), Instant::now());
        assert_eq!(gate.feed(start, 130, true), Some(1));
        assert_eq!(gate.feed(after(start, 200), -130, true), None, "the gap is too short to end the gesture");
        assert_eq!(gate.feed(after(start, 500), -130, true), Some(-1));
    }

    #[test]
    fn a_gesture_started_inside_the_cooldown_is_dropped_until_the_cooldown_ends() {
        let (mut gate, start) = (ScrollGate::default(), Instant::now());
        assert_eq!(gate.feed(start, 130, true), Some(1));
        assert_eq!(gate.feed(after(start, 160), -50, true), None);
        assert_eq!(gate.feed(after(start, 200), -130, true), None);
        assert_eq!(gate.feed(after(start, 320), -130, true), Some(-1), "the cooldown ended while the gesture went on");
        assert_eq!(gate.feed(after(start, 336), -130, true), None, "and it is spent again");
    }

    #[test]
    fn opposite_deltas_cancel_out_before_the_threshold() {
        let (mut gate, start) = (ScrollGate::default(), Instant::now());
        assert_eq!(gate.feed(start, 100, true), None);
        assert_eq!(gate.feed(after(start, 16), -100, true), None);
        assert_eq!(gate.feed(after(start, 32), 100, true), None);
    }

    #[test]
    fn a_gesture_in_progress_is_known_until_the_quiet_time_passes() {
        let (mut gate, start) = (ScrollGate::default(), Instant::now());
        assert!(!gate.is_in_gesture(start));
        gate.feed(start, 10, true);
        assert!(gate.is_in_gesture(after(start, 149)));
        assert!(!gate.is_in_gesture(after(start, 150)));
    }
}
