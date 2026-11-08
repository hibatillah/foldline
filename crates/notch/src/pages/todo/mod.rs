//! A short todo list: add a task by typing in the notch, tick it off, delete it.

mod draw;
mod layout;
mod list;

#[cfg(test)]
use self::list::Item;
use self::{
    layout::{
        CONTENT_WIDTH, EMPTY_WIDTH, Row, add_button, clear_button, content_height, delete_rect, plan_rows, row_rect,
    },
    list::TodoList,
};
use crate::{
    gfx::{Canvas, Size},
    page::{Context, Frame, KeyInput, Mode, Page, PageSizes, PointerEvent, Priority},
    ui::{FieldOutcome, TextField},
};

const RESTING_WIDTH: f32 = 150.0;
const MAX_CHARACTERS: usize = 120;

pub struct TodoPage {
    list: TodoList,
    field: TextField,
    is_adding: bool,
    has_keyboard: bool,
    hovered_row: Option<usize>,
    is_add_hovered: bool,
    is_clear_hovered: bool,
    /// The count under the title and the "N more" row, rebuilt by `refresh_texts` when the list or the field changes.
    count_text: String,
    more_text: String,
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
            hovered_row: None,
            is_add_hovered: false,
            is_clear_hovered: false,
            count_text: String::new(),
            more_text: String::new(),
        };
        page.refresh_texts();
        page
    }

    fn refresh_texts(&mut self) {
        self.count_text = match (self.list.open_count(), self.list.done_count()) {
            (open, 0) => format!("{open} left"),
            (open, done) => format!("{open} left \u{b7} {done} done"),
        };
        let hidden =
            self.rows().into_iter().find_map(|row| if let Row::More(count) = row { Some(count) } else { None });
        self.more_text = hidden.map_or_else(String::new, |count| format!("{count} more"));
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
        page.hovered_row = (scenario == 0).then_some(1);
        page.refresh_texts();
        page
    }

    /// An empty list that is not being added to shows the narrow page with its button.
    fn is_empty_view(&self) -> bool {
        self.list.is_empty() && !self.is_adding
    }

    fn rows(&self) -> Vec<Row> {
        plan_rows(&self.list.shown_order(), self.is_adding)
    }

    fn row_at(&self, bounds: crate::gfx::Rect, scale: f32, point: crate::gfx::Point) -> Option<(usize, Row)> {
        self.rows().into_iter().enumerate().find(|&(position, _)| row_rect(bounds, scale, position).contains(point))
    }

    fn start_adding(&mut self, context: &mut Context) {
        if self.list.is_full() || self.is_adding {
            return;
        }
        self.is_adding = true;
        self.is_add_hovered = false;
        self.field.clear();
        self.refresh_texts();
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

    fn name(&self) -> &'static str {
        "Todo"
    }

    fn priority(&self, mode: Mode) -> Option<Priority> {
        (mode == Mode::Expanded).then_some(Priority::BACKGROUND)
    }

    fn sizes(&self) -> PageSizes {
        let width = if self.is_empty_view() { EMPTY_WIDTH } else { CONTENT_WIDTH };
        PageSizes { resting_width: RESTING_WIDTH, expanded: Size::new(width, content_height(self.rows().len())) }
    }

    fn draw_resting(&self, _canvas: &Canvas, _frame: &Frame, _opacity: f32) {}

    fn draw_expanded(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        let (bounds, scale) = (frame.bounds, frame.scale);
        self.draw_toolbar(canvas, bounds, scale, opacity);
        if self.is_empty_view() {
            self.draw_empty(canvas, bounds, scale, opacity);
            return;
        }
        for (position, row) in self.rows().into_iter().enumerate() {
            self.draw_row(canvas, row, row_rect(bounds, scale, position), position, opacity);
        }
    }

    fn on_pointer(&mut self, event: PointerEvent, frame: &Frame, context: &mut Context) {
        let (bounds, scale) = (frame.bounds, frame.scale);
        match event {
            PointerEvent::Moved(point) => {
                let is_add_hovered = add_button(bounds, scale).contains(point);
                let is_clear_hovered = !self.list.is_empty() && clear_button(bounds, scale).contains(point);
                let hovered = self
                    .row_at(bounds, scale, point)
                    .filter(|(_, row)| matches!(row, Row::Item(_)))
                    .map(|(position, _)| position);
                if hovered != self.hovered_row
                    || is_add_hovered != self.is_add_hovered
                    || is_clear_hovered != self.is_clear_hovered
                {
                    self.hovered_row = hovered;
                    self.is_add_hovered = is_add_hovered;
                    self.is_clear_hovered = is_clear_hovered;
                    context.request_repaint();
                }
            }
            PointerEvent::Pressed(point) if frame.is_open() => {
                if add_button(bounds, scale).contains(point) {
                    self.start_adding(context);
                    return;
                }
                if !self.list.is_empty() && clear_button(bounds, scale).contains(point) {
                    self.list.clear();
                    self.hovered_row = None;
                    self.refresh_texts();
                    context.request_repaint();
                    return;
                }
                let Some((position, row)) = self.row_at(bounds, scale, point) else { return };
                match row {
                    Row::Item(index) => {
                        let rect = row_rect(bounds, scale, position);
                        if self.hovered_row == Some(position) && delete_rect(rect, scale).contains(point) {
                            self.list.remove(index);
                            self.hovered_row = None;
                        } else {
                            self.list.toggle(index);
                        }
                        self.refresh_texts();
                        context.request_repaint();
                    }
                    Row::Field | Row::More(_) => {}
                }
            }
            PointerEvent::Left | PointerEvent::CaptureLost
                if self.hovered_row.take().is_some()
                    | std::mem::take(&mut self.is_add_hovered)
                    | std::mem::take(&mut self.is_clear_hovered) =>
            {
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
                if was_added && !self.list.is_full() {
                    self.field.clear();
                } else {
                    self.stop_adding(context);
                }
            }
            FieldOutcome::Cancelled => self.stop_adding(context),
            FieldOutcome::Changed | FieldOutcome::Unchanged => {}
        }
        context.request_repaint();
    }

    fn on_keyboard_lost(&mut self, context: &mut Context) {
        self.has_keyboard = false;
        self.stop_adding(context);
    }

    fn on_shown_changed(&mut self, is_shown: bool, context: &mut Context) {
        if !is_shown {
            self.hovered_row = None;
            self.stop_adding(context);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{layout::MAX_ROWS, list::MAX_TASKS, *};

    fn page_with(open: usize, done: usize) -> TodoPage {
        let items = (0..open)
            .map(|index| Item { text: format!("open {index}"), is_done: false })
            .chain((0..done).map(|index| Item { text: format!("done {index}"), is_done: true }))
            .collect();
        TodoPage::with_list(TodoList::with_items(items))
    }

    #[test]
    fn an_empty_list_is_the_narrow_page_until_adding_starts() {
        let mut page = page_with(0, 0);
        assert_eq!(page.sizes().expanded.width, EMPTY_WIDTH);
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
    fn a_sixth_task_is_refused_and_a_done_task_counts_toward_the_five() {
        let mut list =
            TodoList::with_items((0..4).map(|index| Item { text: format!("task {index}"), is_done: false }).collect());
        assert!(list.add("fifth"));
        assert!(list.is_full());
        assert!(!list.add("sixth"));
        assert_eq!(list.len(), MAX_TASKS);
        list.toggle(0);
        assert!(list.is_full(), "a finished task keeps its row until it is cleared");
        list.remove(0);
        assert!(list.add("replacement"));
    }

    #[test]
    fn finished_items_follow_open_ones_and_the_input_comes_last() {
        let mut page = page_with(1, 2);
        assert_eq!(page.rows(), [Row::Item(0), Row::Item(1), Row::Item(2)]);
        page.is_adding = true;
        assert_eq!(page.rows().last(), Some(&Row::Field));
        assert!(page.list.item(1).is_some_and(|item| item.is_done));
        assert!(page.rows().len() <= MAX_ROWS);
    }

    #[test]
    fn clearing_the_list_empties_it_and_the_count_and_returns_to_the_narrow_page() {
        let mut page = page_with(2, 1);
        page.list.clear();
        page.refresh_texts();
        assert_eq!(page.count_text, "0 left");
        assert!(page.is_empty_view());
        assert_eq!(page.sizes().expanded.width, EMPTY_WIDTH);
    }
}
