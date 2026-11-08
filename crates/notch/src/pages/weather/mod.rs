//! The weather for the saved location: now and the next hours, or the next days, with an optional rain alert.
//! The forecast comes from MET Norway (Locationforecast); the page asks for it at start and every half hour.

mod alert;
mod draw;
mod fetch;
mod forecast;
mod message;
mod picture;
mod symbol;

use std::sync::{Arc, Mutex};

use places::Location;
use salah::{DateTime, Duration, Local, Utc};

use self::{
    alert::{Alert, STAGE_TIMER, body_text},
    fetch::{Failure, Outcome, place_key},
    forecast::Forecast,
};
use crate::{
    activity::{FLASH_TIMER, Presence},
    gfx::{Canvas, Rect, Size},
    log,
    page::{Context, Frame, Mode, Page, PageSizes, PointerEvent, Priority, TimerId},
};

const CONTENT_WIDTH: f32 = 508.0;
const CONTENT_HEIGHT: f32 = 86.0;
const RESTING_WIDTH: f32 = 150.0;

const RAIN_ALERT_KEY: &str = "weather.rain_alert";
const REFRESH_TIMER: TimerId = TimerId(1);
const ALERT_TIMER: TimerId = TimerId(2);
const REFRESH_MINUTES: u32 = 30;
/// After a failed request the page tries again sooner.
const RETRY_MINUTES: u32 = 10;
const RAIN_ALERT_LEAD_MINUTES: i64 = 30;
const RAIN_WINDOW_HOURS: i64 = 12;
const MAX_ALERT_WAIT_MILLISECONDS: i64 = 6 * 60 * 60 * 1000;

pub struct WeatherPage {
    location: Option<Location>,
    forecast: Option<Forecast>,
    is_offline: bool,
    /// The request failed for a reason other than the network (a bad status or an unreadable answer).
    is_service_error: bool,
    /// The pill reads "Week" and the cells show the days; otherwise it reads "Today" and they show the hours.
    /// Kept while the notch runs, as before.
    shows_days: bool,
    is_toggle_hovered: bool,
    /// The content width of the message on show, so the page hugs it; `None` while the forecast shows.
    message_width: Option<f32>,
    rain_alert: bool,
    /// The first rainy hour the alert was already shown for, so one rain gives one message.
    alerted_for: Option<DateTime<Utc>>,
    alert: Alert,
    outcome: Outcome,
    fixed_now: Option<DateTime<Utc>>,
}

impl WeatherPage {
    pub fn new() -> Self {
        Self {
            location: None,
            forecast: None,
            is_offline: false,
            is_service_error: false,
            shows_days: false,
            is_toggle_hovered: false,
            message_width: None,
            rain_alert: true,
            alerted_for: None,
            alert: Alert::new(),
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
    pub fn sample_alert(fonts: &crate::gfx::Fonts) -> Self {
        let mut page = Self::sample(false);
        page.alert = Alert::sample_banner(fonts);
        page
    }

    #[cfg(test)]
    pub fn sample_offline(fonts: &crate::gfx::Fonts) -> Self {
        let mut page = Self::sample(false);
        page.is_offline = true;
        page.refresh_message_width(fonts);
        page
    }

    #[cfg(test)]
    pub fn sample_no_location(fonts: &crate::gfx::Fonts) -> Self {
        let mut page = Self::new();
        page.refresh_message_width(fonts);
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
        let parsed = match result {
            Ok(body) => {
                let parsed = Forecast::parse(&body, self.now());
                if parsed.is_none() {
                    log::debug(&format!("weather: the response is not a forecast ({} bytes)", body.len()));
                }
                self.is_service_error = parsed.is_none();
                parsed
            }
            Err(failure) => {
                log::debug(&format!("weather: request failed: {}", failure.reason()));
                self.is_service_error = matches!(failure, Failure::Service(_));
                None
            }
        };
        self.is_offline = parsed.is_none();
        if let Some(forecast) = parsed {
            self.forecast = Some(forecast);
        }
        self.refresh_message_width(context.fonts());
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
            let time = hour.time.with_timezone(&Local).format("%H:%M").to_string();
            let place = self.location.as_ref().map(|location| location.label.as_str()).unwrap_or_default();
            let body = body_text(hour.symbol.label(), &time, place);
            self.alert.show(context, format!("Rain in {minutes} min"), time, &body);
        } else {
            let wait = (due - now).num_milliseconds().clamp(1000, MAX_ALERT_WAIT_MILLISECONDS);
            context.set_timer(ALERT_TIMER, wait as u32);
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

    fn is_opt_in(&self) -> bool {
        true
    }

    fn priority(&self, mode: Mode) -> Option<Priority> {
        (mode == Mode::Expanded).then_some(Priority::BACKGROUND)
    }

    fn sizes(&self) -> PageSizes {
        let width = self.message_width.unwrap_or(CONTENT_WIDTH);
        PageSizes { resting_width: RESTING_WIDTH, expanded: Size::new(width, CONTENT_HEIGHT) }
    }

    fn draw_resting(&self, _canvas: &Canvas, _frame: &Frame, _opacity: f32) {}

    fn draw_expanded(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        let (bounds, scale) = (frame.bounds, frame.scale);
        if let Some(message) = self.message() {
            Self::draw_message_row(canvas, bounds, scale, &message, opacity);
            return;
        }
        let Some(forecast) = &self.forecast else {
            Self::draw_message(canvas, bounds, scale, "Loading the weather", "", opacity);
            return;
        };
        self.draw_left(canvas, forecast, Self::left_area(bounds, scale), scale, opacity);
        self.draw_toggle(canvas, Self::toggle_area(bounds, scale), opacity);
        self.draw_columns(canvas, forecast, bounds, scale, opacity);
    }

    fn presence(&self) -> Option<Presence> {
        self.alert.presence()
    }

    fn draw_presence(&self, canvas: &Canvas, area: Rect, scale: f32, opacity: f32) {
        self.alert.draw(canvas, area, scale, opacity);
    }

    fn on_hover_changed(&mut self, is_hovering: bool, context: &mut Context) {
        self.alert.on_hover_changed(is_hovering, context);
    }

    fn on_presence_pointer(&mut self, event: PointerEvent, area: Rect, scale: f32, context: &mut Context) {
        self.alert.on_pointer(event, area, scale, context);
    }

    fn on_start(&mut self, context: &mut Context) {
        self.read_settings(context);
        self.refresh_message_width(context.fonts());
        self.start_fetch(context);
    }

    fn on_settings_changed(&mut self, context: &mut Context) {
        let previous = self.location.clone();
        self.read_settings(context);
        if self.location != previous {
            self.forecast = None;
            self.is_offline = false;
            self.is_service_error = false;
            self.alerted_for = None;
            self.refresh_message_width(context.fonts());
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
        let is_available = self.forecast.is_some() && !self.is_offline;
        let toggle = Self::toggle_area(frame.bounds, frame.scale);
        match event {
            PointerEvent::Moved(point) => {
                let is_hovered = is_available && toggle.contains(point);
                if is_hovered != self.is_toggle_hovered {
                    self.is_toggle_hovered = is_hovered;
                    context.request_repaint();
                }
            }
            PointerEvent::Pressed(point) if frame.is_open() && is_available && toggle.contains(point) => {
                self.shows_days = !self.shows_days;
                context.request_repaint();
            }
            PointerEvent::Left | PointerEvent::CaptureLost if std::mem::take(&mut self.is_toggle_hovered) => {
                context.request_repaint();
            }
            _ => {}
        }
    }

    fn on_timer(&mut self, timer: TimerId, context: &mut Context) {
        match timer {
            REFRESH_TIMER => self.start_fetch(context),
            ALERT_TIMER => self.arm_alert(context),
            FLASH_TIMER | STAGE_TIMER => self.alert.on_timer(timer, context),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{draw::COLUMNS, forecast::rain_line, *};

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
}
