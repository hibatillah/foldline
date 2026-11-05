//! Every tuning number of the notch's size animation and of page switching input.

use std::time::Duration;

/// Spring (stiffness, damping) of every size change: open, close, page width and height, badge room.
/// Damping ratio is `16.7 / (2 * sqrt(215))` = 0.57, which overshoots a move by about 10 percent (11 percent without the lead-in) and settles in
/// about 1.45 times the time of the earlier (420, 21) spring.
pub const SIZE_RESPONSE: (f32, f32) = (215.0, 16.7);

/// The pull of a size spring grows from nothing over this long when a move starts at rest, so it begins gently.
pub const SIZE_LEAD_IN: f32 = 0.09;

/// Spring (stiffness, damping) of the page row sliding between pages; critically damped so the pages never cross over.
pub const PAGE_SLIDE_RESPONSE: (f32, f32) = (650.0, 51.0);

/// The open pill may shrink to this fraction of its collapsed size when a spring undershoots.
pub const MIN_SIZE_FRACTION: f32 = 0.75;

/// Furthest the open amount (1.0 is fully open) may extrapolate on either side while the spring overshoots.
pub const EXPANSION_OVERSHOOT_LIMIT: (f32, f32) = (-0.25, 1.3);

/// A wheel notch reports this many units; smaller deltas come from precision wheels and touchpads.
pub const WHEEL_NOTCH: i32 = 120;

/// Scroll input is ignored for this long after a page switch, so one scroll moves one page.
pub const PAGE_SWITCH_COOLDOWN: Duration = Duration::from_millis(300);

/// A touchpad or precision-wheel gesture ends when no scroll event arrives for this long.
pub const GESTURE_QUIET_TIME: Duration = Duration::from_millis(150);

/// Accumulated scroll units in one gesture that switch a page.
pub const SWIPE_SWITCH_THRESHOLD: f32 = 120.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_size_spring_is_underdamped_with_a_small_overshoot() {
        let (stiffness, damping) = SIZE_RESPONSE;
        let damping_ratio = damping / (2.0 * stiffness.sqrt());
        assert!((0.565..=0.62).contains(&damping_ratio), "damping ratio {damping_ratio}");
        let overshoot = (-std::f32::consts::PI * damping_ratio / (1.0 - damping_ratio * damping_ratio).sqrt()).exp();
        assert!((0.08..=0.115).contains(&overshoot), "overshoot {overshoot}");
    }

    #[test]
    fn the_page_slide_never_overshoots() {
        let (stiffness, damping) = PAGE_SLIDE_RESPONSE;
        assert!(damping / (2.0 * stiffness.sqrt()) >= 1.0);
    }
}
