//! The weather for the saved location: now and the next hours, or the next days, with an optional rain alert.
//! The forecast comes from MET Norway (Locationforecast); the page asks for it at start and every half hour.

mod fetch;
mod forecast;
mod picture;
mod symbol;

use std::sync::{Arc, Mutex};

use places::Location;
use salah::{DateTime, Duration, Local, Utc};

use self::{
    fetch::{Outcome, place_key},
    forecast::{Forecast, rain_line},
    picture::draw_symbol,
};
use crate::{
    activity::{FLASH_TIMER, Flash, Presence},
    gfx::{Align, Canvas, Color, FontStyle, Point, Rect, Size, UI_FONT, Weight},
    page::{Context, Control, Frame, Mode, Page, PageSizes, PointerEvent, Priority, TimerId},
    ui::{self, Icon},
};

const CONTENT_WIDTH: f32 = 508.0;
const CONTENT_HEIGHT: f32 = 86.0;
const RESTING_WIDTH: f32 = 150.0;
const LEFT_WIDTH: f32 = 150.0;
const DIVIDER_GAP: f32 = 14.0;
const COLUMNS: usize = 6;
const ICON_SIZE: f32 = 26.0;

const RAIN_ALERT_KEY: &str = "weather.rain_alert";
const REFRESH_TIMER: TimerId = TimerId(1);
const ALERT_TIMER: TimerId = TimerId(2);
const REFRESH_MINUTES: u32 = 30;
/// After a failed request the page tries again sooner.
const RETRY_MINUTES: u32 = 10;
const RAIN_ALERT_LEAD_MINUTES: i64 = 30;
const RAIN_WINDOW_HOURS: i64 = 12;
const ALERT_MILLISECONDS: u32 = 9000;
const MAX_ALERT_WAIT_MILLISECONDS: i64 = 6 * 60 * 60 * 1000;

const MONO: &str = "Consolas";
const TEMPERATURE: FontStyle = FontStyle::new(MONO, 30.0, Weight::Medium, Align::Leading);
const COLUMN_TEMPERATURE: FontStyle = FontStyle::new(MONO, 13.0, Weight::Medium, Align::Center);
const LOW_TEMPERATURE: FontStyle = FontStyle::new(MONO, 13.0, Weight::Regular, Align::Center);
const LABEL: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Regular, Align::Leading);
const RAIN: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Medium, Align::Leading);
const SMALL: FontStyle = FontStyle::new(UI_FONT, 10.0, Weight::Regular, Align::Leading);
const COLUMN_LABEL: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Regular, Align::Center);
const COLUMN_SUB: FontStyle = FontStyle::new(UI_FONT, 10.0, Weight::Regular, Align::Center);
const TITLE: FontStyle = FontStyle::new(UI_FONT, 14.0, Weight::Medium, Align::Center);
const DETAIL: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Regular, Align::Center);

const GRAY: Color = Color::rgb(0.604, 0.604, 0.651);
const DIM: Color = Color::rgb(0.431, 0.431, 0.478);
const DIVIDER: Color = Color::rgb(0.149, 0.149, 0.176);
const BLUE: Color = Color::rgb(0.3, 0.67, 0.97);
const NOW_FILL: Color = Color::rgb(1.0, 0.62, 0.04);
const NOW_TEXT: Color = Color::rgb(1.0, 0.8, 0.4);

pub struct WeatherPage {
    location: Option<Location>,
    forecast: Option<Forecast>,
    is_offline: bool,
    shows_days: bool,
    rain_alert: bool,
    /// The first rainy hour the alert was already shown for, so one rain gives one message.
    alerted_for: Option<DateTime<Utc>>,
    alert: Flash,
    alert_title: String,
    alert_width: f32,
    outcome: Outcome,
    fixed_now: Option<DateTime<Utc>>,
}

impl WeatherPage {
    pub fn new() -> Self {
        Self {
            location: None,
            forecast: None,
            is_offline: false,
            shows_days: false,
            rain_alert: true,
            alerted_for: None,
            alert: Flash::default(),
            alert_title: String::new(),
            alert_width: 0.0,
            outcome: Arc::new(Mutex::new(None)),
            fixed_now: None,
        }
    }

    #[cfg(test)]
    pub fn sample(shows_days: bool) -> Self {
        let now = DateTime::parse_from_rfc3339("2026-11-05T09:05:00Z").unwrap().with_timezone(&Utc);
        let skies = ["partlycloudy_day", "partlycloudy_day", "rain", "heavyrainandthunder", "rain", "cloudy"];
        let hours: Vec<String> = (0..24)
            .map(|offset| {
                let (code, rain) = if (2..5).contains(&offset) { (skies[offset % 6], 1.4) } else { (skies[offset % 2], 0.0) };
                let time = (now + Duration::hours(offset as i64 - 1)).format("%Y-%m-%dT%H:00:00Z").to_string();
                format!(
                    r#"{{"time":"{time}","data":{{"instant":{{"details":{{"air_temperature":{},"apparent_air_temperature":36.0}}}},"next_1_hours":{{"summary":{{"symbol_code":"{code}"}},"details":{{"precipitation_amount":{rain}}}}},"next_6_hours":{{"summary":{{"symbol_code":"{code}"}},"details":{{"precipitation_amount":{rain}}}}}}}}}"#,
                    31 - (offset as i32 - 2).abs()
                )
            })
            .collect();
        let body = format!(r#"{{"properties":{{"timeseries":[{}]}}}}"#, hours.join(","));
        let mut page = Self::new();
        page.location = Some(Location {
            label: "Pekanbaru, ID".to_owned(),
            country: "ID".to_owned(),
            latitude: 0.517,
            longitude: 101.442,
        });
        page.forecast = Forecast::parse(&body, now);
        page.shows_days = shows_days;
        page.fixed_now = Some(now);
        page
    }

    #[cfg(test)]
    pub fn sample_offline() -> Self {
        let mut page = Self::sample(false);
        page.is_offline = true;
        page
    }

    fn now(&self) -> DateTime<Utc> {
        self.fixed_now.unwrap_or_else(Utc::now)
    }

    fn read_settings(&mut self, context: &Context) {
        let read = |key: &str| context.settings.get_string(key).map(str::to_owned);
        self.location = Location::read(&read);
        self.rain_alert = context.settings.get_bool(RAIN_ALERT_KEY, true);
    }

    fn start_fetch(&self, context: &Context) {
        if let Some(location) = &self.location {
            fetch::start(location.latitude, location.longitude, Arc::clone(&self.outcome), context.notifier());
        }
    }

    /// Takes what the request found: a forecast, or "offline" with a sooner retry.
    fn apply_outcome(&mut self, context: &mut Context) {
        let Some((place, result)) = self.outcome.lock().ok().and_then(|mut slot| slot.take()) else { return };
        let expected = self.location.as_ref().map(|location| place_key(location.latitude, location.longitude));
        if expected != Some(place) {
            return;
        }
        let parsed = result.ok().and_then(|body| Forecast::parse(&body, self.now()));
        self.is_offline = parsed.is_none();
        if let Some(forecast) = parsed {
            self.forecast = Some(forecast);
        }
        let minutes = if self.is_offline { RETRY_MINUTES } else { REFRESH_MINUTES };
        context.set_timer(REFRESH_TIMER, minutes * 60 * 1000);
        self.arm_alert(context);
        context.request_repaint();
    }

    /// Shows the rain message when rain is under the lead time away, or waits until it is.
    fn arm_alert(&mut self, context: &mut Context) {
        context.kill_timer(ALERT_TIMER);
        let now = self.now();
        let Some(hour) = self
            .forecast
            .as_ref()
            .filter(|_| self.rain_alert)
            .and_then(|forecast| forecast.upcoming_rain(now, RAIN_WINDOW_HOURS))
        else {
            return;
        };
        if self.alerted_for == Some(hour.time) {
            return;
        }
        let due = hour.time - Duration::minutes(RAIN_ALERT_LEAD_MINUTES);
        if due <= now {
            let minutes = (hour.time - now).num_minutes().max(1);
            self.alerted_for = Some(hour.time);
            self.alert_title = format!("Rain in {minutes} min");
            self.alert_width = ui::message_width(context.fonts(), &self.alert_title, None);
            self.alert.show(context, ALERT_MILLISECONDS);
        } else {
            let wait = (due - now).num_milliseconds().clamp(1000, MAX_ALERT_WAIT_MILLISECONDS);
            context.set_timer(ALERT_TIMER, wait as u32);
        }
    }

    fn left_area(bounds: Rect, scale: f32) -> Rect {
        Rect::new(bounds.left, bounds.top, bounds.left + LEFT_WIDTH * scale, bounds.bottom)
    }

    fn column_area(bounds: Rect, scale: f32, column: usize) -> Rect {
        let left = bounds.left + (LEFT_WIDTH + DIVIDER_GAP * 2.0) * scale;
        let width = (bounds.right - left) / COLUMNS as f32;
        Rect::new(left + column as f32 * width, bounds.top, left + (column + 1) as f32 * width, bounds.bottom)
    }

    fn draw_message(canvas: &Canvas, bounds: Rect, scale: f32, title: &str, detail: &str, opacity: f32) {
        let middle = bounds.center_y();
        let title_area = Rect::new(bounds.left, middle - 22.0 * scale, bounds.right, middle);
        let detail_area = Rect::new(bounds.left, middle + 2.0 * scale, bounds.right, middle + 22.0 * scale);
        canvas.text(&TITLE, title, title_area, Color::WHITE.faded(opacity));
        canvas.text(&DETAIL, detail, detail_area, GRAY.faded(opacity));
    }

    fn draw_left(&self, canvas: &Canvas, forecast: &Forecast, area: Rect, scale: f32, opacity: f32) {
        let now = self.now();
        let place = self.location.as_ref().map(|location| location.label.as_str()).unwrap_or_default();
        let line =
            |top: f32, bottom: f32| Rect::new(area.left, area.top + top * scale, area.right, area.top + bottom * scale);
        if self.shows_days {
            let Some(today) = forecast.days.first() else { return };
            canvas.text(
                &LABEL,
                &format!("Today \u{b7} {}", today.symbol.label()),
                line(2.0, 18.0),
                GRAY.faded(opacity),
            );
            canvas.text(
                &TEMPERATURE,
                &format!("{:.0}\u{b0}", today.high),
                line(16.0, 52.0),
                Color::WHITE.faded(opacity),
            );
            let low_left =
                area.left + (canvas.measure(&TEMPERATURE, &format!("{:.0}\u{b0}", today.high)) + 8.0) * scale;
            canvas.text(
                &LABEL,
                &format!("{:.0}\u{b0}", today.low),
                Rect { left: low_left, ..line(30.0, 50.0) },
                GRAY.faded(opacity),
            );
            let rain = if today.rain_millimeters >= 0.2 {
                format!("{:.0} mm of rain", today.rain_millimeters)
            } else {
                "No rain expected".to_owned()
            };
            canvas.text(&RAIN, &rain, line(52.0, 68.0), BLUE.faded(opacity));
            canvas.text(&SMALL, place, line(68.0, 82.0), DIM.faded(opacity));
            return;
        }
        let Some(current) = forecast.current(now) else { return };
        canvas.text(&LABEL, current.symbol.label(), line(2.0, 18.0), GRAY.faded(opacity));
        canvas.text(
            &TEMPERATURE,
            &format!("{:.0}\u{b0}", current.temperature),
            line(16.0, 52.0),
            Color::WHITE.faded(opacity),
        );
        canvas.text(&RAIN, &rain_line(forecast, now), line(52.0, 68.0), BLUE.faded(opacity));
        canvas.text(
            &SMALL,
            &format!("{place} \u{b7} feels {:.0}\u{b0}", forecast.feels_like),
            line(68.0, 82.0),
            DIM.faded(opacity),
        );
    }

    fn draw_columns(&self, canvas: &Canvas, forecast: &Forecast, bounds: Rect, scale: f32, opacity: f32) {
        let now = self.now();
        let backdrop = Color::BLACK;
        let first_hour = forecast.hours.iter().position(|hour| hour.time + Duration::hours(1) > now).unwrap_or(0);
        for column in 0..COLUMNS {
            let area = Self::column_area(bounds, scale, column);
            let center_x = area.center_x();
            let row = |top: f32, bottom: f32| {
                Rect::new(area.left, area.top + top * scale, area.right, area.top + bottom * scale)
            };
            let (label, symbol, top, low, rain, is_now) = if self.shows_days {
                let Some(day) = forecast.days.get(column) else { continue };
                let label = if column == 0 { "Today".to_owned() } else { day.date.format("%a").to_string() };
                (label, day.symbol, day.high, Some(day.low), day.rain_millimeters, column == 0)
            } else {
                let Some(hour) = forecast.hours.get(first_hour + column) else { continue };
                let label = if column == 0 {
                    "Now".to_owned()
                } else {
                    hour.time.with_timezone(&Local).format("%H:%M").to_string()
                };
                (label, hour.symbol, hour.temperature, None, hour.rain_millimeters, column == 0)
            };
            if is_now {
                let chip = Rect::new(
                    area.left + 3.0 * scale,
                    area.top + 4.0 * scale,
                    area.right - 3.0 * scale,
                    area.bottom - 4.0 * scale,
                );
                canvas.fill_rounded_rect(chip, 8.0 * scale, NOW_FILL.with_alpha(0.14 * opacity));
            }
            let label_color = if is_now { NOW_TEXT } else { GRAY };
            canvas.text(&COLUMN_LABEL, &label, row(8.0, 22.0), label_color.faded(opacity));
            draw_symbol(
                canvas,
                symbol,
                Point::new(center_x, area.top + 34.0 * scale),
                ICON_SIZE * scale,
                backdrop,
                opacity,
            );
            match low {
                Some(low) => {
                    let high_area = Rect { right: center_x - 1.0 * scale, ..row(48.0, 64.0) };
                    let low_area = Rect { left: center_x + 1.0 * scale, ..row(48.0, 64.0) };
                    canvas.text(&COLUMN_TEMPERATURE, &format!("{top:.0}"), high_area, Color::WHITE.faded(opacity));
                    canvas.text(&LOW_TEMPERATURE, &format!("{low:.0}"), low_area, DIM.faded(opacity));
                }
                None => canvas.text(
                    &COLUMN_TEMPERATURE,
                    &format!("{top:.0}\u{b0}"),
                    row(48.0, 64.0),
                    Color::WHITE.faded(opacity),
                ),
            }
            if !self.shows_days && rain >= 0.1 {
                canvas.text(&COLUMN_SUB, &format!("{rain:.1} mm"), row(65.0, 79.0), BLUE.faded(opacity));
            }
        }
    }
}

impl Page for WeatherPage {
    fn id(&self) -> &'static str {
        "weather"
    }

    fn name(&self) -> &'static str {
        "Weather"
    }

    fn icon(&self) -> Icon {
        Icon::Cloud
    }

    fn is_opt_in(&self) -> bool {
        true
    }

    fn settings(&self) -> Vec<Control> {
        vec![Control::Switch {
            key: RAIN_ALERT_KEY,
            label: "Rain alert",
            hint: "a message 30 min before rain",
            default: true,
        }]
    }

    fn priority(&self, mode: Mode) -> Option<Priority> {
        (mode == Mode::Expanded).then_some(Priority::BACKGROUND)
    }

    fn sizes(&self) -> PageSizes {
        PageSizes { resting_width: RESTING_WIDTH, expanded: Size::new(CONTENT_WIDTH, CONTENT_HEIGHT) }
    }

    fn draw_resting(&self, _canvas: &Canvas, _frame: &Frame, _opacity: f32) {}

    fn draw_expanded(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        let (bounds, scale) = (frame.bounds, frame.scale);
        if self.location.is_none() {
            Self::draw_message(
                canvas,
                bounds,
                scale,
                "Choose a location",
                "Pick a city in Settings, General, to see the weather.",
                opacity,
            );
            return;
        }
        if self.is_offline {
            let detail = match &self.forecast {
                Some(forecast) => format!(
                    "No internet. Last update at {}; it will try again on its own.",
                    forecast.fetched_at.with_timezone(&Local).format("%H:%M")
                ),
                None => "No internet. It will try again on its own.".to_owned(),
            };
            Self::draw_message(canvas, bounds, scale, "Can't update the weather", &detail, opacity);
            return;
        }
        let Some(forecast) = &self.forecast else {
            Self::draw_message(canvas, bounds, scale, "Loading the weather", "", opacity);
            return;
        };
        self.draw_left(canvas, forecast, Self::left_area(bounds, scale), scale, opacity);
        let divider_left = bounds.left + (LEFT_WIDTH + DIVIDER_GAP) * scale;
        canvas.fill_rounded_rect(
            Rect::new(divider_left, bounds.top + 4.0 * scale, divider_left + scale, bounds.bottom - 4.0 * scale),
            0.0,
            DIVIDER.faded(opacity),
        );
        self.draw_columns(canvas, forecast, bounds, scale, opacity);
    }

    fn presence(&self) -> Option<Presence> {
        self.alert.is_visible().then_some(Presence::Transient { width: self.alert_width })
    }

    fn draw_presence(&self, canvas: &Canvas, area: Rect, _scale: f32, opacity: f32) {
        ui::draw_message(canvas, area, opacity, Icon::Cloud, BLUE, &self.alert_title, None);
    }

    fn on_start(&mut self, context: &mut Context) {
        self.read_settings(context);
        self.start_fetch(context);
    }

    fn on_settings_changed(&mut self, context: &mut Context) {
        let previous = self.location.clone();
        self.read_settings(context);
        if self.location != previous {
            self.forecast = None;
            self.is_offline = false;
            self.alerted_for = None;
            self.start_fetch(context);
            context.request_repaint();
        } else {
            self.arm_alert(context);
        }
    }

    fn on_notify(&mut self, context: &mut Context) {
        self.apply_outcome(context);
    }

    fn on_pointer(&mut self, event: PointerEvent, frame: &Frame, context: &mut Context) {
        if let PointerEvent::Pressed(point) = event
            && frame.is_open()
            && self.forecast.is_some()
            && !self.is_offline
            && Self::left_area(frame.bounds, frame.scale).contains(point)
        {
            self.shows_days = !self.shows_days;
            context.request_repaint();
        }
    }

    fn on_timer(&mut self, timer: TimerId, context: &mut Context) {
        match timer {
            REFRESH_TIMER => self.start_fetch(context),
            ALERT_TIMER => self.arm_alert(context),
            FLASH_TIMER => {
                self.alert.on_timer(timer, context);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_weather_is_a_page_to_add_with_no_collapsed_state() {
        let page = WeatherPage::new();
        assert!(page.is_opt_in());
        assert_eq!(page.priority(Mode::Resting), None);
        assert!(page.priority(Mode::Expanded).is_some());
        assert_eq!(page.presence(), None);
    }

    #[test]
    fn the_sample_has_a_current_hour_days_and_rain_ahead() {
        let page = WeatherPage::sample(false);
        let forecast = page.forecast.as_ref().unwrap();
        assert!(forecast.hours.len() >= COLUMNS);
        assert!(!forecast.days.is_empty());
        assert!(forecast.upcoming_rain(page.now(), RAIN_WINDOW_HOURS).is_some());
        assert!(rain_line(forecast, page.now()).starts_with("Rain from "));
    }

    #[test]
    fn the_columns_split_the_right_side_evenly() {
        let bounds = Rect::new(0.0, 0.0, CONTENT_WIDTH, CONTENT_HEIGHT);
        let first = WeatherPage::column_area(bounds, 1.0, 0);
        let last = WeatherPage::column_area(bounds, 1.0, COLUMNS - 1);
        assert_eq!(first.left, LEFT_WIDTH + 2.0 * DIVIDER_GAP);
        assert!((last.right - CONTENT_WIDTH).abs() < 0.01);
        assert!((first.width() - last.width()).abs() < 0.01);
    }

    #[test]
    fn the_left_side_is_what_switches_between_hours_and_days() {
        let bounds = Rect::new(0.0, 0.0, CONTENT_WIDTH, CONTENT_HEIGHT);
        assert!(WeatherPage::left_area(bounds, 1.0).contains(Point::new(20.0, 40.0)));
        assert!(!WeatherPage::left_area(bounds, 1.0).contains(Point::new(300.0, 40.0)));
    }
}
