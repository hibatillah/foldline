//! Where the weather page puts its left column, its six cells and its messages, and how they look.

use salah::{Duration, Local};

use super::{
    WeatherPage,
    forecast::{Forecast, rain_line},
    message::{Message, MessageKind},
    picture::draw_symbol,
};
use crate::{
    gfx::{Align, Canvas, Color, FontStyle, Point, Rect, UI_FONT, Weight},
    ui::{Stroke, draw_strokes},
};

pub(super) const LEFT_WIDTH: f32 = 146.0;
const LEFT_GAP: f32 = 16.0;
const CELL_GAP: f32 = 4.0;
const CELL_RADIUS: f32 = 10.0;
pub(super) const COLUMNS: usize = 6;
const ICON_SIZE: f32 = 20.0;

const EYEBROW_HEIGHT: f32 = 12.0;
const TEMPERATURE_HEIGHT: f32 = 32.0;
const CONDITION_HEIGHT: f32 = 14.0;
const ALERT_HEIGHT: f32 = 15.0;
const LEFT_LINE_GAP: f32 = 2.0;
/// The place name's line (which shares its row with the toggle), the temperature, the condition and the rain line.
const LEFT_HEIGHTS: [f32; 4] = [EYEBROW_HEIGHT, TEMPERATURE_HEIGHT, CONDITION_HEIGHT, ALERT_HEIGHT];

const LABEL_HEIGHT: f32 = 13.0;
const TEMPERATURE_LINE: f32 = 15.0;
const LOW_LINE: f32 = 13.0;
const SUB_HEIGHT: f32 = 12.0;
const HOURS_GAP: f32 = 3.0;
const DAYS_GAP: f32 = 5.0;

pub(super) const MESSAGE_BADGE: f32 = 36.0;
pub(super) const MESSAGE_GAP: f32 = 16.0;
const MESSAGE_TITLE_HEIGHT: f32 = 18.0;
const MESSAGE_DETAIL_HEIGHT: f32 = 16.0;
const MESSAGE_LINE_GAP: f32 = 3.0;

const TOGGLE_WIDTH: f32 = 48.0;
const TOGGLE_HEIGHT: f32 = 22.0;
/// The room kept between the place name and the toggle.
const TOGGLE_GAP: f32 = 6.0;

const EYEBROW: FontStyle = FontStyle::new(UI_FONT, 10.0, Weight::Regular, Align::Leading);
const TEMPERATURE: FontStyle = FontStyle::new(UI_FONT, 28.0, Weight::Medium, Align::Leading).tabular();
const CONDITION: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Regular, Align::Leading);
const RAIN: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Medium, Align::Leading);
const NOW_LABEL: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Medium, Align::Center);
const LABEL: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Regular, Align::Center);
const HOUR_TEMPERATURE: FontStyle = FontStyle::new(UI_FONT, 13.0, Weight::Regular, Align::Center).tabular();
const DAY_HIGH: FontStyle = FontStyle::new(UI_FONT, 13.0, Weight::Medium, Align::Center).tabular();
const DAY_LOW: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Regular, Align::Center).tabular();
const SUB: FontStyle = FontStyle::new(UI_FONT, 10.0, Weight::Regular, Align::Center);
const TITLE: FontStyle = FontStyle::new(UI_FONT, 14.0, Weight::Medium, Align::Center);
/// The design asks for 600; only Regular and Medium are bundled.
pub(super) const MESSAGE_TITLE: FontStyle = FontStyle::new(UI_FONT, 15.0, Weight::Medium, Align::Leading);
pub(super) const MESSAGE_DETAIL: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Regular, Align::Leading);
const DETAIL: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Regular, Align::Center);
const TOGGLE_LABEL: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Medium, Align::Center);

const GRAY: Color = Color::rgb(0.604, 0.604, 0.651);
const DIM: Color = Color::rgb(0.431, 0.431, 0.478);
const BLUE: Color = Color::rgb(0.455, 0.753, 0.988);
const NOW_FILL: Color = Color::rgb(0.169, 0.137, 0.071);
const NOW_TEXT: Color = Color::rgb(1.0, 0.753, 0.471);
const LIGHT: Color = Color::rgb(0.902, 0.902, 0.922);
const OFFLINE_FILL: Color = Color::rgb(0.09, 0.09, 0.11);
const OFFLINE_BORDER: Color = Color::rgb(0.149, 0.149, 0.176);
const OFFLINE_ICON: Color = Color::rgb(1.0, 0.753, 0.471);
const PIN_ICON: Color = Color::rgb(0.302, 0.671, 0.969);
const TOGGLE_FILL: Color = Color::rgb(0.11, 0.11, 0.133);
const TOGGLE_FILL_HOVERED: Color = Color::rgb(0.165, 0.165, 0.2);
const TOGGLE_BORDER: Color = Color::rgb(0.173, 0.173, 0.204);
const PIN: &str = "M12 21s-6.5-5.6-6.5-11a6.5 6.5 0 0 1 13 0c0 5.4-6.5 11-6.5 11z";
const PIN_HOLE: &str = "M9.5 10a2.5 2.5 0 1 0 5 0a2.5 2.5 0 1 0-5 0";
const CLOUD_OFF: &str = "M3 3l18 18";
const CLOUD_OFF_BODY: &str =
    "M9 6.2A6 6 0 0 1 17.5 10a4.5 4.5 0 0 1 3.1 7.6M17 19H7a3.75 3.75 0 0 1-1-7.4c.1-.6.3-1.2.6-1.8";

/// A line of the left column or of a cell: its height and what goes in it.
fn stack_height(heights: &[f32], gap: f32) -> f32 {
    heights.iter().sum::<f32>() + gap * heights.len().saturating_sub(1) as f32
}

impl WeatherPage {
    pub(super) fn left_area(bounds: Rect, scale: f32) -> Rect {
        Rect::new(bounds.left, bounds.top, bounds.left + LEFT_WIDTH * scale, bounds.bottom)
    }

    pub(super) fn column_area(bounds: Rect, scale: f32, column: usize) -> Rect {
        let left = bounds.left + (LEFT_WIDTH + LEFT_GAP) * scale;
        let width = (bounds.right - left - CELL_GAP * scale * (COLUMNS - 1) as f32) / COLUMNS as f32;
        let cell_left = left + column as f32 * (width + CELL_GAP * scale);
        Rect::new(cell_left, bounds.top, cell_left + width, bounds.bottom)
    }

    pub(super) fn draw_message(canvas: &Canvas, bounds: Rect, scale: f32, title: &str, detail: &str, opacity: f32) {
        let middle = bounds.center_y();
        let title_area = Rect::new(bounds.left, middle - 22.0 * scale, bounds.right, middle);
        let detail_area = Rect::new(bounds.left, middle + 2.0 * scale, bounds.right, middle + 22.0 * scale);
        canvas.text(&TITLE, title, title_area, Color::WHITE.faded(opacity));
        canvas.text(&DETAIL, detail, detail_area, GRAY.faded(opacity));
    }

    /// A round badge on the left and a left-aligned title and detail on the right, the whole row centered.
    pub(super) fn draw_message_row(canvas: &Canvas, bounds: Rect, scale: f32, message: &Message, opacity: f32) {
        let text_width =
            canvas.measure(&MESSAGE_TITLE, &message.title).max(canvas.measure(&MESSAGE_DETAIL, &message.detail));
        let left = bounds.center_x() - (MESSAGE_BADGE + MESSAGE_GAP + text_width) * scale / 2.0;
        let center = Point::new(left + MESSAGE_BADGE * scale / 2.0, bounds.center_y());
        let radius = MESSAGE_BADGE * scale / 2.0;
        canvas.fill_circle(center, radius, OFFLINE_BORDER.faded(opacity));
        canvas.fill_circle(center, radius - scale, OFFLINE_FILL.faded(opacity));
        let size = ICON_SIZE * scale;
        match message.kind {
            MessageKind::NoLocation => draw_strokes(
                canvas,
                &[Stroke::new(PIN, PIN_ICON), Stroke::new(PIN_HOLE, PIN_ICON)],
                center,
                size,
                2.0,
                opacity,
            ),
            MessageKind::Offline => draw_strokes(
                canvas,
                &[Stroke::new(CLOUD_OFF, OFFLINE_ICON), Stroke::new(CLOUD_OFF_BODY, OFFLINE_ICON)],
                center,
                size,
                2.0,
                opacity,
            ),
        }
        let stack = stack_height(&[MESSAGE_TITLE_HEIGHT, MESSAGE_DETAIL_HEIGHT], MESSAGE_LINE_GAP) * scale;
        let text_left = left + (MESSAGE_BADGE + MESSAGE_GAP) * scale;
        let title_top = bounds.center_y() - stack / 2.0;
        let title_area = Rect::new(
            text_left,
            title_top,
            text_left + text_width * scale + 2.0,
            title_top + MESSAGE_TITLE_HEIGHT * scale,
        );
        canvas.text(&MESSAGE_TITLE, &message.title, title_area, Color::WHITE.faded(opacity));
        let detail_top = title_area.bottom + MESSAGE_LINE_GAP * scale;
        let detail_area = Rect { top: detail_top, bottom: detail_top + MESSAGE_DETAIL_HEIGHT * scale, ..title_area };
        canvas.text(&MESSAGE_DETAIL, &message.detail, detail_area, GRAY.faded(opacity));
    }

    /// The pill on the first line of the left column, at its right end, level with the place name.
    pub(super) fn toggle_area(bounds: Rect, scale: f32) -> Rect {
        let center_y = bounds.center_y() - stack_height(&LEFT_HEIGHTS, LEFT_LINE_GAP) * scale / 2.0
            + (EYEBROW_HEIGHT / 2.0 + 1.5) * scale;
        let right = bounds.left + LEFT_WIDTH * scale;
        let top = (center_y - TOGGLE_HEIGHT * scale / 2.0).max(bounds.top);
        Rect::new(right - TOGGLE_WIDTH * scale, top, right, top + TOGGLE_HEIGHT * scale)
    }

    /// The pill names what is shown: "Today" for the hours, "Week" for the days; pressing it shows the other.
    pub(super) fn draw_toggle(&self, canvas: &Canvas, area: Rect, opacity: f32) {
        let scale = canvas.scale();
        let radius = area.height() / 2.0;
        let fill = if self.is_toggle_hovered { TOGGLE_FILL_HOVERED } else { TOGGLE_FILL };
        canvas.fill_rounded_rect(area, radius, TOGGLE_BORDER.faded(opacity));
        let inner = Rect::new(area.left + scale, area.top + scale, area.right - scale, area.bottom - scale);
        canvas.fill_rounded_rect(inner, radius - scale, fill.faded(opacity));
        let label = if self.shows_days { "Week" } else { "Today" };
        canvas.text(&TOGGLE_LABEL, label, area, LIGHT.faded(opacity));
    }

    pub(super) fn draw_left(&self, canvas: &Canvas, forecast: &Forecast, area: Rect, scale: f32, opacity: f32) {
        let now = self.now();
        let place = self.location.as_ref().map(|location| location.label.as_str()).unwrap_or_default();
        let (temperature, condition, rain) = if self.shows_days {
            let Some(today) = forecast.days.first() else { return };
            let rain = if today.rain_millimeters >= 0.2 {
                format!("{:.0} mm of rain", today.rain_millimeters)
            } else {
                "No rain expected".to_owned()
            };
            (today.high, today.symbol.label().to_owned(), rain)
        } else {
            let Some(current) = forecast.current(now) else { return };
            (
                current.temperature,
                format!("{} \u{b7} feels {:.0}\u{b0}", current.symbol.label(), forecast.feels_like),
                rain_line(forecast, now),
            )
        };
        let mut top = area.center_y() - stack_height(&LEFT_HEIGHTS, LEFT_LINE_GAP) * scale / 2.0;
        let toggle = Self::toggle_area(area, scale);
        let eyebrow_top = toggle.center_y() - EYEBROW_HEIGHT * scale / 2.0;
        let eyebrow =
            Rect::new(area.left, eyebrow_top, toggle.left - TOGGLE_GAP * scale, eyebrow_top + EYEBROW_HEIGHT * scale);
        canvas.text(&EYEBROW, place, eyebrow, DIM.faded(opacity));
        let lines: [(&FontStyle, String, Color); 4] = [
            (&EYEBROW, String::new(), DIM),
            (&TEMPERATURE, format!("{temperature:.0}\u{b0}"), Color::WHITE),
            (&CONDITION, condition, GRAY),
            (&RAIN, rain, BLUE),
        ];
        for ((style, text, color), height) in lines.into_iter().zip(LEFT_HEIGHTS) {
            let line = Rect::new(area.left, top, area.right, top + height * scale);
            canvas.text(style, &text, line, color.faded(opacity));
            top = line.bottom + LEFT_LINE_GAP * scale;
        }
    }

    pub(super) fn draw_columns(&self, canvas: &Canvas, forecast: &Forecast, bounds: Rect, scale: f32, opacity: f32) {
        let now = self.now();
        let first_hour = forecast.hours.iter().position(|hour| hour.time + Duration::hours(1) > now).unwrap_or(0);
        for column in 0..COLUMNS {
            let area = Self::column_area(bounds, scale, column);
            let is_now = column == 0;
            let (label, symbol, high, low, rain) = if self.shows_days {
                let Some(day) = forecast.days.get(column) else { continue };
                let label = if is_now { "Today".to_owned() } else { day.date.format("%a").to_string() };
                (label, day.symbol, day.high, Some(day.low), day.rain_millimeters)
            } else {
                let Some(hour) = forecast.hours.get(first_hour + column) else { continue };
                let label =
                    if is_now { "Now".to_owned() } else { hour.time.with_timezone(&Local).format("%H:%M").to_string() };
                (label, hour.symbol, hour.temperature, None, hour.rain_millimeters)
            };
            if is_now {
                canvas.fill_rounded_rect(area, CELL_RADIUS * scale, NOW_FILL.faded(opacity));
            }
            let (gap, heights): (f32, &[f32]) = if self.shows_days {
                (DAYS_GAP, &[LABEL_HEIGHT, ICON_SIZE, TEMPERATURE_LINE + LOW_LINE])
            } else {
                (HOURS_GAP, &[LABEL_HEIGHT, ICON_SIZE, TEMPERATURE_LINE, SUB_HEIGHT])
            };
            let mut top = area.center_y() - stack_height(heights, gap) * scale / 2.0;
            let mut next_row = |height: f32| {
                let row = Rect::new(area.left, top, area.right, top + height * scale);
                top = row.bottom + gap * scale;
                row
            };
            let (label_style, label_color) = if is_now { (&NOW_LABEL, NOW_TEXT) } else { (&LABEL, GRAY) };
            canvas.text(label_style, &label, next_row(LABEL_HEIGHT), label_color.faded(opacity));
            let icon_row = next_row(ICON_SIZE);
            let icon_center = Point::new(icon_row.center_x(), icon_row.center_y());
            draw_symbol(canvas, symbol, icon_center, ICON_SIZE * scale, is_now, opacity);
            let high_color = if is_now { Color::WHITE } else { LIGHT };
            match low {
                Some(low) => {
                    let row = next_row(TEMPERATURE_LINE + LOW_LINE);
                    let high_row = Rect { bottom: row.top + TEMPERATURE_LINE * scale, ..row };
                    let low_row = Rect { top: high_row.bottom, ..row };
                    canvas.text(&DAY_HIGH, &format!("{high:.0}\u{b0}"), high_row, high_color.faded(opacity));
                    canvas.text(&DAY_LOW, &format!("{low:.0}\u{b0}"), low_row, DIM.faded(opacity));
                }
                None => {
                    let row = next_row(TEMPERATURE_LINE);
                    canvas.text(&HOUR_TEMPERATURE, &format!("{high:.0}\u{b0}"), row, high_color.faded(opacity));
                    let sub = next_row(SUB_HEIGHT);
                    if rain >= 0.1 {
                        canvas.text(&SUB, &format!("{rain:.1} mm"), sub, BLUE.faded(opacity));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOUNDS: Rect = Rect::new(0.0, 0.0, super::super::CONTENT_WIDTH, super::super::CONTENT_HEIGHT);

    #[test]
    fn the_cells_fill_the_right_side_with_four_between_them() {
        let first = WeatherPage::column_area(BOUNDS, 1.0, 0);
        let second = WeatherPage::column_area(BOUNDS, 1.0, 1);
        let last = WeatherPage::column_area(BOUNDS, 1.0, COLUMNS - 1);
        assert_eq!(first.left, LEFT_WIDTH + LEFT_GAP);
        assert!((second.left - first.right - CELL_GAP).abs() < 0.01);
        assert!((last.right - BOUNDS.right).abs() < 0.01);
        assert!((first.width() - last.width()).abs() < 0.01);
        assert!((first.width() - 54.33).abs() < 0.01);
    }

    #[test]
    fn the_pill_sits_at_the_right_end_of_the_first_line_inside_the_left_column() {
        let pill = WeatherPage::toggle_area(BOUNDS, 1.0);
        let left = WeatherPage::left_area(BOUNDS, 1.0);
        assert_eq!(pill.right, left.right);
        assert_eq!((pill.width(), pill.height()), (TOGGLE_WIDTH, TOGGLE_HEIGHT));
        assert!(pill.top >= BOUNDS.top && pill.bottom <= BOUNDS.top + 32.0);
        assert!(left.contains(Point::new(pill.center_x(), pill.center_y())));
        assert!(!pill.contains(Point::new(20.0, pill.center_y())), "the temperature is not a button");
    }

    #[test]
    fn the_pill_scales_with_the_screen() {
        let pill = WeatherPage::toggle_area(Rect::new(0.0, 0.0, 2.0 * BOUNDS.right, 2.0 * BOUNDS.bottom), 2.0);
        assert_eq!((pill.width(), pill.height()), (2.0 * TOGGLE_WIDTH, 2.0 * TOGGLE_HEIGHT));
    }

    #[test]
    fn a_stack_adds_its_lines_and_the_gaps_between_them() {
        assert_eq!(stack_height(&[12.0, 32.0, 14.0, 15.0], 2.0), 79.0);
        assert_eq!(stack_height(&[], 2.0), 0.0);
    }

    #[test]
    fn both_cell_stacks_fit_the_page_height() {
        assert!(stack_height(&[LABEL_HEIGHT, ICON_SIZE, TEMPERATURE_LINE, SUB_HEIGHT], HOURS_GAP) <= 86.0);
        assert!(stack_height(&[LABEL_HEIGHT, ICON_SIZE, TEMPERATURE_LINE + LOW_LINE], DAYS_GAP) <= 86.0);
    }
}
