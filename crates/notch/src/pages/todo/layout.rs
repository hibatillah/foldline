//! Which rows the todo page shows and where the toolbar, its buttons and the rows sit.

use crate::{gfx::Rect, page::DEFAULT_CONTENT_HEIGHT};

pub(super) const CONTENT_WIDTH: f32 = 508.0;
pub(super) const TOOLBAR_HEIGHT: f32 = 28.0;
pub(super) const TOOLBAR_GAP: f32 = 8.0;
pub(super) const BUTTON_HEIGHT: f32 = 26.0;
pub(super) const BUTTON_GAP: f32 = 8.0;
pub(super) const ADD_WIDTH: f32 = 72.0;
pub(super) const CLEAR_WIDTH: f32 = 80.0;
/// Wide enough for "Urungkan" in the Clear all style.
pub(super) const UNDO_WIDTH: f32 = 80.0;
pub(super) const ROW_HEIGHT: f32 = 32.0;
pub(super) const ROW_GAP: f32 = 6.0;
pub(super) const ROW_PADDING: f32 = 10.0;
pub(super) const CHECK_SIZE: f32 = 14.0;
pub(super) const TEXT_LEFT: f32 = ROW_PADDING + CHECK_SIZE + 10.0;
pub(super) const DELETE_SIZE: f32 = 22.0;
pub(super) const DELETE_INSET: f32 = 6.0;
/// Between the copy and delete buttons of a row: a dead band, so a press between them does nothing and cannot land on the
/// wrong one.
pub(super) const COPY_DELETE_GAP: f32 = 6.0;
/// Rows the open notch shows at most; a longer list scrolls, with no bar and no hint of the rows beyond.
pub(super) const VISIBLE_ROWS: usize = 5;
/// One wheel notch scrolls one row.
pub(super) const WHEEL_NOTCH_DISTANCE: f32 = ROW_HEIGHT + ROW_GAP;

/// What a row of the list shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Row {
    Item(usize),
    /// The input, the last row while typing.
    Field,
}

/// The rows for the tasks in `order` (indices into the list), then the input while typing.
pub(super) fn plan_rows(order: &[usize], is_adding: bool) -> Vec<Row> {
    let mut rows: Vec<Row> = order.iter().copied().map(Row::Item).collect();
    if is_adding {
        rows.push(Row::Field);
    }
    rows
}

fn rows_height(row_count: usize) -> f32 {
    let rows = row_count as f32;
    rows * ROW_HEIGHT + (rows - 1.0).max(0.0) * ROW_GAP
}

/// The open height: the toolbar, then up to `VISIBLE_ROWS` rows; the usual page height until they need more.
pub(super) fn content_height(row_count: usize) -> f32 {
    (TOOLBAR_HEIGHT + TOOLBAR_GAP + rows_height(row_count.min(VISIBLE_ROWS))).max(DEFAULT_CONTENT_HEIGHT)
}

/// How far the list can scroll: zero while every row is in view.
pub(super) fn max_scroll(row_count: usize) -> f32 {
    (rows_height(row_count) - rows_height(row_count.min(VISIBLE_ROWS))).max(0.0)
}

/// Every row runs the full width, under the toolbar, moved up by `scroll` DIP.
pub(super) fn row_rect(bounds: Rect, position: usize, scroll: f32) -> Rect {
    let top = bounds.top + TOOLBAR_HEIGHT + TOOLBAR_GAP + position as f32 * (ROW_HEIGHT + ROW_GAP) - scroll;
    Rect::new(bounds.left, top, bounds.right, top + ROW_HEIGHT)
}

/// The room under the toolbar: the rows are drawn and scrolled in it, and the empty page says "All clear" in it.
pub(super) fn empty_area(bounds: Rect) -> Rect {
    Rect::new(bounds.left, bounds.top + TOOLBAR_HEIGHT + TOOLBAR_GAP, bounds.right, bounds.bottom)
}

fn button_rect(bounds: Rect, right_inset: f32, width: f32) -> Rect {
    let top = bounds.top + (TOOLBAR_HEIGHT - BUTTON_HEIGHT) / 2.0;
    let right = bounds.right - right_inset;
    Rect::new(right - width, top, right, top + BUTTON_HEIGHT)
}

/// The Add pill, always at the top right.
pub(super) fn add_button(bounds: Rect) -> Rect {
    button_rect(bounds, 0.0, ADD_WIDTH)
}

/// The Clear all pill, left of Add.
pub(super) fn clear_button(bounds: Rect) -> Rect {
    button_rect(bounds, ADD_WIDTH + BUTTON_GAP, CLEAR_WIDTH)
}

/// The Undo pill, always left of Clear all's place, which stays empty while the list is, so a second press on Clear
/// all never lands on Undo.
pub(super) fn undo_button(bounds: Rect) -> Rect {
    button_rect(bounds, ADD_WIDTH + BUTTON_GAP + CLEAR_WIDTH + BUTTON_GAP, UNDO_WIDTH)
}

pub(super) fn check_rect(row: Rect) -> Rect {
    let left = row.left + ROW_PADDING;
    let top = row.center_y() - CHECK_SIZE / 2.0;
    Rect::new(left, top, left + CHECK_SIZE, top + CHECK_SIZE)
}

pub(super) fn delete_rect(row: Rect) -> Rect {
    let right = row.right - DELETE_INSET;
    let top = row.center_y() - DELETE_SIZE / 2.0;
    Rect::new(right - DELETE_SIZE, top, right, top + DELETE_SIZE)
}

/// The copy button, left of the delete button with `COPY_DELETE_GAP` between them.
pub(super) fn copy_rect(row: Rect) -> Rect {
    let delete = delete_rect(row);
    let right = delete.left - COPY_DELETE_GAP;
    Rect { left: right - DELETE_SIZE, right, ..delete }
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
    fn the_list_has_no_limit_and_typing_adds_the_input_after_every_task() {
        let order: Vec<usize> = (0..9).collect();
        assert_eq!(plan_rows(&order, false).len(), 9);
        assert_eq!(plan_rows(&order, true).last(), Some(&Row::Field));
    }

    #[test]
    fn a_long_list_scrolls_by_the_rows_beyond_the_fifth() {
        assert_eq!(max_scroll(5), 0.0);
        assert_eq!(max_scroll(6), ROW_HEIGHT + ROW_GAP);
        assert_eq!(max_scroll(20), 15.0 * (ROW_HEIGHT + ROW_GAP));
    }

    #[test]
    fn the_page_height_is_the_toolbar_plus_the_rows_as_in_the_design() {
        assert_eq!(content_height(0), DEFAULT_CONTENT_HEIGHT);
        assert_eq!(content_height(1), DEFAULT_CONTENT_HEIGHT);
        assert_eq!(content_height(2), 106.0);
        assert_eq!(content_height(3), 144.0);
        assert_eq!(content_height(VISIBLE_ROWS), 220.0);
        assert_eq!(content_height(40), 220.0);
    }

    #[test]
    fn rows_run_the_full_width_below_the_toolbar_with_six_between() {
        let bounds = Rect::new(0.0, 0.0, CONTENT_WIDTH, 144.0);
        let first = row_rect(bounds, 0, 0.0);
        let second = row_rect(bounds, 1, 0.0);
        assert_eq!((first.left, first.right), (0.0, CONTENT_WIDTH));
        assert_eq!(first.top, TOOLBAR_HEIGHT + TOOLBAR_GAP);
        assert_eq!(second.top - first.bottom, ROW_GAP);
        assert_eq!(row_rect(bounds, 2, 0.0).bottom, bounds.bottom);
    }

    #[test]
    fn the_buttons_sit_at_the_top_right_with_clear_all_left_of_add() {
        let bounds = Rect::new(0.0, 0.0, CONTENT_WIDTH, 144.0);
        let (add, clear) = (add_button(bounds), clear_button(bounds));
        assert_eq!(add.right, bounds.right);
        assert_eq!(add.height(), 26.0);
        assert_eq!(add.top, 1.0);
        assert_eq!(add.left - clear.right, BUTTON_GAP);
        assert!(clear.bottom <= row_rect(bounds, 0, 0.0).top);
    }

    #[test]
    fn a_row_has_copy_then_delete_at_its_right_end_with_a_gap_between_them() {
        let row = row_rect(Rect::new(0.0, 0.0, CONTENT_WIDTH, 144.0), 0, 0.0);
        let (copy, delete) = (copy_rect(row), delete_rect(row));
        assert_eq!(delete.right, row.right - DELETE_INSET);
        assert_eq!(delete.left - copy.right, 6.0);
        assert_eq!((copy.width(), copy.height()), (delete.width(), delete.height()));
        assert_eq!(copy.center_y(), row.center_y());
        assert!(copy.left > check_rect(row).right, "the buttons stay clear of the ring");
    }

    #[test]
    fn undo_sits_left_of_clear_all_and_never_overlaps_it() {
        let bounds = Rect::new(0.0, 0.0, CONTENT_WIDTH, 144.0);
        let undo = undo_button(bounds);
        assert_eq!(clear_button(bounds).left - undo.right, BUTTON_GAP);
        assert_eq!((undo.top, undo.height()), (add_button(bounds).top, BUTTON_HEIGHT));
    }

    #[test]
    fn the_empty_page_keeps_its_buttons_on_top_and_its_message_below() {
        let bounds = Rect::new(0.0, 0.0, CONTENT_WIDTH, 86.0);
        assert_eq!(add_button(bounds).right, CONTENT_WIDTH);
        assert_eq!(empty_area(bounds).top, 36.0);
        assert_eq!(empty_area(bounds).bottom, 86.0);
    }
}
