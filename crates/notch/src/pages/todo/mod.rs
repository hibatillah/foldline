//! A short todo list: add a task by typing in the notch, tick it off, delete it.

mod draw;
mod layout;
mod list;

#[cfg(test)]
use self::list::Item;
use self::{
    layout::{
        CONTENT_WIDTH, Row, WHEEL_NOTCH_DISTANCE, add_button, clear_button, content_height, delete_rect, empty_area,
        max_scroll, plan_rows, row_rect,
    },
    list::TodoList,
};
use crate::i18n::tr;
use crate::{
    gfx::{Canvas, Point, Rect, Size},
    page::{Context, Frame, KeyInput, Mode, Page, PageSizes, PointerEvent, Priority},
    ui::{FieldOutcome, HoverTarget, ScrollOffset, TextField},
};

const RESTING_WIDTH: f32 = 150.0;
const MAX_CHARACTERS: usize = 120;

/// What the pointer is over: a toolbar button or a row, by its position in the list.
#[derive(Clone, Copy, Debug, PartialEq)]
enum TodoTarget {
    Add,
    ClearAll,
    Row(usize),
}

pub struct TodoPage {
    list: TodoList,
    field: TextField,
    is_adding: bool,
    has_keyboard: bool,
    hover: HoverTarget<TodoTarget>,
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

    #[cfg(test)]
    pub fn sample(scenario: u8) -> Self {
        let item = |text: &str, is_done| Item { text: text.to_owned(), is_done };
        let open = || {
            vec![
                item("Check hover on the 1440p monitor", false),
                item("Write the v0.5 release notes", false),
                item("Buy a code-signing certificate", false),
            ]
        };
        let items = match scenario {
            1 | 2 => Vec::new(),
            3 => [open(), vec![item("Send the invoice to the client", false), item("Book the Jakarta flight", false)]]
                .concat(),
            4 => [open(), vec![item("Send the invoice to the client", false)]].concat(),
            5 => [
                open(),
                vec![item("Send the invoice to the client", true), item("Reply to Rafi about the deploy", true)],
            ]
            .concat(),
            _ => [open(), vec![item("Reply to Rafi about the deploy", true)]].concat(),
        };

        let mut page = Self::with_list(TodoList::with_items(items));
        page.is_adding = matches!(scenario, 2 | 4);
        if scenario == 4 {
            page.field.apply(&KeyInput::Text("Call the notary".to_owned()));
            page.has_keyboard = true;
        }
        page.hover.set((scenario == 0).then_some(TodoTarget::Row(1)));
        page.refresh_texts();
        page
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

    fn uses_units(&self) -> bool {
        true
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
            PointerEvent::Pressed(point) if frame.is_open() => {
                if add_button(bounds).contains(point) {
                    self.start_adding(context);
                    return;
                }
                if !self.list.is_empty() && clear_button(bounds).contains(point) {
                    self.list.clear();
                    self.clear_row_hover();
                    self.scroll.reset();
                    self.refresh_texts();
                    context.request_repaint();
                    return;
                }
                let Some((position, row)) = self.row_at(bounds, point) else { return };
                match row {
                    Row::Item(index) => {
                        let rect = row_rect(bounds, position, self.scroll.value());
                        if self.hover.is(TodoTarget::Row(position)) && delete_rect(rect).contains(point) {
                            self.list.remove(index);
                            self.hover.clear();
                        } else {
                            self.list.toggle(index);
                        }
                        self.refresh_texts();
                        context.request_repaint();
                    }
                    Row::Field => {}
                }
            }
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

    fn on_keyboard_lost(&mut self, context: &mut Context) {
        self.has_keyboard = false;
        self.stop_adding(context);
    }

    fn on_shown_changed(&mut self, is_shown: bool, context: &mut Context) {
        if !is_shown {
            self.clear_row_hover();
            self.stop_adding(context);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{layout::VISIBLE_ROWS, *};

    fn page_with(open: usize, done: usize) -> TodoPage {
        let items = (0..open)
            .map(|index| Item { text: format!("open {index}"), is_done: false })
            .chain((0..done).map(|index| Item { text: format!("done {index}"), is_done: true }))
            .collect();
        TodoPage::with_list(TodoList::with_items(items))
    }

    #[test]
    fn an_empty_list_is_as_wide_as_a_full_one_and_keeps_the_usual_height() {
        let mut page = page_with(0, 0);
        assert_eq!(page.sizes().expanded.width, CONTENT_WIDTH);
        assert_eq!(page.sizes().expanded.height, 86.0);
        page.is_adding = true;
        assert_eq!(page.sizes().expanded.width, CONTENT_WIDTH);
        assert_eq!(page.rows(), [Row::Field]);
    }

    #[test]
    fn the_page_height_follows_the_toolbar_and_the_task_rows() {
        assert_eq!(page_with(3, 0).sizes().expanded.height, 144.0);
        assert_eq!(page_with(5, 0).sizes().expanded.height, 220.0);
        assert_eq!(page_with(2, 0).sizes().expanded.width, CONTENT_WIDTH);
        let mut adding = page_with(4, 0);
        adding.is_adding = true;
        assert_eq!(adding.sizes().expanded.height, 220.0, "the input is a fifth row");
    }

    #[test]
    fn the_list_takes_any_number_of_tasks_and_the_page_stops_growing_at_five_rows() {
        let mut page = page_with(0, 0);
        for index in 0..12 {
            assert!(page.list.add(&format!("task {index}")));
        }
        assert_eq!(page.list.len(), 12);
        assert_eq!(page.sizes().expanded.height, 220.0);
    }

    #[test]
    fn scrolling_stops_at_the_last_row_and_a_cleared_list_returns_to_the_top() {
        let mut page = page_with(8, 0);
        page.scroll.jump_to_end(max_scroll(page.rows().len()));
        assert_eq!(page.scroll.value(), 3.0 * WHEEL_NOTCH_DISTANCE);
        page.list.clear();
        page.refresh_texts();
        assert_eq!(page.scroll.value(), 0.0);
    }

    #[test]
    fn finished_items_follow_open_ones_and_the_input_comes_last() {
        let mut page = page_with(1, 2);
        assert_eq!(page.rows(), [Row::Item(0), Row::Item(1), Row::Item(2)]);
        page.is_adding = true;
        assert_eq!(page.rows().last(), Some(&Row::Field));
        assert!(page.list.item(1).is_some_and(|item| item.is_done));
        assert!(page.rows().len() <= VISIBLE_ROWS);
    }

    #[test]
    fn clearing_the_list_empties_it_and_the_count_and_shows_the_empty_message() {
        let mut page = page_with(2, 1);
        page.list.clear();
        page.refresh_texts();
        assert_eq!(page.count_text, "0 left");
        assert!(page.is_empty_view());
        assert_eq!(page.sizes().expanded.width, CONTENT_WIDTH);
    }

    #[test]
    fn moving_from_a_row_to_the_add_button_repaints_once() {
        let mut page = page_with(3, 0);
        let bounds = Rect::new(0.0, 0.0, CONTENT_WIDTH, 144.0);
        let center = |rect: Rect| Point::new(rect.center_x(), rect.center_y());
        let on_row = center(row_rect(bounds, 1, 0.0));
        let on_add = center(add_button(bounds));
        assert!(page.hover.set(page.target_at(bounds, on_row)), "entering the row repaints");
        assert_eq!(page.hover.get(), Some(TodoTarget::Row(1)));
        assert!(page.hover.set(page.target_at(bounds, on_add)), "the move to Add repaints");
        assert_eq!(page.hover.get(), Some(TodoTarget::Add));
        assert!(!page.hover.set(page.target_at(bounds, on_add)), "staying on Add does not repaint");
    }

    #[test]
    fn the_todo_count_reads_in_indonesian() {
        store::language::with_override(store::language::Language::Indonesian, || {
            let mut page = page_with(2, 1);
            page.refresh_texts();
            assert_eq!(page.count_text, "2 tersisa \u{b7} 1 selesai");
            let mut open_only = page_with(3, 0);
            open_only.refresh_texts();
            assert_eq!(open_only.count_text, "3 tersisa");
        });
    }
}
