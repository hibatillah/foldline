//! What a press does: add, clear, undo, and on a hovered row copy, delete or tick off.

use super::{
    TodoPage, TodoTarget, UNDO_TIMER,
    layout::{Row, copy_rect, delete_rect, row_rect},
};
use crate::{
    gfx::{Point, Rect},
    page::Context,
    platform::clipboard,
};

/// How long Undo stays after a removal, as on the clipboard page.
const UNDO_MILLISECONDS: u32 = 5000;

/// What a press on a task row does, from where it lands; the row's buttons answer only while it is hovered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RowPress {
    Copy,
    Delete,
    Toggle,
    /// The gap between the two buttons, so a press that misses both does not tick the task off either.
    Nothing,
}

pub(super) fn row_press(row: Rect, point: Point, is_hovered: bool) -> RowPress {
    let (copy, delete) = (copy_rect(row), delete_rect(row));
    let is_between = point.x > copy.right && point.x < delete.left && point.y >= copy.top && point.y <= copy.bottom;
    match (is_hovered, delete.contains(point), copy.contains(point)) {
        (true, true, _) => RowPress::Delete,
        (true, false, true) => RowPress::Copy,
        (true, false, false) if is_between => RowPress::Nothing,
        _ => RowPress::Toggle,
    }
}

impl TodoPage {
    /// Acts on what `target_at` finds, so a press lands only on a button that is shown.
    pub(super) fn press(&mut self, bounds: Rect, point: Point, context: &mut Context) {
        match self.target_at(bounds, point) {
            Some(TodoTarget::Add) => self.start_adding(context),
            Some(TodoTarget::ClearAll) => {
                self.clear_tasks(bounds, point);
                self.after_removal(context);
            }
            Some(TodoTarget::Undo) => {
                context.kill_timer(UNDO_TIMER);
                if self.undo_removal(bounds, point) {
                    context.request_repaint();
                }
            }
            Some(TodoTarget::Row(_)) => self.press_row(bounds, point, context),
            None => {}
        }
    }

    /// Removes every task; the hover is found again, since Clear all is gone and Undo has appeared.
    pub(super) fn clear_tasks(&mut self, bounds: Rect, point: Point) {
        self.list.clear();
        self.scroll.reset();
        self.refresh_texts();
        self.hover.set(self.target_at(bounds, point));
    }

    /// Puts the last removal back; the hover is found again, since Undo is gone.
    pub(super) fn undo_removal(&mut self, bounds: Rect, point: Point) -> bool {
        if !self.list.undo() {
            return false;
        }
        self.refresh_texts();
        self.hover.set(self.target_at(bounds, point));
        true
    }

    fn press_row(&mut self, bounds: Rect, point: Point, context: &mut Context) {
        let Some((position, Row::Item(index))) = self.row_at(bounds, point) else { return };
        let rect = row_rect(bounds, position, self.scroll.value());
        match row_press(rect, point, self.hover.is(TodoTarget::Row(position))) {
            RowPress::Copy => {
                if let Some(item) = self.list.item(index) {
                    clipboard::write_text(context.window(), &item.text);
                }
            }
            RowPress::Delete => {
                self.list.remove(index);
                self.hover.clear();
                self.after_removal(context);
            }
            RowPress::Toggle => {
                self.list.toggle(index);
                self.refresh_texts();
                context.request_repaint();
            }
            RowPress::Nothing => {}
        }
    }

    /// Shows Undo for the removal just made; a removal before it can no longer be undone.
    fn after_removal(&mut self, context: &mut Context) {
        self.refresh_texts();
        context.set_timer(UNDO_TIMER, UNDO_MILLISECONDS);
        context.request_repaint();
    }

    pub(super) fn expire_undo(&mut self, context: &mut Context) {
        context.kill_timer(UNDO_TIMER);
        self.list.forget_removed();
        if self.hover.is(TodoTarget::Undo) {
            self.hover.clear();
        }
        context.request_repaint();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::todo::layout::CONTENT_WIDTH;

    fn row() -> Rect {
        row_rect(Rect::new(0.0, 0.0, CONTENT_WIDTH, 144.0), 0, 0.0)
    }

    fn center(rect: Rect) -> Point {
        Point::new(rect.center_x(), rect.center_y())
    }

    #[test]
    fn on_a_hovered_row_the_buttons_copy_and_delete_and_the_rest_ticks_off() {
        let row = row();
        assert_eq!(row_press(row, center(copy_rect(row)), true), RowPress::Copy);
        assert_eq!(row_press(row, center(delete_rect(row)), true), RowPress::Delete);
        assert_eq!(row_press(row, center(row), true), RowPress::Toggle);
    }

    #[test]
    fn a_press_in_the_gap_between_copy_and_delete_does_nothing() {
        let row = row();
        let gap = Point::new((copy_rect(row).right + delete_rect(row).left) / 2.0, row.center_y());
        assert_eq!(row_press(row, gap, true), RowPress::Nothing);
    }

    #[test]
    fn a_row_that_is_not_hovered_has_no_buttons_to_press() {
        let row = row();
        assert_eq!(row_press(row, center(copy_rect(row)), false), RowPress::Toggle);
        assert_eq!(row_press(row, center(delete_rect(row)), false), RowPress::Toggle);
    }
}
