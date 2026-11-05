//! Where the week, its days and the event rows sit.

use super::dates::week_start;
use super::{COLUMN_GAP, DAY_LETTER_HEIGHT, DAY_TOP_PADDING, LEFT_WIDTH, TODAY_SIZE};
use crate::gfx::{Point, Rect};

pub(super) const DAY_WIDTH: f32 = 26.0;
pub(super) const DAY_HEIGHT: f32 = DAY_TOP_PADDING + DAY_LETTER_HEIGHT + 1.0 + TODAY_SIZE + 1.0;
pub(super) const ROW_HEIGHT: f32 = 40.0;
pub(super) const ROW_GAP: f32 = 6.0;

pub(super) fn widest_form(text: &str) -> String {
    text.chars().map(|character| if character.is_ascii_digit() { '0' } else { character }).collect()
}

pub(super) fn events_area(bounds: Rect, scale: f32) -> Rect {
    Rect::new(bounds.left + (LEFT_WIDTH + 2.0 * COLUMN_GAP + 1.0) * scale, bounds.top, bounds.right, bounds.bottom)
}

pub(super) fn day_cell(position: usize, bounds: Rect, scale: f32) -> Rect {
    let spacing = (LEFT_WIDTH - 7.0 * DAY_WIDTH) / 6.0;
    let left = bounds.left + position as f32 * (DAY_WIDTH + spacing) * scale;
    Rect::new(left, bounds.bottom - DAY_HEIGHT * scale, left + DAY_WIDTH * scale, bounds.bottom)
}

pub(super) fn day_at(point: Point, bounds: Rect, scale: f32) -> Option<usize> {
    // The gaps between days belong to the day beside them, so moving along the week never loses the pointer.
    let spacing = (LEFT_WIDTH - 7.0 * DAY_WIDTH) / 6.0;
    (0..7).find(|&position| {
        let cell = day_cell(position, bounds, scale);
        Rect::new(cell.left - spacing * scale / 2.0, cell.top, cell.right + spacing * scale / 2.0, cell.bottom)
            .contains(point)
    })
}

/// Whether `day` is in the same week (Monday to Sunday) as `today`.
pub(super) fn is_in_week_of(day: i64, today: i64) -> bool {
    (week_start(today)..week_start(today) + 7).contains(&day)
}

pub(super) fn row_rect(position: usize, bounds: Rect, scale: f32) -> Rect {
    let area = events_area(bounds, scale);
    let top = area.top + position as f32 * (ROW_HEIGHT + ROW_GAP) * scale;
    Rect::new(area.left, top, area.right, top + ROW_HEIGHT * scale)
}

#[cfg(test)]
mod tests {
    use super::super::*;

    use super::super::dates::days_from_civil;

    const BOUNDS: Rect = Rect::new(0.0, 0.0, 508.0, 86.0);

    #[test]
    fn the_rows_fill_the_height_of_the_page() {
        let last = row_rect(agenda::MAX_ROWS - 1, BOUNDS, 1.0);
        assert_eq!(last.bottom, BOUNDS.bottom);
        assert!(row_rect(1, BOUNDS, 1.0).top > row_rect(0, BOUNDS, 1.0).bottom);
    }

    #[test]
    fn the_week_sits_on_the_bottom_edge_and_each_day_has_its_own_place() {
        let (first, last) = (day_cell(0, BOUNDS, 1.0), day_cell(6, BOUNDS, 1.0));
        assert_eq!(first.bottom, BOUNDS.bottom);
        assert_eq!(first.left, BOUNDS.left);
        assert!((last.right - LEFT_WIDTH).abs() < 0.01);
        assert!(day_cell(1, BOUNDS, 1.0).left > first.right);
        assert!(first.top >= HEADER_TOP + 28.0, "the week stays below the month");
    }

    #[test]
    fn a_press_finds_the_day_under_it_even_between_two_days() {
        for position in 0..7 {
            let cell = day_cell(position, BOUNDS, 1.0);
            assert_eq!(day_at(Point::new(cell.center_x(), cell.center_y()), BOUNDS, 1.0), Some(position));
        }
        let (first, second) = (day_cell(0, BOUNDS, 1.0), day_cell(1, BOUNDS, 1.0));
        let between = Point::new((first.right + second.left) / 2.0, first.center_y());
        assert!(day_at(between, BOUNDS, 1.0).is_some());
        assert_eq!(day_at(Point::new(LEFT_WIDTH + 40.0, 60.0), BOUNDS, 1.0), None);
        assert_eq!(day_at(Point::new(50.0, 5.0), BOUNDS, 1.0), None);
    }

    #[test]
    fn the_week_is_monday_to_sunday() {
        let monday = week_start(days_from_civil(2026, 10, 28));
        assert!(is_in_week_of(monday, monday + 3));
        assert!(is_in_week_of(monday + 6, monday));
        assert!(!is_in_week_of(monday + 7, monday));
        assert!(!is_in_week_of(monday - 1, monday + 2));
    }

    #[test]
    fn the_events_sit_to_the_right_of_the_week() {
        assert!(events_area(BOUNDS, 1.0).left > LEFT_WIDTH + COLUMN_GAP);
        assert_eq!(events_area(BOUNDS, 1.0).right, BOUNDS.right);
    }

    #[test]
    fn the_widest_form_keeps_the_shape_and_drops_the_digits() {
        assert_eq!(widest_form("in 12m"), "in 00m");
    }
}
