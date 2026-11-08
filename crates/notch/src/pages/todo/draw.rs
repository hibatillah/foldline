//! How the todo page looks: the toolbar, the rows and the empty page.

use super::{
    TodoPage,
    layout::{
        BUTTON_HEIGHT, CHECK_SIZE, ROW_PADDING, Row, TEXT_LEFT, add_button, check_rect, clear_button, delete_rect,
        empty_area,
    },
};
use crate::{
    gfx::{Align, Canvas, Color, FontStyle, Point, Rect, UI_FONT, Weight},
    ui::{FieldLook, Stroke, draw_strokes},
};

const ROW_RADIUS: f32 = 10.0;
const FIELD_BORDER_WIDTH: f32 = 1.0;
const HINT_WIDTH: f32 = 80.0;
const TITLE_HEIGHT: f32 = 24.0;
const COUNT_HEIGHT: f32 = 14.0;
const TITLE_GAP: f32 = 8.0;
/// The count sits this far below the title's vertical middle so both share a baseline (about 0.35 of a font size each).
const COUNT_DROP: f32 = 2.8;
const BUTTON_CONTENT_GAP: f32 = 5.0;
const TRASH_SIZE: f32 = 12.0;
const MESSAGE_ICON_SIZE: f32 = 18.0;
const MESSAGE_ICON_GAP: f32 = 8.0;
const MESSAGE_HEIGHT: f32 = 18.0;

const TITLE: FontStyle = FontStyle::new(UI_FONT, 20.0, Weight::Medium, Align::Leading);
const COUNT: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Regular, Align::Leading);
const ROW_TEXT: FontStyle = FontStyle::new(UI_FONT, 13.0, Weight::Regular, Align::Leading);
const PLUS: FontStyle = FontStyle::new(UI_FONT, 15.0, Weight::Regular, Align::Center);
const ADD_PLUS: FontStyle = FontStyle::new(UI_FONT, 14.0, Weight::Regular, Align::Leading);
const HINT: FontStyle = FontStyle::new(UI_FONT, 10.0, Weight::Regular, Align::Trailing);
const MESSAGE: FontStyle = FontStyle::new(UI_FONT, 13.0, Weight::Medium, Align::Leading);
/// The design asks for 600; only Regular and Medium are bundled.
const BUTTON: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Medium, Align::Leading);

const GRAY: Color = Color::rgb(0.604, 0.604, 0.651);
const DIM: Color = Color::rgb(0.431, 0.431, 0.478);
const ROW: Color = Color::rgb(0.078, 0.078, 0.094);
const ROW_HOVERED: Color = Color::rgb(0.11, 0.11, 0.133);
const DELETE_FILL: Color = Color::rgb(0.165, 0.165, 0.188);
const DELETE_CROSS: Color = Color::rgb(0.769, 0.769, 0.8);
const FIELD_BORDER: Color = Color::rgb(0.251, 0.561, 1.0);
const DONE_GREEN: Color = Color::rgb(0.19, 0.82, 0.35);
const MESSAGE_COLOR: Color = Color::rgb(0.769, 0.769, 0.8);
const CLEAR_FILL: Color = Color::rgb(0.11, 0.11, 0.133);
const CLEAR_FILL_HOVERED: Color = Color::rgb(0.149, 0.149, 0.176);
const CLEAR_BORDER: Color = Color::rgb(0.173, 0.173, 0.204);
const CLEAR_TEXT: Color = Color::rgb(0.902, 0.902, 0.922);
const TRASH_RED: Color = Color::rgb(1.0, 0.42, 0.42);
const ADD_FILL_DIMMED: Color = Color::rgb(0.149, 0.149, 0.176);
const ADD_TEXT_DIMMED: Color = Color::rgb(0.431, 0.431, 0.478);
const CLOSE: &str = "M6 6l12 12M18 6 6 18";
const CHECK: &str = "m5 12 5 5 9-10";
const TRASH: &str = "M4 7h16M10 11v6M14 11v6M6 7l1 13h10l1-13M9 7V4h6v3";
const ALL_CLEAR_RING: &str = "M3 12a9 9 0 1 0 18 0a9 9 0 1 0-18 0";
const ALL_CLEAR_CHECK: &str = "m8 12.5 2.8 2.8L16 9.5";

impl TodoPage {
    /// The title and count at the top left, level on one baseline; Clear all and Add at the top right.
    pub(super) fn draw_toolbar(&self, canvas: &Canvas, bounds: Rect, scale: f32, opacity: f32) {
        let middle = bounds.top + super::layout::TOOLBAR_HEIGHT * scale / 2.0;
        let title_area = Rect::new(
            bounds.left,
            middle - TITLE_HEIGHT * scale / 2.0,
            bounds.right,
            middle + TITLE_HEIGHT * scale / 2.0,
        );
        canvas.text(&TITLE, "Todo", title_area, Color::WHITE.faded(opacity));
        let count_left = bounds.left + (canvas.measure(&TITLE, "Todo") + TITLE_GAP) * scale;
        let count_middle = middle + COUNT_DROP * scale;
        let count_area = Rect::new(
            count_left,
            count_middle - COUNT_HEIGHT * scale / 2.0,
            bounds.right,
            count_middle + COUNT_HEIGHT * scale / 2.0,
        );
        canvas.text(&COUNT, &self.count_text, count_area, GRAY.faded(opacity));
        if !self.list.is_empty() {
            self.draw_clear_button(canvas, clear_button(bounds, scale), scale, opacity);
        }
        self.draw_add_button(canvas, add_button(bounds, scale), scale, opacity);
    }

    fn draw_clear_button(&self, canvas: &Canvas, area: Rect, scale: f32, opacity: f32) {
        let radius = BUTTON_HEIGHT * scale / 2.0;
        let fill = if self.is_clear_hovered { CLEAR_FILL_HOVERED } else { CLEAR_FILL };
        canvas.fill_rounded_rect(area, radius, CLEAR_BORDER.faded(opacity));
        let inner = Rect::new(area.left + scale, area.top + scale, area.right - scale, area.bottom - scale);
        canvas.fill_rounded_rect(inner, radius - scale, fill.faded(opacity));
        let text_width = canvas.measure(&BUTTON, "Clear all") * scale;
        let content = (TRASH_SIZE + BUTTON_CONTENT_GAP) * scale + text_width;
        let left = area.center_x() - content / 2.0;
        let icon_center = Point::new(left + TRASH_SIZE * scale / 2.0, area.center_y());
        draw_strokes(canvas, &[Stroke::new(TRASH, TRASH_RED)], icon_center, TRASH_SIZE * scale, 2.0, opacity);
        let label = Rect { left: left + (TRASH_SIZE + BUTTON_CONTENT_GAP) * scale, right: area.right, ..area };
        canvas.text(&BUTTON, "Clear all", label, CLEAR_TEXT.faded(opacity));
    }

    fn draw_add_button(&self, canvas: &Canvas, area: Rect, scale: f32, opacity: f32) {
        let is_dimmed = self.list.is_full();
        let fill = match (is_dimmed, self.is_add_hovered) {
            (true, _) => ADD_FILL_DIMMED,
            (false, true) => Color::gray(0.85),
            (false, false) => Color::WHITE,
        };
        let text = if is_dimmed { ADD_TEXT_DIMMED } else { Color::BLACK };
        canvas.fill_rounded_rect(area, area.height() / 2.0, fill.faded(opacity));
        let plus_width = canvas.measure(&ADD_PLUS, "+") * scale;
        let content = plus_width + BUTTON_CONTENT_GAP * scale + canvas.measure(&BUTTON, "Add") * scale;
        let left = area.center_x() - content / 2.0;
        canvas.text(&ADD_PLUS, "+", Rect { left, right: left + plus_width + scale, ..area }, text.faded(opacity));
        let label = Rect { left: left + plus_width + BUTTON_CONTENT_GAP * scale, right: area.right, ..area };
        canvas.text(&BUTTON, "Add", label, text.faded(opacity));
    }

    /// "All clear" with its check mark, centered under the toolbar.
    pub(super) fn draw_empty(&self, canvas: &Canvas, bounds: Rect, scale: f32, opacity: f32) {
        let area = empty_area(bounds, scale);
        let text_width = canvas.measure(&MESSAGE, "All clear") * scale;
        let total = (MESSAGE_ICON_SIZE + MESSAGE_ICON_GAP) * scale + text_width;
        let left = area.center_x() - total / 2.0;
        let icon_center = Point::new(left + MESSAGE_ICON_SIZE * scale / 2.0, area.center_y());
        let layers = [Stroke::new(ALL_CLEAR_RING, DIM), Stroke::new(ALL_CLEAR_CHECK, DIM)];
        draw_strokes(canvas, &layers, icon_center, MESSAGE_ICON_SIZE * scale, 1.6, opacity);
        let text_area = Rect::new(
            left + (MESSAGE_ICON_SIZE + MESSAGE_ICON_GAP) * scale,
            area.center_y() - MESSAGE_HEIGHT * scale / 2.0,
            area.right,
            area.center_y() + MESSAGE_HEIGHT * scale / 2.0,
        );
        canvas.text(&MESSAGE, "All clear", text_area, MESSAGE_COLOR.faded(opacity));
    }

    pub(super) fn draw_row(&self, canvas: &Canvas, row: Row, rect: Rect, position: usize, opacity: f32) {
        let scale = canvas.scale();
        let text_area = Rect { left: rect.left + TEXT_LEFT * scale, right: rect.right - 36.0 * scale, ..rect };
        let is_hovered = self.hovered_row == Some(position);
        let fill = if is_hovered { ROW_HOVERED } else { ROW };
        match row {
            Row::Field => {
                canvas.fill_rounded_rect(rect, ROW_RADIUS * scale, FIELD_BORDER.faded(opacity));
                let border = FIELD_BORDER_WIDTH * scale;
                let inner = Rect::new(rect.left + border, rect.top + border, rect.right - border, rect.bottom - border);
                canvas.fill_rounded_rect(inner, (ROW_RADIUS - FIELD_BORDER_WIDTH) * scale, ROW.faded(opacity));
                canvas.text(&PLUS, "+", check_rect(rect, scale), FIELD_BORDER.faded(opacity));
                let field_area = Rect { right: rect.right - (HINT_WIDTH + ROW_PADDING) * scale, ..text_area };
                let look = FieldLook { style: &ROW_TEXT, color: Color::WHITE, placeholder: "What needs doing?" };
                self.field.draw(canvas, field_area, &look, self.has_keyboard, opacity);
                let hint_area =
                    Rect { left: rect.right - HINT_WIDTH * scale, right: rect.right - ROW_PADDING * scale, ..rect };
                canvas.text(&HINT, "Enter to add", hint_area, DIM.faded(opacity));
            }
            Row::Item(index) => {
                let Some(item) = self.list.item(index) else { return };
                if !item.is_done || is_hovered {
                    canvas.fill_rounded_rect(rect, ROW_RADIUS * scale, fill.faded(opacity));
                }
                let check = check_rect(rect, scale);
                let center = Point::new(check.center_x(), check.center_y());
                let half = CHECK_SIZE * scale / 2.0;
                if item.is_done {
                    canvas.fill_circle(center, half, DONE_GREEN.faded(opacity));
                    let mark = [Stroke::new(CHECK, Color::BLACK)];
                    draw_strokes(canvas, &mark, center, 9.0 * scale, 3.5, opacity);
                } else {
                    canvas.fill_circle(center, half, DIM.faded(opacity));
                    canvas.fill_circle(center, half - 1.5 * scale, fill.faded(opacity));
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
                    let delete = delete_rect(rect, scale);
                    canvas.fill_rounded_rect(delete, delete.height() / 2.0, DELETE_FILL.faded(opacity));
                    let center = Point::new(delete.center_x(), delete.center_y());
                    let cross = [Stroke::new(CLOSE, DELETE_CROSS)];
                    draw_strokes(canvas, &cross, center, 10.0 * scale, 2.5, opacity);
                }
            }
            Row::More(_) => {
                canvas.text(
                    &COUNT,
                    &self.more_text,
                    Rect { left: rect.left + ROW_PADDING * scale, ..rect },
                    DIM.faded(opacity),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::todo::list::{Item, TodoList};

    fn page(open: usize, done: usize) -> TodoPage {
        let items = (0..open)
            .map(|index| Item { text: format!("open {index}"), is_done: false })
            .chain((0..done).map(|index| Item { text: format!("done {index}"), is_done: true }))
            .collect();
        TodoPage::with_list(TodoList::with_items(items))
    }

    #[test]
    fn the_count_reads_left_and_done() {
        assert_eq!(page(0, 0).count_text, "0 left");
        assert_eq!(page(3, 0).count_text, "3 left");
        assert_eq!(page(3, 2).count_text, "3 left \u{b7} 2 done");
    }

    #[test]
    fn the_cached_texts_follow_the_list_and_the_overflow_row() {
        let mut crowded = page(7, 0);
        assert!(crowded.more_text.ends_with(" more"), "{}", crowded.more_text);
        crowded.list.toggle(0);
        crowded.refresh_texts();
        assert_eq!(crowded.count_text, "6 left \u{b7} 1 done");
        assert!(page(1, 0).more_text.is_empty());
    }
}
