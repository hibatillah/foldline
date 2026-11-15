//! Every tuning number of the notch's size animation and of page switching input.

use std::time::Duration;

/// Spring (stiffness, damping) of every size change: open, close, page width and height, badge room.
/// Damping ratio is `19.7 / (2 * sqrt(215))` = 0.67, which overshoots a move by about 5 percent (5.8 without the lead-in).
pub(super) const SIZE_RESPONSE: (f32, f32) = (215.0, 19.7);

/// The pull of a size spring grows from nothing over this long when a move starts at rest, so it begins gently.
pub(super) const SIZE_LEAD_IN: f32 = 0.09;

/// Spring (stiffness, damping) of the page dots' long mark moving to the open page; critically damped so it never passes it.
pub(super) const PAGE_SLIDE_RESPONSE: (f32, f32) = (650.0, 51.0);

/// Seconds the leaving page takes to fade out (eased out, so most of it is gone in the first half).
pub(super) const PAGE_SWAP_OUT_SECONDS: f32 = 0.11;

/// DIP the leaving page drifts away from the arriving one while it fades.
pub(super) const PAGE_SWAP_OUT_SHIFT: f32 = 12.0;

/// Seconds after a swap starts before the arriving page begins to show, so the leaving one is nearly gone.
pub(super) const PAGE_SWAP_IN_DELAY: f32 = 0.08;

/// Seconds the arriving page takes to shift into place; with the delay it ends just after a size spring is within
/// 3 percent of its move (about 0.41 s), so the pill has nearly its new size before the content is still.
pub(super) const PAGE_SWAP_IN_SECONDS: f32 = 0.34;

/// DIP the arriving page starts away from its place, on the side it comes from (east for the next page).
pub(super) const PAGE_SWAP_IN_SHIFT: f32 = 24.0;

/// Share of the arriving page's move over which it fades in; the rest of the move is at full opacity.
pub(super) const PAGE_SWAP_IN_FADE_SHARE: f32 = 0.6;

/// The open pill may shrink to this fraction of its collapsed size when a spring undershoots.
pub(super) const MIN_SIZE_FRACTION: f32 = 0.75;

/// Furthest the open amount (1.0 is fully open) may extrapolate on either side while the spring overshoots.
pub(super) const EXPANSION_OVERSHOOT_LIMIT: (f32, f32) = (-0.25, 1.3);

/// A wheel notch reports this many units; smaller deltas come from precision wheels and touchpads.
pub(super) const WHEEL_NOTCH: i32 = 120;

/// Scroll input is ignored for this long after a page switch, so one scroll moves one page.
pub(super) const PAGE_SWITCH_COOLDOWN: Duration = Duration::from_millis(300);

/// A touchpad or precision-wheel gesture ends when no scroll event arrives for this long.
pub(super) const GESTURE_QUIET_TIME: Duration = Duration::from_millis(150);

/// Accumulated scroll units in one gesture that switch a page.
pub(super) const SWIPE_SWITCH_THRESHOLD: f32 = 120.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_size_spring_is_underdamped_with_a_small_overshoot() {
        let (stiffness, damping) = SIZE_RESPONSE;
        let damping_ratio = damping / (2.0 * stiffness.sqrt());
        assert!((0.66..=0.68).contains(&damping_ratio), "damping ratio {damping_ratio}");
        let overshoot = (-std::f32::consts::PI * damping_ratio / (1.0 - damping_ratio * damping_ratio).sqrt()).exp();
        assert!((0.05..=0.065).contains(&overshoot), "overshoot {overshoot}");
    }

    #[test]
    fn the_page_slide_never_overshoots() {
        let (stiffness, damping) = PAGE_SLIDE_RESPONSE;
        assert!(damping / (2.0 * stiffness.sqrt()) >= 1.0);
    }
}
