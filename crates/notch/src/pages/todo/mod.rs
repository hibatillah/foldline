//! A short todo list: add a task by typing in the notch, tick it off, copy it, delete it and undo the delete.

mod actions;
mod copy_check;
mod draw;
mod layout;
mod list;
#[cfg(test)]
mod sample;
#[cfg(test)]
mod tests;

use self::{
    copy_check::CopyCheck,
    layout::{
        CONTENT_WIDTH, Row, WHEEL_NOTCH_DISTANCE, add_button, clear_button, content_height, empty_area, max_scroll,
        plan_rows, row_rect, undo_button,
    },
    list::TodoList,
};
use crate::i18n::tr;
use crate::{
    gfx::{Canvas, Point, Rect, Size},
    page::{Context, Frame, KeyInput, Mode, Page, PageSizes, PointerEvent, Priority, TimerId},
    ui::{FieldOutcome, HoverTarget, ScrollOffset, TextField},
};

const RESTING_WIDTH: f32 = 150.0;
const MAX_CHARACTERS: usize = 120;
/// Ends the undo of the last removal.
const UNDO_TIMER: TimerId = TimerId(0);
/// Ends the green check on a copied row.
const COPY_CHECK_TIMER: TimerId = TimerId(1);

/// What the pointer is over: a toolbar button or a row, by its position in the list.
#[derive(Clone, Copy, Debug, PartialEq)]
enum TodoTarget {
    Add,
    ClearAll,
    Undo,
    Row(usize),
}

pub struct TodoPage {
    list: TodoList,
    field: TextField,
    is_adding: bool,
    has_keyboard: bool,
    hover: HoverTarget<TodoTarget>,
    /// The task whose copy button shows the check.
    copy_check: CopyCheck,
    /// The count under the title, rebuilt by `refresh_texts` when the list or the field changes.
    count_text: String,
    /// How far the rows are scrolled up, in DIP.
    scroll: ScrollOffset,
}

impl TodoPage {
    pub fn new() -> Self {
        Self::with_list(TodoList::load())
    }

    fn with_list(list: TodoList) -> Self {
        let mut page = Self {
            list,
            field: TextField::new(MAX_CHARACTERS),
            is_adding: false,
            has_keyboard: false,
            hover: HoverTarget::default(),
            copy_check: CopyCheck::default(),
            count_text: String::new(),
            scroll: ScrollOffset::default(),
        };
        page.refresh_texts();
        page
    }

    fn refresh_texts(&mut self) {
        self.count_text = match (self.list.open_count(), self.list.done_count()) {
            (open, 0) => tr!("todo.left", count = open).into_owned(),
            (open, done) => tr!("todo.left_done", open = open, done = done).into_owned(),
        };
        self.scroll.clamp_to(max_scroll(self.rows().len()));
    }

    /// Shows the last row, where the input is while typing.
    fn scroll_to_end(&mut self) {
        self.scroll.jump_to_end(max_scroll(self.rows().len()));
    }

    /// An empty list that is not being added to shows the "All clear" message under the buttons.
    fn is_empty_view(&self) -> bool {
        self.list.is_empty() && !self.is_adding
    }

    fn rows(&self) -> Vec<Row> {
        plan_rows(&self.list.shown_order(), self.is_adding)
    }

    /// The row under `point`, when the point is in the list area and not under the toolbar.
    fn row_at(&self, bounds: Rect, point: Point) -> Option<(usize, Row)> {
        if !empty_area(bounds).contains(point) {
            return None;
        }
        let scroll = self.scroll.value();
        self.rows().into_iter().enumerate().find(|&(position, _)| row_rect(bounds, position, scroll).contains(point))
    }

    /// What the pointer is over at `point`; the buttons and the list area never overlap.
    fn target_at(&self, bounds: Rect, point: Point) -> Option<TodoTarget> {
        if add_button(bounds).contains(point) {
            return Some(TodoTarget::Add);
        }
        if !self.list.is_empty() && clear_button(bounds).contains(point) {
            return Some(TodoTarget::ClearAll);
        }
        if self.list.can_undo() && undo_button(bounds).contains(point) {
            return Some(TodoTarget::Undo);
        }
        self.row_at(bounds, point)
            .filter(|(_, row)| matches!(row, Row::Item(_)))
            .map(|(position, _)| TodoTarget::Row(position))
    }

    /// Forgets a hovered row and keeps a hovered button.
    fn clear_row_hover(&mut self) {
        if matches!(self.hover.get(), Some(TodoTarget::Row(_))) {
            self.hover.clear();
        }
    }

    fn start_adding(&mut self, context: &mut Context) {
        if self.is_adding {
            return;
        }
        self.is_adding = true;
        if self.hover.is(TodoTarget::Add) {
            self.hover.clear();
        }
        self.field.clear();
        self.refresh_texts();
        self.scroll_to_end();
        context.take_keyboard();
        context.request_repaint();
    }

    fn stop_adding(&mut self, context: &mut Context) {
        if !self.is_adding {
            return;
        }
        self.is_adding = false;
        self.field.clear();
        self.refresh_texts();
        if self.has_keyboard {
            self.has_keyboard = false;
            context.release_keyboard();
        }
        context.request_repaint();
    }
}

impl Page for TodoPage {
    fn id(&self) -> &'static str {
        "todo"
    }

    fn name(&self) -> String {
        crate::i18n::tr!("page.todo").into_owned()
    }

    fn priority(&self, mode: Mode) -> Option<Priority> {
        (mode == Mode::Expanded).then_some(Priority::BACKGROUND)
    }

    fn sizes(&self) -> PageSizes {
        PageSizes {
            resting_width: RESTING_WIDTH,
            expanded: Size::new(CONTENT_WIDTH, content_height(self.rows().len())),
        }
    }

    fn draw_resting(&self, _canvas: &Canvas, _frame: &Frame, _opacity: f32) {}

    fn draw_expanded(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        let bounds = frame.bounds;
        self.draw_toolbar(canvas, bounds, opacity);
        if self.is_empty_view() {
            self.draw_empty(canvas, bounds, opacity);
            return;
        }
        canvas.clipped(empty_area(bounds), || {
            for (position, row) in self.rows().into_iter().enumerate() {
                self.draw_row(canvas, row, row_rect(bounds, position, self.scroll.value()), position, opacity);
            }
        });
    }

    fn on_language_changed(&mut self, context: &mut Context) {
        self.refresh_texts();
        context.request_repaint();
    }

    fn on_pointer(&mut self, event: PointerEvent, frame: &Frame, context: &mut Context) {
        let bounds = frame.bounds;
        match event {
            PointerEvent::Moved(point) => {
                if self.hover.set(self.target_at(bounds, point)) {
                    context.request_repaint();
                }
            }
            PointerEvent::Pressed(point) if frame.is_open() => self.press(bounds, point, context),
            PointerEvent::Scrolled { notches } => {
                if self.scroll.scroll_by(notches * WHEEL_NOTCH_DISTANCE, max_scroll(self.rows().len())) {
                    self.clear_row_hover();
                    context.request_repaint();
                }
            }
            PointerEvent::Left | PointerEvent::CaptureLost if self.hover.clear() => {
                context.request_repaint();
            }
            _ => {}
        }
    }

    fn on_key(&mut self, key: KeyInput, context: &mut Context) {
        if !self.is_adding {
            return;
        }
        self.has_keyboard = true;
        match self.field.apply(&key) {
            FieldOutcome::Submitted => {
                let was_added = self.list.add(self.field.text());
                self.refresh_texts();
                if was_added {
                    self.field.clear();
                    self.scroll_to_end();
                } else {
                    self.stop_adding(context);
                }
            }
            FieldOutcome::Cancelled => self.stop_adding(context),
            FieldOutcome::Changed | FieldOutcome::Unchanged => {}
        }
        context.request_repaint();
    }

    fn claims_scroll(&self, point: Point, frame: &Frame) -> bool {
        max_scroll(self.rows().len()) > 0.0 && empty_area(frame.bounds).contains(point)
    }

    fn on_timer(&mut self, timer: TimerId, context: &mut Context) {
        if timer == UNDO_TIMER {
            self.expire_undo(context);
        } else if timer == COPY_CHECK_TIMER {
            self.expire_copy_check(context);
        }
    }

    fn on_keyboard_lost(&mut self, context: &mut Context) {
        self.has_keyboard = false;
        self.stop_adding(context);
    }

    fn on_shown_changed(&mut self, is_shown: bool, context: &mut Context) {
        if !is_shown {
            self.clear_row_hover();
            self.forget_copy_check(context);
            self.stop_adding(context);
        }
    }
}
