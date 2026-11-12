//! Where the day grid, its cells and the event cards sit; both columns end on the bottom of the page.

use super::dates::DayGrid;
use super::{COLUMN_GAP, LEFT_WIDTH, view::View};
use crate::gfx::{Point, Rect};

pub(super) const CELL_WIDTH: f32 = 26.0;
pub(super) const CELL_HEIGHT: f32 = 20.0;
pub(super) const CIRCLE_SIZE: f32 = 20.0;
pub(super) const HEADER_HEIGHT: f32 = 24.0;
pub(super) const WEEKDAY_HEIGHT: f32 = 12.0;
/// Month line to weekday letters: a 2 px margin and the 6 px column gap.
const HEADER_GAP: f32 = 8.0;
/// Weekday letters to the first row of dates.
const WEEKDAY_GAP: f32 = 6.0;
/// The month view's events viewport: four cards and four gaps (156 px) plus 12 px of a fifth.
pub(super) const MONTH_VIEWPORT_HEIGHT: f32 = 168.0;
pub(super) const ROW_HEIGHT: f32 = 34.0;
pub(super) const ROW_GAP: f32 = 5.0;
/// One wheel notch scrolls one card.
pub(super) const WHEEL_NOTCH_DISTANCE: f32 = ROW_HEIGHT + ROW_GAP;

pub(super) fn widest_form(text: &str) -> String {
    text.chars().map(|character| if character.is_ascii_digit() { '0' } else { character }).collect()
}

/// Height of the left column: the month line, the weekday letters and the rows of the month.
pub(super) fn left_height(rows: usize) -> f32 {
    HEADER_HEIGHT + HEADER_GAP + WEEKDAY_HEIGHT + WEEKDAY_GAP + rows as f32 * CELL_HEIGHT
}

/// The page is the viewport, or the grid when that is taller (a six-row month).
pub(super) fn content_height(rows: usize, view: View) -> f32 {
    left_height(rows).max(view.viewport_height())
}

/// The cards stacked end to end.
pub(super) fn stack_height(row_count: usize) -> f32 {
    row_count as f32 * ROW_HEIGHT + row_count.saturating_sub(1) as f32 * ROW_GAP
}

/// How far the list can scroll: zero while the cards fit the viewport.
pub(super) fn max_scroll(row_count: usize, view: View) -> f32 {
    (stack_height(row_count) - view.viewport_height()).max(0.0)
}

/// The viewport sits on the bottom of the page, right of the left column.
pub(super) fn events_area(bounds: Rect, view: View) -> Rect {
    Rect::new(
        bounds.left + (LEFT_WIDTH + COLUMN_GAP),
        bounds.bottom - view.viewport_height(),
        bounds.right,
        bounds.bottom,
    )
}

/// The month line and the year, above the weekday letters.
pub(super) fn header(bounds: Rect, rows: usize) -> Rect {
    let bottom = bounds.bottom - (rows as f32 * CELL_HEIGHT + WEEKDAY_GAP + WEEKDAY_HEIGHT + HEADER_GAP);
    Rect::new(bounds.left, bottom - HEADER_HEIGHT, bounds.left + LEFT_WIDTH, bottom)
}

/// The row of weekday letters, 6 px above the first row of dates.
pub(super) fn weekday_row(bounds: Rect, rows: usize) -> Rect {
    let bottom = bounds.bottom - (rows as f32 * CELL_HEIGHT + WEEKDAY_GAP);
    Rect::new(bounds.left, bottom - WEEKDAY_HEIGHT, bounds.left + 7.0 * CELL_WIDTH, bottom)
}

/// The cell of grid position `cell` (0 is the Monday of the first row).
pub(super) fn day_cell(cell: usize, rows: usize, bounds: Rect) -> Rect {
    let (row, column) = (cell / 7, cell % 7);
    let top = bounds.bottom - (rows - row) as f32 * CELL_HEIGHT;
    let left = bounds.left + column as f32 * CELL_WIDTH;
    Rect::new(left, top, left + CELL_WIDTH, top + CELL_HEIGHT)
}

/// The 20 px circle inside a cell.
pub(super) fn day_circle(cell: Rect) -> Rect {
    let inset = (cell.width() - CIRCLE_SIZE) / 2.0;
    Rect::new(cell.left + inset, cell.top, cell.right - inset, cell.bottom)
}

/// The grid position under `point`; only cells that hold a day count.
pub(super) fn day_at(point: Point, grid: &DayGrid, bounds: Rect) -> Option<usize> {
    let rows = grid.rows();
    (grid.leading..grid.leading + grid.day_count as usize).find(|&cell| day_cell(cell, rows, bounds).contains(point))
}

/// Card `position` of the list, moved up by `scroll`; the list starts at the top of the viewport.
pub(super) fn row_rect(position: usize, scroll: f32, bounds: Rect, view: View) -> Rect {
    let area = events_area(bounds, view);
    let top = area.top + (position as f32 * (ROW_HEIGHT + ROW_GAP) - scroll);
    Rect::new(area.left, top, area.right, top + ROW_HEIGHT)
}

#[cfg(test)]
mod tests {
    use super::super::dates::{DayGrid, days_from_civil};
    use super::*;
    use crate::page::DEFAULT_CONTENT_HEIGHT;

    const BOUNDS: Rect = Rect::new(0.0, 0.0, 508.0, 168.0);

    #[test]
    fn the_page_is_the_viewport_until_the_month_has_six_rows() {
        assert_eq!(left_height(1), 70.0);
        assert_eq!(left_height(4), 130.0);
        assert_eq!(left_height(5), 150.0);
        assert_eq!(left_height(6), 170.0);
        assert_eq!(content_height(5, View::Month), 168.0);
        assert_eq!(content_height(6, View::Month), 170.0);
    }

    #[test]
    fn the_week_view_is_as_tall_as_the_default_page_and_its_list_scrolls() {
        assert_eq!(content_height(1, View::Week), DEFAULT_CONTENT_HEIGHT);
        assert_eq!(max_scroll(4, View::Week), stack_height(4) - 86.0);
        assert!(left_height(1) < DEFAULT_CONTENT_HEIGHT, "the week row fits the page");
    }

    #[test]
    fn the_month_list_scrolls_past_four_and_a_bit_cards() {
        assert_eq!(max_scroll(0, View::Month), 0.0);
        assert_eq!(max_scroll(4, View::Month), 0.0);
        assert_eq!(max_scroll(5, View::Month), 190.0 - 168.0);
        assert_eq!(max_scroll(9, View::Month), 178.0, "nine events end flush with the viewport after 178 px");
    }

    #[test]
    fn the_week_list_scrolls_past_two_cards() {
        assert_eq!(max_scroll(2, View::Week), 0.0);
        assert_eq!(max_scroll(3, View::Week), 26.0);
        assert_eq!(max_scroll(4, View::Week), 65.0);
    }

    #[test]
    fn the_fifth_card_shows_by_12_px_when_there_are_more_than_four_events() {
        let fifth = row_rect(4, 0.0, BOUNDS, View::Month);
        assert_eq!(BOUNDS.bottom - fifth.top, 12.0);
        assert_eq!(fifth.height(), 34.0);
        assert_eq!(row_rect(0, 0.0, BOUNDS, View::Month).top, BOUNDS.top);
    }

    #[test]
    fn the_cards_start_at_the_top_of_the_viewport_and_move_up_with_the_scroll() {
        let tall = Rect::new(0.0, 0.0, 508.0, 170.0);
        let first = row_rect(0, 0.0, tall, View::Month);
        assert_eq!(first.top, 2.0);
        assert_eq!(row_rect(1, 0.0, tall, View::Month).top - first.bottom, ROW_GAP);
        assert_eq!(row_rect(2, 39.0, tall, View::Month).top, row_rect(1, 0.0, tall, View::Month).top);
    }

    #[test]
    fn the_month_sits_on_the_bottom_edge_below_the_weekday_letters_and_the_month_line() {
        let rows = 6;
        let last_row = day_cell(35, rows, BOUNDS);
        assert_eq!(last_row.bottom, BOUNDS.bottom);
        assert_eq!(last_row.left, BOUNDS.left);
        assert_eq!(day_cell(6, rows, BOUNDS).right, 7.0 * CELL_WIDTH);
        assert_eq!(weekday_row(BOUNDS, rows).bottom + 6.0, day_cell(0, rows, BOUNDS).top);
        assert_eq!(header(BOUNDS, rows).bottom + 8.0, weekday_row(BOUNDS, rows).top);
        assert_eq!(header(BOUNDS, rows).height(), HEADER_HEIGHT);
        assert_eq!(day_circle(day_cell(0, rows, BOUNDS)).width(), CIRCLE_SIZE);
    }

    #[test]
    fn a_press_finds_only_the_days_of_the_month() {
        let month = DayGrid::month_of(days_from_civil(2026, 11, 13));
        let rows = month.rows();
        let first = day_cell(month.leading, rows, BOUNDS);
        assert_eq!(day_at(Point::new(first.center_x(), first.center_y()), &month, BOUNDS), Some(month.leading));
        let blank = day_cell(0, rows, BOUNDS);
        assert_eq!(day_at(Point::new(blank.center_x(), blank.center_y()), &month, BOUNDS), None);
        assert_eq!(day_at(Point::new(LEFT_WIDTH + 40.0, 100.0), &month, BOUNDS), None);
    }

    #[test]
    fn the_events_sit_16_px_to_the_right_of_the_left_column_on_the_bottom() {
        let area = events_area(BOUNDS, View::Month);
        assert_eq!(area.left, LEFT_WIDTH + COLUMN_GAP);
        assert_eq!((area.right, area.bottom, area.height()), (BOUNDS.right, BOUNDS.bottom, MONTH_VIEWPORT_HEIGHT));
    }

    #[test]
    fn the_widest_form_keeps_the_shape_and_drops_the_digits() {
        assert_eq!(widest_form("in 12m"), "in 00m");
    }
}
