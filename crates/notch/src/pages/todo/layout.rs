//! Which rows the todo page shows and where the toolbar, its buttons and the rows sit.

use super::list::MAX_TASKS;
use crate::{gfx::Rect, page::DEFAULT_CONTENT_HEIGHT};

pub(super) const CONTENT_WIDTH: f32 = 508.0;
/// The empty page is as wide as the timer page's open notch (322) less the notch's 32 of padding.
pub(super) const EMPTY_WIDTH: f32 = 290.0;
pub(super) const TOOLBAR_HEIGHT: f32 = 28.0;
pub(super) const TOOLBAR_GAP: f32 = 8.0;
pub(super) const BUTTON_HEIGHT: f32 = 26.0;
pub(super) const BUTTON_GAP: f32 = 8.0;
pub(super) const ADD_WIDTH: f32 = 60.0;
pub(super) const CLEAR_WIDTH: f32 = 88.0;
pub(super) const ROW_HEIGHT: f32 = 32.0;
pub(super) const ROW_GAP: f32 = 6.0;
pub(super) const ROW_PADDING: f32 = 10.0;
pub(super) const CHECK_SIZE: f32 = 14.0;
pub(super) const TEXT_LEFT: f32 = ROW_PADDING + CHECK_SIZE + 10.0;
pub(super) const DELETE_SIZE: f32 = 22.0;
pub(super) const DELETE_INSET: f32 = 6.0;
/// Rows the open notch shows at most; a longer list from an older version ends in a "more" line.
pub(super) const MAX_ROWS: usize = MAX_TASKS;

/// What a row of the list shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Row {
    Item(usize),
    /// The input, the last row while typing.
    Field,
    More(usize),
}

/// The rows for the tasks in `order` (indices into the list), then the input while typing and there is room.
pub(super) fn plan_rows(order: &[usize], is_adding: bool) -> Vec<Row> {
    if order.len() > MAX_ROWS {
        let shown = MAX_ROWS - 1;
        let mut rows: Vec<Row> = order[..shown].iter().copied().map(Row::Item).collect();
        rows.push(Row::More(order.len() - shown));
        return rows;
    }
    let mut rows: Vec<Row> = order.iter().copied().map(Row::Item).collect();
    if is_adding && order.len() < MAX_TASKS {
        rows.push(Row::Field);
    }
    rows
}

/// The open height: the toolbar, then the rows; the usual page height until they need more.
pub(super) fn content_height(row_count: usize) -> f32 {
    let rows = row_count as f32;
    let rows_height = rows * ROW_HEIGHT + (rows - 1.0).max(0.0) * ROW_GAP;
    (TOOLBAR_HEIGHT + TOOLBAR_GAP + rows_height).max(DEFAULT_CONTENT_HEIGHT)
}

/// Every row runs the full width, under the toolbar.
pub(super) fn row_rect(bounds: Rect, scale: f32, position: usize) -> Rect {
    let top = bounds.top + (TOOLBAR_HEIGHT + TOOLBAR_GAP + position as f32 * (ROW_HEIGHT + ROW_GAP)) * scale;
    Rect::new(bounds.left, top, bounds.right, top + ROW_HEIGHT * scale)
}

/// The room under the toolbar, where the empty page says "All clear".
pub(super) fn empty_area(bounds: Rect, scale: f32) -> Rect {
    Rect::new(bounds.left, bounds.top + (TOOLBAR_HEIGHT + TOOLBAR_GAP) * scale, bounds.right, bounds.bottom)
}

fn button_rect(bounds: Rect, scale: f32, right_inset: f32, width: f32) -> Rect {
    let top = bounds.top + (TOOLBAR_HEIGHT - BUTTON_HEIGHT) / 2.0 * scale;
    let right = bounds.right - right_inset * scale;
    Rect::new(right - width * scale, top, right, top + BUTTON_HEIGHT * scale)
}

/// The Add pill, always at the top right.
pub(super) fn add_button(bounds: Rect, scale: f32) -> Rect {
    button_rect(bounds, scale, 0.0, ADD_WIDTH)
}

/// The Clear all pill, left of Add.
pub(super) fn clear_button(bounds: Rect, scale: f32) -> Rect {
    button_rect(bounds, scale, ADD_WIDTH + BUTTON_GAP, CLEAR_WIDTH)
}

pub(super) fn check_rect(row: Rect, scale: f32) -> Rect {
    let left = row.left + ROW_PADDING * scale;
    let top = row.center_y() - CHECK_SIZE * scale / 2.0;
    Rect::new(left, top, left + CHECK_SIZE * scale, top + CHECK_SIZE * scale)
}

pub(super) fn delete_rect(row: Rect, scale: f32) -> Rect {
    let right = row.right - DELETE_INSET * scale;
    let top = row.center_y() - DELETE_SIZE * scale / 2.0;
    Rect::new(right - DELETE_SIZE * scale, top, right, top + DELETE_SIZE * scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_tasks_are_rows_and_typing_adds_the_input_below_them() {
        assert_eq!(plan_rows(&[], false), []);
        assert_eq!(plan_rows(&[0, 1, 2], false), [Row::Item(0), Row::Item(1), Row::Item(2)]);
        assert_eq!(plan_rows(&[0, 1], true), [Row::Item(0), Row::Item(1), Row::Field]);
        assert_eq!(plan_rows(&[], true), [Row::Field]);
    }

    #[test]
    fn five_tasks_leave_no_room_for_the_input() {
        let order = [0, 1, 2, 3, 4];
        assert_eq!(plan_rows(&order, false).len(), 5);
        assert!(!plan_rows(&order, true).contains(&Row::Field));
    }

    #[test]
    fn a_list_from_an_older_version_ends_in_a_count_of_the_rest() {
        let order: Vec<usize> = (0..7).collect();
        assert_eq!(plan_rows(&order, false), [Row::Item(0), Row::Item(1), Row::Item(2), Row::Item(3), Row::More(3)]);
    }

    #[test]
    fn the_page_height_is_the_toolbar_plus_the_rows_as_in_the_design() {
        assert_eq!(content_height(0), DEFAULT_CONTENT_HEIGHT);
        assert_eq!(content_height(1), DEFAULT_CONTENT_HEIGHT);
        assert_eq!(content_height(2), 106.0);
        assert_eq!(content_height(3), 144.0);
        assert_eq!(content_height(MAX_ROWS), 220.0);
    }

    #[test]
    fn rows_run_the_full_width_below_the_toolbar_with_six_between() {
        let bounds = Rect::new(0.0, 0.0, CONTENT_WIDTH, 144.0);
        let first = row_rect(bounds, 1.0, 0);
        let second = row_rect(bounds, 1.0, 1);
        assert_eq!((first.left, first.right), (0.0, CONTENT_WIDTH));
        assert_eq!(first.top, TOOLBAR_HEIGHT + TOOLBAR_GAP);
        assert_eq!(second.top - first.bottom, ROW_GAP);
        assert_eq!(row_rect(bounds, 1.0, 2).bottom, bounds.bottom);
    }

    #[test]
    fn the_buttons_sit_at_the_top_right_with_clear_all_left_of_add() {
        let bounds = Rect::new(0.0, 0.0, CONTENT_WIDTH, 144.0);
        let (add, clear) = (add_button(bounds, 1.0), clear_button(bounds, 1.0));
        assert_eq!(add.right, bounds.right);
        assert_eq!(add.height(), 26.0);
        assert_eq!(add.top, 1.0);
        assert_eq!(add.left - clear.right, BUTTON_GAP);
        assert!(clear.bottom <= row_rect(bounds, 1.0, 0).top);
    }

    #[test]
    fn the_empty_page_keeps_its_buttons_on_top_and_its_message_below() {
        let bounds = Rect::new(0.0, 0.0, EMPTY_WIDTH, 86.0);
        assert_eq!(add_button(bounds, 1.0).right, EMPTY_WIDTH);
        assert_eq!(empty_area(bounds, 1.0).top, 36.0);
        assert_eq!(empty_area(bounds, 1.0).bottom, 86.0);
    }
}
