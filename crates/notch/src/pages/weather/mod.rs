mod alert;
mod badge;
mod click;
mod draw;
mod fetch;
mod forecast;
mod input;
mod link;
mod message;
mod picture;
mod refresh;
#[cfg(test)]
mod sample;
mod symbol;
#[cfg(test)]
mod tests;

use std::sync::{Arc, Mutex};

use places::Location;
use salah::{DateTime, Utc};

use self::{
    alert::{Alert, BADGE_TIMER},
    fetch::Outcome,
    forecast::Forecast,
};
use crate::{
    activity::{FLASH_TIMER, Presence},
    gfx::{Canvas, Point, Rect, Size},
    page::{Context, Frame, Mode, Page, PageSizes, PointerEvent, Priority, TimerId},
    ui::{HoverTarget, Icon, palette::ACCENT_BLUE_LIGHT},
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

/// What the pointer can be on: the Today/Week pill or one of the cells.
#[derive(Clone, Copy, Debug, PartialEq)]
enum WeatherTarget {
    Toggle,
    Cell(usize),
}

pub struct WeatherPage {
    location: Option<Location>,
    forecast: Option<Forecast>,
    is_offline: bool,
    /// The request failed for a reason other than the network (a bad status or an unreadable answer).
    is_service_error: bool,
    /// The pill reads "Week" and the cells show the days; otherwise it reads "Today" and they show the hours.
    shows_days: bool,
    /// The pill or the hour or day cell under the pointer.
    hover: HoverTarget<WeatherTarget>,
    /// The content width of the message on show, so the page hugs it; `None` while the forecast shows.
    message_width: Option<f32>,
    /// The user's `weather.rain_alert` choice.
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
            hover: HoverTarget::default(),
            message_width: None,
            rain_alert: true,
            alerted_for: None,
            alert: Alert::new(),
            outcome: Arc::new(Mutex::new(None)),
            fixed_now: None,
        }
    }

    fn now(&self) -> DateTime<Utc> {
        self.fixed_now.unwrap_or_else(Utc::now)
    }
}

impl Page for WeatherPage {
    fn id(&self) -> &'static str {
        "weather"
    }

    fn name(&self) -> String {
        crate::i18n::tr!("page.weather").into_owned()
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
        let bounds = frame.bounds;
        if let Some(message) = self.message() {
            Self::draw_message_row(canvas, bounds, &message, opacity);
            return;
        }
        let Some(forecast) = &self.forecast else {
            Self::draw_message(canvas, bounds, "Loading the weather", "", opacity);
            return;
        };
        self.draw_left(canvas, forecast, Self::left_area(bounds), opacity);
        self.draw_toggle(canvas, Self::toggle_area(bounds), opacity);
        self.draw_columns(canvas, forecast, bounds, opacity);
    }

    fn presence(&self) -> Option<Presence> {
        self.alert.presence(self.now())
    }

    fn draw_presence(&self, canvas: &Canvas, area: Rect, opacity: f32) {
        self.alert.draw(canvas, area, opacity, self.now());
    }

    fn short_presence_width(&self) -> Option<f32> {
        self.alert.short_badge_width(self.now())
    }

    fn draw_short_presence(&self, canvas: &Canvas, area: Rect, opacity: f32) {
        self.alert.draw_short_badge(canvas, area, opacity, self.now());
    }

    /// Lowest rank sits nearest the clock: wave 0, Caps Lock 1, muted 2, timer 3, event 4, prayer 5, rain 6.
    fn header_status_rank(&self) -> u8 {
        6
    }

    fn header_status_width(&self) -> Option<f32> {
        self.alert.badge_icon(self.now()).map(Icon::size)
    }

    fn header_status_page(&self) -> Option<&'static str> {
        self.alert.has_badge(self.now()).then_some(self.id())
    }

    fn draw_header_status(&self, canvas: &Canvas, area: Rect, opacity: f32) {
        if let Some(icon) = self.alert.badge_icon(self.now()) {
            let center = Point::new(area.center_x(), area.center_y());
            icon.draw(canvas, center, icon.size(), ACCENT_BLUE_LIGHT, opacity);
        }
    }

    fn countdown_seconds(&self) -> Option<u64> {
        self.alert.countdown_seconds(self.now())
    }

    fn on_open_changed(&mut self, is_open: bool, context: &mut Context) {
        self.alert.on_open_changed(is_open, context);
    }

    fn on_presence_pointer(&mut self, event: PointerEvent, area: Rect, context: &mut Context) {
        self.alert.on_pointer(event, area, self.now(), context);
    }

    fn on_start(&mut self, context: &mut Context) {
        self.read_settings(context);
        self.refresh_message_width(context.fonts());
        self.start_fetch(context);
    }

    fn on_language_changed(&mut self, context: &mut Context) {
        self.refresh_message_width(context.fonts());
        self.alert.remeasure_badge(context.fonts());
        context.request_repaint();
    }

    fn on_settings_changed(&mut self, context: &mut Context) {
        let previous = self.location.clone();
        self.read_settings(context);
        if self.location != previous {
            self.forecast = None;
            if matches!(self.hover.get(), Some(WeatherTarget::Cell(_))) {
                self.hover.clear();
            }
            self.alert.clear(context);
            self.is_offline = false;
            self.is_service_error = false;
            self.alerted_for = None;
            self.refresh_message_width(context.fonts());
            self.start_fetch(context);
            context.request_repaint();
        } else if self.rain_alert {
            self.arm_alert(context);
        } else {
            context.kill_timer(ALERT_TIMER);
            self.alert.clear(context);
        }
    }

    fn on_notify(&mut self, context: &mut Context) {
        self.apply_outcome(context);
    }

    fn on_pointer(&mut self, event: PointerEvent, frame: &Frame, context: &mut Context) {
        self.handle_pointer(event, frame, context);
    }

    fn on_timer(&mut self, timer: TimerId, context: &mut Context) {
        match timer {
            REFRESH_TIMER => self.start_fetch(context),
            ALERT_TIMER => self.arm_alert(context),
            FLASH_TIMER | BADGE_TIMER => self.alert.on_timer(timer, self.now(), context),
            _ => {}
        }
    }
}
