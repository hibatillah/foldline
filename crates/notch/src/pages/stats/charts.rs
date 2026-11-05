//! Area charts that run to the card's edge, and the bars that keep the card's padding.

use std::collections::VecDeque;

use super::{HISTORY_LENGTH, TILE_PADDING_X, TILE_RADIUS, TRACK, inside};
use crate::gfx::{Canvas, Color, Rect};

/// How tall an area chart is at the bottom of its card.
const CHART_HEIGHT: f32 = 18.0;
/// A chart is drawn in columns this wide, so no curve or path geometry is needed.
const COLUMN_WIDTH: f32 = 2.0;
const LINE_HEIGHT: f32 = 1.5;
const FILL_ALPHA: f32 = 0.3;

/// The history value at `position` across the chart, between 0 and 1 from left to right.
/// The newest reading sits at the right edge; slots before the first reading have no value.
pub(super) fn history_at(history: &VecDeque<f32>, position: f32) -> Option<f32> {
    let first_slot = HISTORY_LENGTH.saturating_sub(history.len());
    let slot = position.clamp(0.0, 1.0) * (HISTORY_LENGTH - 1) as f32 - first_slot as f32;
    if slot < 0.0 || history.is_empty() {
        return None;
    }
    let lower = (slot.floor() as usize).min(history.len() - 1);
    let upper = (lower + 1).min(history.len() - 1);
    let blend = slot - lower as f32;
    Some(history[lower] + (history[upper] - history[lower]) * blend)
}

/// Slots before the first reading count as zero, so an empty chart is a flat line at the baseline.
pub(super) fn column_value(history: &VecDeque<f32>, position: f32) -> f32 {
    history_at(history, position).unwrap_or(0.0)
}

/// How far the card's rounded bottom corner pulls the chart up at `distance` from the card's side edge.
pub(super) fn corner_inset(distance: f32, radius: f32) -> f32 {
    if distance >= radius {
        return 0.0;
    }
    let across = radius - distance.max(0.0);
    radius - (radius * radius - across * across).max(0.0).sqrt()
}

/// Draws `history` (values from 0 to 1) as a filled chart along the bottom of `tile`, edge to edge, following the
/// tile's rounded bottom corners.
pub(super) fn draw_area(canvas: &Canvas, tile: Rect, scale: f32, history: &VecDeque<f32>, color: Color, opacity: f32) {
    let height = CHART_HEIGHT * scale;
    let radius = TILE_RADIUS * scale;
    let step = COLUMN_WIDTH * scale;
    let columns = (tile.width() / step).ceil() as usize;
    for column in 0..columns {
        let left = tile.left + column as f32 * step;
        let right = (left + step).min(tile.right);
        let middle = (left + right) / 2.0;
        let value = column_value(history, (middle - tile.left) / tile.width());
        let bottom = tile.bottom - corner_inset((middle - tile.left).min(tile.right - middle), radius);
        let top = (tile.bottom - (value.clamp(0.0, 1.0) * height).max(LINE_HEIGHT * scale)).min(bottom);
        if bottom - top <= 0.0 {
            continue;
        }
        canvas.fill_rounded_rect(Rect::new(left, top, right, bottom), 0.0, color.with_alpha(FILL_ALPHA * opacity));
        let line_bottom = (top + LINE_HEIGHT * scale).min(bottom);
        canvas.fill_rounded_rect(Rect::new(left, top, right, line_bottom), 0.0, color.faded(opacity * 0.95));
    }
}

/// Where a bar sits in a card: the card's padding on both sides, centered on `center_y`.
pub(super) fn bar_track(tile: Rect, scale: f32, center_y: f32, height: f32) -> Rect {
    let area = inside(tile, scale, TILE_PADDING_X, 0.0, 0.0);
    let half = height * scale / 2.0;
    Rect::new(area.left, center_y - half, area.right, center_y + half)
}

/// A value, not a chart: the bar keeps the card's padding on both sides.
pub(super) fn draw_bar(canvas: &Canvas, track: Rect, percent: f32, color: Color, opacity: f32) {
    let half = track.height() / 2.0;
    canvas.fill_rounded_rect(track, half, TRACK.faded(opacity));
    let fill_right = track.left + track.width() * (percent / 100.0).clamp(0.0, 1.0);
    if fill_right - track.left > half {
        canvas.fill_rounded_rect(Rect { right: fill_right, ..track }, half, color.faded(opacity));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn history(values: &[f32]) -> VecDeque<f32> {
        values.iter().copied().collect()
    }

    #[test]
    fn the_newest_reading_sits_at_the_right_edge_and_older_slots_are_empty() {
        let readings = history(&[0.2, 0.8]);
        assert_eq!(history_at(&readings, 1.0), Some(0.8));
        assert_eq!(history_at(&readings, 0.0), None);
        let full: VecDeque<f32> = (0..HISTORY_LENGTH).map(|index| index as f32 / 100.0).collect();
        assert_eq!(history_at(&full, 0.0), Some(0.0));
    }

    #[test]
    fn a_position_between_two_readings_blends_them() {
        let full: VecDeque<f32> = (0..HISTORY_LENGTH).map(|index| index as f32).collect();
        let halfway = 0.5 / (HISTORY_LENGTH - 1) as f32;
        assert!((history_at(&full, halfway).unwrap() - 0.5).abs() < 0.001);
    }

    #[test]
    fn no_readings_have_no_history_value_but_draw_as_zero() {
        assert_eq!(history_at(&VecDeque::new(), 0.5), None);
        assert_eq!(column_value(&VecDeque::new(), 0.0), 0.0);
        assert_eq!(column_value(&VecDeque::new(), 1.0), 0.0);
    }

    #[test]
    fn a_short_history_is_zero_on_the_left_and_grows_in_from_the_right() {
        let readings = history(&[0.5, 0.8]);
        assert_eq!(column_value(&readings, 0.0), 0.0);
        assert_eq!(column_value(&readings, 0.5), 0.0);
        assert_eq!(column_value(&readings, 1.0), 0.8);
    }

    #[test]
    fn the_rounded_corner_pulls_the_chart_up_only_near_the_side() {
        assert_eq!(corner_inset(12.0, 12.0), 0.0);
        assert_eq!(corner_inset(40.0, 12.0), 0.0);
        assert!((corner_inset(0.0, 12.0) - 12.0).abs() < 0.001);
        assert!(corner_inset(3.0, 12.0) > corner_inset(8.0, 12.0));
    }
}
