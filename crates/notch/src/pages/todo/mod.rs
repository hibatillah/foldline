//! A short todo list: add a task by typing in the notch, tick it off, delete it.

mod list;

#[cfg(test)]
use self::list::Item;
use self::list::TodoList;
use crate::{
    gfx::{Align, Canvas, Color, FontStyle, Point, Rect, Size, UI_FONT, Weight},
    page::{Context, DEFAULT_CONTENT_HEIGHT, Frame, KeyInput, Mode, Page, PageSizes, PointerEvent, Priority},
    ui::{FieldLook, FieldOutcome, Icon, PillButton, TextField},
};

const CONTENT_WIDTH: f32 = 508.0;
const RESTING_WIDTH: f32 = 150.0;
const SIDE_WIDTH: f32 = 120.0;
const DIVIDER_GAP: f32 = 16.0;
const LIST_LEFT: f32 = SIDE_WIDTH + DIVIDER_GAP * 2.0 + 1.0;
const ROW_HEIGHT: f32 = 32.0;
const ROW_GAP: f32 = 6.0;
const ROW_RADIUS: f32 = 10.0;
const ROW_PADDING: f32 = 10.0;
const CHECK_SIZE: f32 = 14.0;
const TEXT_LEFT: f32 = ROW_PADDING + CHECK_SIZE + 10.0;
const DELETE_SIZE: f32 = 22.0;
/// Rows the open notch shows at most; more appear as a "more" line.
const MAX_ROWS: usize = 4;
const MAX_CHARACTERS: usize = 120;
const ADD_BUTTON: Size = Size::new(64.0, 26.0);

const TITLE: FontStyle = FontStyle::new(UI_FONT, 22.0, Weight::Medium, Align::Leading);
const COUNT: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Regular, Align::Leading);
const ROW_TEXT: FontStyle = FontStyle::new(UI_FONT, 13.0, Weight::Regular, Align::Leading);
const HINT: FontStyle = FontStyle::new(UI_FONT, 10.0, Weight::Regular, Align::Trailing);
const EMPTY: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Regular, Align::Center);

const GRAY: Color = Color::rgb(0.604, 0.604, 0.651);
const DIM: Color = Color::rgb(0.431, 0.431, 0.478);
const ROW: Color = Color::rgb(0.078, 0.078, 0.094);
const ROW_HOVERED: Color = Color::rgb(0.11, 0.11, 0.133);
const DIVIDER: Color = Color::rgb(0.149, 0.149, 0.176);
const DELETE_FILL: Color = Color::rgb(0.165, 0.165, 0.188);
const FIELD_BORDER: Color = Color::rgb(0.251, 0.561, 1.0);
const DONE_GREEN: Color = Color::rgb(0.19, 0.82, 0.35);
const ACCENT: Color = Color::rgb(0.9, 0.9, 0.92);

/// What a row of the list shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Row {
    Field,
    Item(usize),
    More(usize),
}

pub struct TodoPage {
    list: TodoList,
    field: TextField,
    is_adding: bool,
    has_keyboard: bool,
    hovered_row: Option<usize>,
    is_add_hovered: bool,
}

impl TodoPage {
    pub fn new() -> Self {
        Self::with_list(TodoList::load())
    }

    fn with_list(list: TodoList) -> Self {
        Self {
            list,
            field: TextField::new(MAX_CHARACTERS),
            is_adding: false,
            has_keyboard: false,
            hovered_row: None,
            is_add_hovered: false,
        }
    }

    #[cfg(test)]
    pub fn sample() -> Self {
        let item = |text: &str, is_done| Item { text: text.to_owned(), is_done };
        let mut page = Self::with_list(TodoList::with_items(vec![
            item("Check hover on the 1440p monitor", false),
            item("Reply to Rafi about the deploy", true),
            item("Write the v0.5 release notes", false),
            item("Buy a code-signing certificate", false),
        ]));
        page.hovered_row = Some(1);
        page
    }

    fn rows(&self) -> Vec<Row> {
        let order = self.list.shown_order();
        let field = self.is_adding.then_some(Row::Field);
        let room = MAX_ROWS - usize::from(self.is_adding);
        let mut rows: Vec<Row> = field.into_iter().collect();
        if order.len() <= room {
            rows.extend(order.into_iter().map(Row::Item));
        } else {
            rows.extend(order[..room - 1].iter().copied().map(Row::Item));
            rows.push(Row::More(order.len() - (room - 1)));
        }
        rows
    }

    fn row_rect(bounds: Rect, scale: f32, position: usize) -> Rect {
        let top = bounds.top + position as f32 * (ROW_HEIGHT + ROW_GAP) * scale;
        Rect::new(bounds.left + LIST_LEFT * scale, top, bounds.right, top + ROW_HEIGHT * scale)
    }

    fn add_button(bounds: Rect, scale: f32) -> Rect {
        let top = bounds.top + 58.0 * scale;
        Rect::new(bounds.left, top, bounds.left + ADD_BUTTON.width * scale, top + ADD_BUTTON.height * scale)
    }

    fn check_rect(row: Rect, scale: f32) -> Rect {
        let left = row.left + ROW_PADDING * scale;
        let top = row.center_y() - CHECK_SIZE * scale / 2.0;
        Rect::new(left, top, left + CHECK_SIZE * scale, top + CHECK_SIZE * scale)
    }

    fn delete_rect(row: Rect, scale: f32) -> Rect {
        let right = row.right - 5.0 * scale;
        let top = row.center_y() - DELETE_SIZE * scale / 2.0;
        Rect::new(right - DELETE_SIZE * scale, top, right, top + DELETE_SIZE * scale)
    }

    fn start_adding(&mut self, context: &mut Context) {
        self.is_adding = true;
        self.field.clear();
        context.take_keyboard();
        context.request_repaint();
    }

    fn stop_adding(&mut self, context: &mut Context) {
        if !self.is_adding {
            return;
        }
        self.is_adding = false;
        self.field.clear();
        if self.has_keyboard {
            self.has_keyboard = false;
            context.release_keyboard();
        }
        context.request_repaint();
    }

    fn draw_row(&self, canvas: &Canvas, row: Row, rect: Rect, position: usize, opacity: f32) {
        let scale = canvas.scale();
        let text_area = Rect { left: rect.left + TEXT_LEFT * scale, right: rect.right - 36.0 * scale, ..rect };
        match row {
            Row::Field => {
                canvas.fill_rounded_rect(rect, ROW_RADIUS * scale, FIELD_BORDER.faded(opacity));
                let inner = Rect::new(rect.left + scale, rect.top + scale, rect.right - scale, rect.bottom - scale);
                canvas.fill_rounded_rect(inner, (ROW_RADIUS - 1.0) * scale, ROW.faded(opacity));
                canvas.text(&ROW_TEXT, "+", Self::check_rect(rect, scale), FIELD_BORDER.faded(opacity));
                let field_area = Rect { right: rect.right - 80.0 * scale, ..text_area };
                let look = FieldLook { style: &ROW_TEXT, color: Color::WHITE, placeholder: "Add a task" };
                self.field.draw(canvas, field_area, &look, self.has_keyboard, opacity);
                let hint_area =
                    Rect { left: rect.right - 80.0 * scale, right: rect.right - ROW_PADDING * scale, ..rect };
                canvas.text(&HINT, "Enter to add", hint_area, DIM.faded(opacity));
            }
            Row::Item(index) => {
                let Some(item) = self.list.item(index) else { return };
                let is_hovered = self.hovered_row == Some(position);
                let fill = if item.is_done {
                    None
                } else if is_hovered {
                    Some(ROW_HOVERED)
                } else {
                    Some(ROW)
                };
                if let Some(fill) = fill {
                    canvas.fill_rounded_rect(rect, ROW_RADIUS * scale, fill.faded(opacity));
                } else if is_hovered {
                    canvas.fill_rounded_rect(rect, ROW_RADIUS * scale, ROW.faded(opacity));
                }
                let check = Self::check_rect(rect, scale);
                let half = check.width() / 2.0;
                let center = Point::new(check.center_x(), check.center_y());
                if item.is_done {
                    canvas.fill_circle(center, half, DONE_GREEN.faded(opacity));
                    canvas.text(&EMPTY, "\u{2713}", check, Color::BLACK.faded(opacity));
                } else {
                    canvas.fill_circle(center, half, DIM.faded(opacity));
                    canvas.fill_circle(center, half - 1.5 * scale, ROW.faded(opacity));
                }
                let color = if item.is_done { DIM } else { Color::WHITE };
                canvas.text(&ROW_TEXT, &item.text, text_area, color.faded(opacity));
                if item.is_done {
                    let width = (canvas.measure(&ROW_TEXT, &item.text) * scale).min(text_area.width());
                    let line_y = rect.center_y();
                    let line =
                        Rect::new(text_area.left, line_y - 0.5 * scale, text_area.left + width, line_y + 0.5 * scale);
                    canvas.fill_rounded_rect(line, 0.5 * scale, DIM.faded(opacity));
                }
                if is_hovered {
                    let delete = Self::delete_rect(rect, scale);
                    canvas.fill_rounded_rect(delete, delete.height() / 2.0, DELETE_FILL.faded(opacity));
                    let center = Point::new(delete.center_x(), delete.center_y());
                    Icon::Close.draw(canvas, center, 10.0 * scale, Color::rgb(0.769, 0.769, 0.8), opacity);
                }
            }
            Row::More(count) => {
                let label = format!("{count} more");
                canvas.text(&COUNT, &label, Rect { left: rect.left + ROW_PADDING * scale, ..rect }, DIM.faded(opacity));
            }
        }
    }
}

/// The open height: today's height until the rows need more, up to `MAX_ROWS` rows.
fn content_height(row_count: usize) -> f32 {
    let rows = row_count as f32;
    (rows * ROW_HEIGHT + (rows - 1.0).max(0.0) * ROW_GAP).max(DEFAULT_CONTENT_HEIGHT)
}

impl Page for TodoPage {
    fn id(&self) -> &'static str {
        "todo"
    }

    fn name(&self) -> &'static str {
        "Todo"
    }

    fn icon(&self) -> Icon {
        Icon::Checklist
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
        let (bounds, scale) = (frame.bounds, frame.scale);
        let title_area =
            Rect::new(bounds.left, bounds.top, bounds.left + SIDE_WIDTH * scale, bounds.top + 30.0 * scale);
        canvas.text(&TITLE, "Todo", title_area, Color::WHITE.faded(opacity));
        let count = match (self.list.open_count(), self.list.done_count()) {
            (0, 0) => "No tasks yet".to_owned(),
            (open, 0) => format!("{open} left"),
            (open, done) => format!("{open} left · {done} done"),
        };
        let count_area = Rect { top: bounds.top + 32.0 * scale, bottom: bounds.top + 48.0 * scale, ..title_area };
        canvas.text(&COUNT, &count, count_area, GRAY.faded(opacity));
        let add = PillButton {
            label: "+ Add",
            area: Self::add_button(bounds, scale),
            is_primary: false,
            is_destructive: false,
            is_hovered: self.is_add_hovered,
            accent: ACCENT,
        };
        add.draw(canvas, opacity);

        let divider_left = bounds.left + (SIDE_WIDTH + DIVIDER_GAP) * scale;
        let divider = Rect::new(divider_left, bounds.top, divider_left + scale, bounds.bottom);
        canvas.fill_rounded_rect(divider, 0.0, DIVIDER.faded(opacity));

        let rows = self.rows();
        if rows.is_empty() {
            let area = Rect { left: bounds.left + LIST_LEFT * scale, ..bounds };
            canvas.text(
                &EMPTY,
                "Nothing to do. Add a task and it stays here until you tick it off.",
                area,
                DIM.faded(opacity),
            );
            return;
        }
        for (position, row) in rows.into_iter().enumerate() {
            self.draw_row(canvas, row, Self::row_rect(bounds, scale, position), position, opacity);
        }
    }

    fn on_pointer(&mut self, event: PointerEvent, frame: &Frame, context: &mut Context) {
        let (bounds, scale) = (frame.bounds, frame.scale);
        match event {
            PointerEvent::Moved(point) => {
                let rows = self.rows();
                let hovered = (0..rows.len())
                    .find(|&position| Self::row_rect(bounds, scale, position).contains(point))
                    .filter(|&position| matches!(rows[position], Row::Item(_)));
                let is_add_hovered = Self::add_button(bounds, scale).contains(point);
                if hovered != self.hovered_row || is_add_hovered != self.is_add_hovered {
                    self.hovered_row = hovered;
                    self.is_add_hovered = is_add_hovered;
                    context.request_repaint();
                }
            }
            PointerEvent::Pressed(point) if frame.is_open() => {
                if Self::add_button(bounds, scale).contains(point) {
                    if self.is_adding {
                        self.stop_adding(context);
                    } else {
                        self.start_adding(context);
                    }
                    return;
                }
                let rows = self.rows();
                let Some(position) =
                    (0..rows.len()).find(|&position| Self::row_rect(bounds, scale, position).contains(point))
                else {
                    return;
                };
                let Row::Item(index) = rows[position] else { return };
                let row = Self::row_rect(bounds, scale, position);
                if self.hovered_row == Some(position) && Self::delete_rect(row, scale).contains(point) {
                    self.list.remove(index);
                    self.hovered_row = None;
                } else {
                    self.list.toggle(index);
                }
                context.request_repaint();
            }
            PointerEvent::Left | PointerEvent::CaptureLost
                if self.hovered_row.take().is_some() | std::mem::take(&mut self.is_add_hovered) =>
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
                if self.list.add(self.field.text()) {
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
    use super::*;

    fn page_with(open: usize, done: usize) -> TodoPage {
        let items = (0..open)
            .map(|index| Item { text: format!("open {index}"), is_done: false })
            .chain((0..done).map(|index| Item { text: format!("done {index}"), is_done: true }))
            .collect();
        TodoPage::with_list(TodoList::with_items(items))
    }

    #[test]
    fn a_short_list_keeps_the_usual_height_and_a_long_one_grows_to_four_rows() {
        assert_eq!(content_height(0), DEFAULT_CONTENT_HEIGHT);
        assert_eq!(content_height(2), DEFAULT_CONTENT_HEIGHT);
        assert_eq!(content_height(3), 3.0 * ROW_HEIGHT + 2.0 * ROW_GAP);
        assert_eq!(content_height(MAX_ROWS), 4.0 * ROW_HEIGHT + 3.0 * ROW_GAP);
    }

    #[test]
    fn more_items_than_rows_end_with_a_count_of_the_rest() {
        let page = page_with(5, 1);
        assert_eq!(page.rows(), [Row::Item(0), Row::Item(1), Row::Item(2), Row::More(3)]);
    }

    #[test]
    fn the_add_field_takes_the_first_row_while_adding() {
        let mut page = page_with(4, 0);
        page.is_adding = true;
        assert_eq!(page.rows(), [Row::Field, Row::Item(0), Row::Item(1), Row::More(2)]);
        let mut short = page_with(1, 0);
        short.is_adding = true;
        assert_eq!(short.rows(), [Row::Field, Row::Item(0)]);
    }

    #[test]
    fn finished_items_follow_open_ones() {
        let page = page_with(1, 2);
        assert_eq!(page.rows(), [Row::Item(0), Row::Item(1), Row::Item(2)]);
        assert!(page.list.item(1).is_some_and(|item| item.is_done));
    }
}
