//! Drawing the week and the event rows.

use super::{
    BLUE, CalendarPage, DAY_LETTER, DAY_LETTER_HEIGHT, DAY_NUMBER, DAY_TOP_PADDING, DIM, EVENT_DETAIL, EVENT_TIME,
    EVENT_TITLE, EVENT_TRAILING, GRAY, HEADER_TOP, JOIN, LEFT_WIDTH, LIGHT, MONTH, PALE_BLUE, ROW, ROW_NEXT,
    SUNDAY_RED, TODAY_FILL, TODAY_SIZE, YEAR, day_cell, row_rect,
};
use super::{
    agenda::{Row, State},
    dates::{MONTH_NAMES, WEEKDAY_LETTERS, civil_from_days, week_start},
};
use crate::gfx::{Canvas, Color, Point, Rect};

pub(super) const DAY_RADIUS: f32 = 5.0;
pub(super) const DAY_WASH_ALPHA: f32 = 0.12;
pub(super) const ROW_RADIUS: f32 = 10.0;
pub(super) const ROW_PADDING: f32 = 10.0;
pub(super) const TIME_WIDTH: f32 = 44.0;
pub(super) const JOIN_WIDTH: f32 = 52.0;
pub(super) const JOIN_HEIGHT: f32 = 22.0;

impl CalendarPage {
    pub(super) fn join_button(&self, position: usize, bounds: Rect, scale: f32) -> Option<Rect> {
        let row = self.agenda.rows.get(position)?;
        if row.state != State::Next || row.meeting.is_none() {
            return None;
        }
        let area = row_rect(position, bounds, scale);
        Some(Rect::new(
            area.right - (ROW_PADDING + JOIN_WIDTH) * scale,
            area.center_y() - JOIN_HEIGHT * scale / 2.0,
            area.right - ROW_PADDING * scale,
            area.center_y() + JOIN_HEIGHT * scale / 2.0,
        ))
    }

    pub(super) fn draw_week(&self, canvas: &Canvas, bounds: Rect, opacity: f32) {
        let scale = canvas.scale();
        let (year, month, _) = civil_from_days(self.today);
        let column = Rect::new(bounds.left, bounds.top, bounds.left + LEFT_WIDTH * scale, bounds.bottom);

        let month_name = MONTH_NAMES[(month as usize - 1) % 12];
        let header = Rect::new(
            column.left,
            bounds.top + HEADER_TOP * scale,
            column.right,
            bounds.top + (HEADER_TOP + 28.0) * scale,
        );
        canvas.text(&MONTH, month_name, header, Color::WHITE.faded(opacity));
        let month_width = canvas.measure(&MONTH, month_name) * scale;
        let year_area =
            Rect::new(header.left + month_width + 8.0 * scale, header.top, header.right, header.bottom - 3.0 * scale);
        canvas.text(
            &YEAR,
            &year.to_string(),
            Rect { top: year_area.bottom - 16.0 * scale, ..year_area },
            GRAY.faded(opacity),
        );

        let first_day = week_start(self.today);
        let selected = self.selected();
        for (position, letter) in WEEKDAY_LETTERS.iter().enumerate() {
            let date = first_day + position as i64;
            let cell = day_cell(position, bounds, scale);
            let is_today = date == self.today;
            if self.hovered_day == Some(position) || date == selected {
                canvas.fill_rounded_rect(cell, DAY_RADIUS * scale, Color::WHITE.with_alpha(DAY_WASH_ALPHA * opacity));
            }
            let letter_color = if is_today { BLUE } else { DIM };
            canvas.text(
                &DAY_LETTER,
                letter,
                Rect::new(
                    cell.left,
                    cell.top + DAY_TOP_PADDING * scale,
                    cell.right,
                    cell.top + (DAY_TOP_PADDING + DAY_LETTER_HEIGHT) * scale,
                ),
                letter_color.faded(opacity),
            );
            let number = Rect { top: cell.bottom - (TODAY_SIZE + 1.0) * scale, bottom: cell.bottom - scale, ..cell };
            let (_, _, date_of_month) = civil_from_days(date);
            if is_today {
                let square = Rect::around(
                    Point::new(number.center_x(), number.center_y()),
                    TODAY_SIZE * scale / 2.0,
                    TODAY_SIZE * scale / 2.0,
                );
                canvas.fill_rounded_rect(square, 6.0 * scale, TODAY_FILL.faded(opacity));
                canvas.text(&DAY_NUMBER, &date_of_month.to_string(), number, Color::WHITE.faded(opacity));
            } else {
                let color = if position == 6 { SUNDAY_RED } else { LIGHT };
                canvas.text(&DAY_NUMBER, &date_of_month.to_string(), number, color.faded(opacity));
            }
        }
    }

    pub(super) fn draw_row(
        &self,
        canvas: &Canvas,
        position: usize,
        row: &Row,
        bounds: Rect,
        hovered_join: bool,
        opacity: f32,
    ) {
        let scale = canvas.scale();
        let area = row_rect(position, bounds, scale);
        let dimmed = if row.state == State::Done { 0.5 } else { 1.0 };
        let fade = opacity * dimmed;
        let fill = if row.state == State::Next { ROW_NEXT } else { ROW };
        canvas.fill_rounded_rect(area, ROW_RADIUS * scale, fill.faded(fade));

        let time_color = if row.state == State::Next { PALE_BLUE } else { GRAY };
        let time_area = Rect::new(
            area.left + ROW_PADDING * scale,
            area.top,
            area.left + (ROW_PADDING + TIME_WIDTH) * scale,
            area.bottom,
        );
        canvas.text(&EVENT_TIME, &row.time, time_area, time_color.faded(fade));

        let text_left = time_area.right;
        let join = self.join_button(position, bounds, scale);
        let text_right = join.map_or(area.right - ROW_PADDING * scale, |button| button.left - 8.0 * scale);
        let trailing_width = if join.is_none() { 54.0 * scale } else { 0.0 };
        let title_right = text_right - trailing_width;
        let title_color = Color::WHITE.faded(fade);
        match &row.detail {
            Some(detail) => {
                let title = Rect::new(text_left, area.top + 4.0 * scale, title_right, area.top + 21.0 * scale);
                let sub = Rect::new(text_left, area.top + 21.0 * scale, title_right, area.bottom - 4.0 * scale);
                canvas.text(&EVENT_TITLE, &row.occurrence.summary, title, title_color);
                canvas.text(&EVENT_DETAIL, detail, sub, GRAY.faded(fade));
            }
            None => canvas.text(
                &EVENT_TITLE,
                &row.occurrence.summary,
                Rect::new(text_left, area.top, title_right, area.bottom),
                title_color,
            ),
        }
        match join {
            Some(button) => {
                let fill = if hovered_join { Color::rgb(0.2, 0.58, 0.9) } else { TODAY_FILL };
                canvas.fill_rounded_rect(button, button.height() / 2.0, fill.faded(fade));
                canvas.text(&JOIN, "Join", button, Color::WHITE.faded(fade));
            }
            None => {
                let trailing = Rect::new(
                    area.right - (ROW_PADDING + 54.0) * scale,
                    area.top,
                    area.right - ROW_PADDING * scale,
                    area.bottom,
                );
                canvas.text(&EVENT_TRAILING, &row.trailing, trailing, GRAY.faded(fade));
            }
        }
    }
}
