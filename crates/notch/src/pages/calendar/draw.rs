use super::{
    BLUE, CalendarPage, DAY_LETTER, DAY_NUMBER, DIM, DONE_GAP, DONE_ICON_SIZE, EMPTY_DETAIL, EMPTY_GAP, EMPTY_HINT,
    EMPTY_ICON_SIZE, EMPTY_TITLE, EVENT_TIME, EVENT_TITLE, GRAY, LIGHT, MESSAGE_DETAIL, MESSAGE_LEADING, MONTH, ROW,
    ROW_BORDER, SUNDAY_RED, TODAY_BLUE, TODAY_NUMBER, YEAR,
    geometry::{CELL_WIDTH, day_cell, day_circle, events_area, header, row_rect, weekday_row},
};
use super::{
    agenda::Row,
    dates::{MONTH_NAMES, WEEKDAY_LETTERS, civil_from_days},
};
use crate::{
    gfx::{Canvas, Color, Point, Rect},
    ui::Icon,
};

pub(super) const SELECTED_CIRCLE_ALPHA: f32 = 0.28;
pub(super) const HOVER_WASH_ALPHA: f32 = 0.06;
pub(super) const ROW_RADIUS: f32 = 8.0;
pub(super) const ROW_PADDING: f32 = 14.0;
pub(super) const TIME_WIDTH: f32 = 52.0;
pub(super) const TIME_GAP: f32 = 14.0;
/// The year sits on the month's baseline, 8 px after it.
const YEAR_GAP: f32 = 8.0;
/// How far the year's line ends above the month line's bottom so both share a baseline (measured on a snapshot).
const YEAR_LIFT: f32 = 1.0;
const YEAR_HEIGHT: f32 = 16.0;
const TITLE_LINE_HEIGHT: f32 = 17.0;
const DETAIL_LINE_HEIGHT: f32 = 15.0;
const HINT_LINE_HEIGHT: f32 = 14.0;
const HINT_MARGIN: f32 = 8.0;
/// Between the title and the hint under it; the icon keeps `EMPTY_GAP`.
const HINT_GAP: f32 = 3.0;
const NO_LINK_HINT_LINES: f32 = 2.0;
const NO_LINK_TITLE: &str = "No calendar linked";
const NO_LINK_HINT: &str = "Paste your calendar's iCal address in Settings, under Calendar.";
const DONE_TITLE: &str = "Done for today";
const DONE_DETAIL: &str = "No more events";
const EMPTY_DAY_DETAIL: &str = "Events from your calendar show here.";

impl CalendarPage {
    pub(super) fn draw_month(&self, canvas: &Canvas, bounds: Rect, opacity: f32) {
        let scale = canvas.scale();
        let (_, month_number, _) = civil_from_days(self.today);
        let rows = self.grid.rows();
        let header = header(bounds, scale, rows);
        let month_name = MONTH_NAMES[(month_number as usize - 1) % 12];
        canvas.text(&MONTH, month_name, header, Color::WHITE.faded(opacity));
        let month_width = canvas.measure(&MONTH, month_name) * scale;
        let year_bottom = header.bottom - YEAR_LIFT * scale;
        let year_area = Rect::new(
            header.left + month_width + YEAR_GAP * scale,
            year_bottom - YEAR_HEIGHT * scale,
            header.right,
            year_bottom,
        );
        canvas.text(&YEAR, &self.labels.year, year_area, GRAY.faded(opacity));

        let letters = weekday_row(bounds, scale, rows);
        for (column, letter) in WEEKDAY_LETTERS.iter().enumerate() {
            let left = letters.left + column as f32 * CELL_WIDTH * scale;
            let area = Rect::new(left, letters.top, left + CELL_WIDTH * scale, letters.bottom);
            canvas.text(&DAY_LETTER, letter, area, GRAY.faded(opacity));
        }

        let selected = self.selected();
        for date_index in 0..self.grid.day_count as usize {
            let cell_index = self.grid.leading + date_index;
            let day = self.grid.first_day + date_index as i64;
            let circle = day_circle(day_cell(cell_index, rows, bounds, scale), scale);
            let center = Point::new(circle.center_x(), circle.center_y());
            let radius = circle.width() / 2.0;
            let is_today = day == self.today;
            // Today's solid circle covers a selection of the same day.
            if is_today {
                canvas.fill_circle(center, radius, TODAY_BLUE.faded(opacity));
            } else if day == selected {
                canvas.fill_circle(center, radius, BLUE.with_alpha(SELECTED_CIRCLE_ALPHA * opacity));
            } else if self.hovered_cell == Some(cell_index) {
                canvas.fill_circle(center, radius, Color::WHITE.with_alpha(HOVER_WASH_ALPHA * opacity));
            }
            let Some(number) = self.labels.day_numbers.get(date_index) else { continue };
            if is_today {
                canvas.text(&TODAY_NUMBER, number, circle, Color::WHITE.faded(opacity));
            } else {
                let color = if cell_index % 7 == 6 { SUNDAY_RED } else { Color::WHITE };
                canvas.text(&DAY_NUMBER, number, circle, color.faded(opacity));
            }
        }
    }

    /// The right column when no event shows: done for today, an empty day or no linked calendar.
    pub(super) fn draw_events_message(&self, canvas: &Canvas, area: Rect, opacity: f32) {
        let scale = canvas.scale();
        if self.link.is_some() && self.agenda.is_done {
            let title_width = canvas.measure(&MESSAGE_LEADING, DONE_TITLE);
            let detail_width = canvas.measure(&MESSAGE_DETAIL, DONE_DETAIL);
            let total = DONE_ICON_SIZE + DONE_GAP + title_width + DONE_GAP + detail_width;
            let left = area.center_x() - total * scale / 2.0;
            let icon = Point::new(left + DONE_ICON_SIZE * scale / 2.0, area.center_y());
            Icon::CircleCheck.draw(canvas, icon, DONE_ICON_SIZE * scale, DIM, opacity);
            let title_left = left + (DONE_ICON_SIZE + DONE_GAP) * scale;
            let title_right = title_left + title_width * scale + 2.0 * scale;
            let line = |left: f32, right: f32| Rect::new(left, area.top, right, area.bottom);
            canvas.text(&MESSAGE_LEADING, DONE_TITLE, line(title_left, title_right), LIGHT.faded(opacity));
            let detail_left = title_left + (title_width + DONE_GAP) * scale;
            canvas.text(&MESSAGE_DETAIL, DONE_DETAIL, line(detail_left, area.right), DIM.faded(opacity));
            return;
        }
        // An empty day is the icon, a title and a one-line detail; a missing link has a hint of two lines instead.
        let has_link = self.link.is_some();
        let title = if has_link { self.labels.empty_title.as_str() } else { NO_LINK_TITLE };
        let below_title =
            if has_link { EMPTY_GAP + DETAIL_LINE_HEIGHT } else { HINT_GAP + NO_LINK_HINT_LINES * HINT_LINE_HEIGHT };
        let total = EMPTY_ICON_SIZE + EMPTY_GAP + TITLE_LINE_HEIGHT + below_title;
        let top = area.center_y() - total * scale / 2.0;
        let icon_size = EMPTY_ICON_SIZE * scale;
        Icon::Calendar.draw(canvas, Point::new(area.center_x(), top + icon_size / 2.0), icon_size, GRAY, opacity);
        let title_top = top + (EMPTY_ICON_SIZE + EMPTY_GAP) * scale;
        let title_area = Rect::new(area.left, title_top, area.right, title_top + TITLE_LINE_HEIGHT * scale);
        canvas.text(&EMPTY_TITLE, title, title_area, Color::WHITE.faded(opacity));
        if has_link {
            let detail_top = title_area.bottom + EMPTY_GAP * scale;
            let detail_area = Rect::new(area.left, detail_top, area.right, detail_top + DETAIL_LINE_HEIGHT * scale);
            canvas.text(&EMPTY_DETAIL, EMPTY_DAY_DETAIL, detail_area, GRAY.faded(opacity));
        } else {
            let hint_area = Rect::new(
                area.left + HINT_MARGIN * scale,
                title_area.bottom + HINT_GAP * scale,
                area.right - HINT_MARGIN * scale,
                title_area.bottom + (HINT_GAP + NO_LINK_HINT_LINES * HINT_LINE_HEIGHT) * scale,
            );
            canvas.text(&EMPTY_HINT, NO_LINK_HINT, hint_area, DIM.faded(opacity));
        }
    }

    /// The cards, clipped to the viewport so a scrolled card is cut at its edge.
    pub(super) fn draw_rows(&self, canvas: &Canvas, bounds: Rect, opacity: f32) {
        let scale = canvas.scale();
        let viewport = events_area(bounds, scale, self.view);
        canvas.clipped(viewport, || {
            for (position, row) in self.agenda.rows.iter().enumerate() {
                let area = row_rect(position, self.scroll, bounds, scale, self.view);
                if area.bottom > viewport.top && area.top < viewport.bottom {
                    self.draw_row(canvas, area, row, opacity);
                }
            }
        });
    }

    fn draw_row(&self, canvas: &Canvas, area: Rect, row: &Row, opacity: f32) {
        let scale = canvas.scale();
        canvas.fill_rounded_rect(area, ROW_RADIUS * scale, ROW_BORDER.faded(opacity));
        let inner = Rect::new(area.left + scale, area.top + scale, area.right - scale, area.bottom - scale);
        canvas.fill_rounded_rect(inner, (ROW_RADIUS - 1.0) * scale, ROW.faded(opacity));

        let time_left = area.left + ROW_PADDING * scale;
        let time_area = Rect::new(time_left, area.top, time_left + TIME_WIDTH * scale, area.bottom);
        canvas.text(&EVENT_TIME, &row.time, time_area, GRAY.faded(opacity));
        let title_area =
            Rect::new(time_area.right + TIME_GAP * scale, area.top, area.right - ROW_PADDING * scale, area.bottom);
        canvas.text(&EVENT_TITLE, &row.occurrence.summary, title_area, Color::WHITE.faded(opacity));
    }
}
