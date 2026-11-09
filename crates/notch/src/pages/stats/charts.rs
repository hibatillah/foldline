//! The load charts (a dotted baseline with two bars at its right end) and the disk bars.

use super::DISK_GREEN;
use crate::gfx::{Canvas, Color, Rect};
use crate::ui::palette::SURFACE_BORDER;

const BASELINE_LEFT: f32 = 12.0;
const BASELINE_RIGHT: f32 = 118.0;
const BASELINE_STROKE: f32 = 1.4;
const BASELINE_DOT: f32 = 1.4;
const BASELINE_PITCH: f32 = 4.0;
const BASELINE_OPACITY: f32 = 0.9;
const BAR_WIDTH: f32 = 2.6;
const OLDER_BAR_LEFT: f32 = 111.5;
const NEWER_BAR_LEFT: f32 = 115.5;
const BAR_RADIUS: f32 = 1.0;
const BAR_MIN_HEIGHT: f32 = 1.0;
const BAR_MAX_HEIGHT: f32 = 12.0;
/// A load below this draws the newer bar alone, at its smallest.
const IDLE_LOAD: f32 = 0.005;

pub(super) const DISK_BAR_LEFT: f32 = 34.0;
pub(super) const DISK_BAR_WIDTH: f32 = 56.0;
pub(super) const DISK_BAR_HEIGHT: f32 = 4.0;
/// From the top of a disk row's text to the top of its bar.
const DISK_BAR_OFFSET: f32 = 5.0;

/// The line and bars of one chart: the newer bar and the line share `base`, the older bar is the lighter `light`.
#[derive(Clone, Copy)]
pub(super) struct Accent {
    pub base: Color,
    pub light: Color,
}

/// How high the bars stand for `load` (0 to 1): 1 px at rest up to 12 px at full load.
pub(super) fn bar_height(load: f32) -> f32 {
    BAR_MIN_HEIGHT + (BAR_MAX_HEIGHT - BAR_MIN_HEIGHT) * load.clamp(0.0, 1.0)
}

/// Draws the dotted baseline `baseline` px below the card top and two bars standing on it, as high as `load` (0 to 1).
pub(super) fn draw_load(
    canvas: &Canvas,
    card: Rect,
    scale: f32,
    baseline: f32,
    load: f32,
    accent: Accent,
    opacity: f32,
) {
    let line_top = card.top + (baseline - BASELINE_STROKE / 2.0) * scale;
    let mut left = BASELINE_LEFT;
    while left < BASELINE_RIGHT {
        let right = (left + BASELINE_DOT).min(BASELINE_RIGHT);
        let dot = Rect::new(
            card.left + left * scale,
            line_top,
            card.left + right * scale,
            line_top + BASELINE_STROKE * scale,
        );
        canvas.fill_rounded_rect(dot, 0.0, accent.base.faded(opacity * BASELINE_OPACITY));
        left += BASELINE_PITCH;
    }
    let bottom = card.top + (baseline + BASELINE_STROKE / 2.0) * scale;
    let top = bottom - bar_height(load) * scale;
    let bar = |left: f32, color: Color| {
        let rect = Rect::new(card.left + left * scale, top, card.left + (left + BAR_WIDTH) * scale, bottom);
        canvas.fill_rounded_rect(rect, BAR_RADIUS * scale, color.faded(opacity));
    };
    if load >= IDLE_LOAD {
        bar(OLDER_BAR_LEFT, accent.light);
    }
    bar(NEWER_BAR_LEFT, accent.base);
}

/// The 56 x 4 track of a disk row whose text starts `row_top` px below the card top.
pub(super) fn disk_bar_track(card: Rect, scale: f32, row_top: f32) -> Rect {
    let top = card.top + (row_top + DISK_BAR_OFFSET) * scale;
    let left = card.left + DISK_BAR_LEFT * scale;
    Rect::new(left, top, left + DISK_BAR_WIDTH * scale, top + DISK_BAR_HEIGHT * scale)
}

pub(super) fn draw_disk_bar(canvas: &Canvas, track: Rect, percent: f32, opacity: f32) {
    let half = track.height() / 2.0;
    canvas.fill_rounded_rect(track, half, SURFACE_BORDER.faded(opacity));
    let fill_right = track.left + track.width() * (percent / 100.0).clamp(0.0, 1.0);
    if fill_right - track.left > half {
        canvas.fill_rounded_rect(Rect { right: fill_right, ..track }, half, DISK_GREEN.faded(opacity));
    }
}

#[cfg(test)]
mod tests {
    use super::{super::TILE_PADDING_X, *};

    #[test]
    fn bars_grow_from_one_pixel_at_rest_to_twelve_at_full_load() {
        assert_eq!(bar_height(0.0), 1.0);
        assert_eq!(bar_height(1.0), 12.0);
        assert_eq!(bar_height(-3.0), 1.0);
        assert_eq!(bar_height(9.0), 12.0);
        assert!(bar_height(0.75) > bar_height(0.14));
    }

    #[test]
    fn the_baseline_and_both_bars_keep_the_card_padding() {
        assert_eq!(BASELINE_LEFT, TILE_PADDING_X);
        const {
            assert!(NEWER_BAR_LEFT + BAR_WIDTH <= BASELINE_RIGHT + 0.2);
            assert!(OLDER_BAR_LEFT + BAR_WIDTH < NEWER_BAR_LEFT);
        }
    }

    #[test]
    fn the_disk_bar_is_56_by_4_and_starts_at_34() {
        let card = Rect::new(10.0, 20.0, 142.0, 106.0);
        let track = disk_bar_track(card, 1.0, 32.0);
        assert_eq!((track.left, track.width(), track.height()), (44.0, 56.0, 4.0));
        assert_eq!(track.top, 20.0 + 37.0);
    }
}
