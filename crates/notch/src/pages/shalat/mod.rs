//! Prayer times for a city, calculated on this PC: the next prayer and today's times, a mark in the collapsed pill
//! before a prayer, and a short message when it starts.

use crate::{
    activity::{Flash, Presence},
    gfx::{Align, Canvas, Color, FontStyle, Fonts, Point, Rect, Size, UI_FONT, Weight},
    host::Settings,
    page::{Context, DEFAULT_CONTENT_HEIGHT, Frame, Mode, Page, PageSizes, Priority, TimerId},
    ui::{self, Icon},
};
use config::Config;
use salah::{DateTime, Utc};
use schedule::Schedule;
use text::{badge_text, clock_text, short_badge_text};

mod arc;
mod config;
mod day;
mod schedule;
#[cfg(test)]
mod tests;
mod text;

mod times;

const CONTENT_WIDTH: f32 = 508.0;

const RESTING_WIDTH: f32 = 150.0;

const BADGE_MINUTES: i64 = 30;

const BADGE_ICON_SIZE: f32 = 14.0;

const BADGE_ICON_GAP: f32 = 4.0;

const REFRESH_TIMER: TimerId = TimerId(1);

const SMALL: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Regular, Align::Leading);

const COUNTDOWN: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Medium, Align::Leading);

const BADGE: FontStyle = FontStyle::new(UI_FONT, 13.0, Weight::Medium, Align::Leading);

const EMPTY_TITLE: FontStyle = FontStyle::new(UI_FONT, 14.0, Weight::Medium, Align::Leading);

const GRAY: Color = Color::rgb(0.604, 0.604, 0.651);

const DIM: Color = Color::rgb(0.431, 0.431, 0.478);

const LIGHT: Color = Color::rgb(0.9, 0.9, 0.92);

const GREEN: Color = Color::rgb(0.388, 0.902, 0.745);

/// The moon, a gap and `text`.
fn badge_width(fonts: &Fonts, widest_text: &str) -> f32 {
    BADGE_ICON_SIZE + BADGE_ICON_GAP + fonts.measure(&BADGE, widest_text).ceil()
}

pub struct ShalatPage {
    config: Config,
    schedule: Option<Schedule>,
    is_shown: bool,
    alert: Flash,
    alert_title: String,
    alert_value: String,
    alert_width: f32,
    badge: Option<String>,
    badge_width: f32,
    badge_short: Option<String>,
    badge_short_width: f32,
    last_announced: Option<DateTime<Utc>>,
    /// Snapshots and tests fix the clock here; `None` uses the real one.
    fixed_now: Option<DateTime<Utc>>,
}

impl ShalatPage {
    pub fn new() -> Self {
        Self {
            config: Config::from_settings(&Settings::load()),
            schedule: None,
            is_shown: false,
            alert: Flash::default(),
            alert_title: String::new(),
            alert_value: String::new(),
            alert_width: 0.0,
            badge: None,
            badge_width: 0.0,
            badge_short: None,
            badge_short_width: 0.0,
            last_announced: None,
            fixed_now: None,
        }
    }

    fn now(&self) -> DateTime<Utc> {
        self.fixed_now.unwrap_or_else(Utc::now)
    }

    #[cfg(test)]
    pub fn sample() -> Self {
        Self::sample_at(|today| today[2] + salah::Duration::minutes(10))
    }

    /// Eight minutes before Asr.
    #[cfg(test)]
    pub fn sample_soon() -> Self {
        Self::sample_at(|today| today[3] - salah::Duration::minutes(8))
    }

    /// Forty minutes after Isha, so the next prayer is tomorrow's Fajr.
    #[cfg(test)]
    pub fn sample_after_isha() -> Self {
        Self::sample_at(|today| today[5] + salah::Duration::minutes(40))
    }

    #[cfg(test)]
    fn sample_at(moment: impl Fn(&[DateTime<Utc>; 6]) -> DateTime<Utc>) -> Self {
        use places::{CITY_KEY, COUNTRY_KEY, LATITUDE_KEY, LONGITUDE_KEY};
        use salah::Local;
        use schedule::compute;
        let get = |key: &str| match key {
            CITY_KEY => Some("Pekanbaru, ID".to_owned()),
            LATITUDE_KEY => Some("0.517".to_owned()),
            LONGITUDE_KEY => Some("101.442".to_owned()),
            COUNTRY_KEY => Some("ID".to_owned()),
            _ => None,
        };
        let mut page = Self::new();
        page.config = Config::read(&get);
        let now = Utc::now();
        let place = page.config.place.clone().expect("the sample has a place");
        let today = times::day_times(now.with_timezone(&Local).date_naive(), &place, &page.config.rules)
            .expect("the sample's place has a sunrise and a sunset");
        let at = moment(&today);
        page.schedule = compute(&page.config, at, at.with_timezone(&Local).date_naive());
        page.fixed_now = Some(at);
        page
    }

    #[cfg(test)]
    pub fn sample_badge(fonts: &crate::gfx::Fonts) -> Self {
        let mut page = Self::sample();
        page.set_badge(fonts, times::Prayer::Asr, 24 * 60);
        page
    }

    #[cfg(test)]
    pub fn sample_alert(fonts: &crate::gfx::Fonts) -> Self {
        let mut page = Self::sample();
        page.alert = Flash::visible();
        page.alert_title = "Time for Asr".to_owned();
        page.alert_value = "15:19".to_owned();
        page.alert_width = ui::message_width(fonts, &page.alert_title, Some(&page.alert_value));
        page
    }

    fn draw_badge(&self, canvas: &Canvas, area: Rect, scale: f32, opacity: f32, text: &str) {
        let icon_center = Point::new(area.left + BADGE_ICON_SIZE * scale / 2.0, area.center_y());
        Icon::Moon.draw(canvas, icon_center, BADGE_ICON_SIZE * scale, GREEN, opacity);
        let text_area =
            Rect::new(area.left + (BADGE_ICON_SIZE + BADGE_ICON_GAP) * scale, area.top, area.right, area.bottom);
        canvas.text(&BADGE, text, text_area, GREEN.faded(opacity));
    }

    fn draw_empty(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        let (bounds, scale) = (frame.bounds, frame.scale);
        let center = crate::gfx::Point::new(bounds.left + 22.0 * scale, bounds.center_y());
        Icon::Clock.draw(canvas, center, 22.0 * scale, GREEN, opacity);
        let left = bounds.left + 52.0 * scale;
        let title = Rect::new(left, bounds.center_y() - 20.0 * scale, bounds.right, bounds.center_y());
        let (headline, hint_text) = if self.config.place.is_some() {
            ("No prayer times today", "The sun does not rise and set at this place today.")
        } else {
            ("Choose your city", "Open the gear in the header and search for it. Times are worked out on this PC.")
        };
        canvas.text(&EMPTY_TITLE, headline, title, Color::WHITE.faded(opacity));
        let hint = Rect::new(left, bounds.center_y() + 2.0 * scale, bounds.right, bounds.center_y() + 20.0 * scale);
        canvas.text(&SMALL, hint_text, hint, GRAY.faded(opacity));
    }
}

impl Page for ShalatPage {
    fn id(&self) -> &'static str {
        "shalat"
    }

    fn name(&self) -> &'static str {
        "Shalat"
    }

    fn is_opt_in(&self) -> bool {
        true
    }

    fn priority(&self, mode: Mode) -> Option<Priority> {
        (mode == Mode::Expanded).then_some(Priority::BACKGROUND)
    }

    fn sizes(&self) -> PageSizes {
        PageSizes { resting_width: RESTING_WIDTH, expanded: Size::new(CONTENT_WIDTH, DEFAULT_CONTENT_HEIGHT) }
    }

    /// Lowest rank sits nearest the clock: wave 0, Caps Lock 1, muted 2, timer 3, event 4, prayer 5.
    fn header_status_rank(&self) -> u8 {
        5
    }

    fn header_status_width(&self) -> Option<f32> {
        self.badge.as_ref().map(|_| Icon::Moon.size())
    }

    fn draw_header_status(&self, canvas: &Canvas, area: Rect, scale: f32, opacity: f32) {
        if self.badge.is_none() {
            return;
        }
        let center = Point::new(area.center_x(), area.center_y());
        Icon::Moon.draw(canvas, center, Icon::Moon.size() * scale, GREEN, opacity);
    }

    fn header_status_page(&self) -> Option<&'static str> {
        Some("shalat")
    }

    fn countdown_seconds(&self) -> Option<u64> {
        let schedule = self.schedule.as_ref()?;
        self.badge.as_ref().map(|_| (schedule.next.1 - self.now()).num_seconds().max(0) as u64)
    }

    fn draw_resting(&self, _canvas: &Canvas, _frame: &Frame, _opacity: f32) {}

    fn draw_expanded(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        let Some(schedule) = &self.schedule else {
            self.draw_empty(canvas, frame, opacity);
            return;
        };
        self.draw_day(canvas, schedule, frame, opacity);
    }

    fn presence(&self) -> Option<Presence> {
        if self.alert.is_visible() {
            Some(Presence::Transient { width: self.alert_width })
        } else {
            self.badge.as_ref().map(|_| Presence::Badge { width: self.badge_width })
        }
    }

    fn draw_presence(&self, canvas: &Canvas, area: Rect, scale: f32, opacity: f32) {
        if self.alert.is_visible() {
            ui::draw_message(canvas, area, opacity, Icon::Moon, GREEN, &self.alert_title, Some(&self.alert_value));
        } else if let Some(text) = &self.badge {
            self.draw_badge(canvas, area, scale, opacity, text);
        }
    }

    fn short_presence_width(&self) -> Option<f32> {
        self.badge_short.as_ref().map(|_| self.badge_short_width)
    }

    fn draw_short_presence(&self, canvas: &Canvas, area: Rect, scale: f32, opacity: f32) {
        if self.alert.is_visible() {
            self.draw_presence(canvas, area, scale, opacity);
        } else if let Some(text) = self.badge_short.as_ref().or(self.badge.as_ref()) {
            self.draw_badge(canvas, area, scale, opacity, text);
        }
    }

    fn on_start(&mut self, context: &mut Context) {
        self.config = Config::from_settings(context.settings);
        self.refresh(context);
    }

    fn on_settings_changed(&mut self, context: &mut Context) {
        self.config = Config::from_settings(context.settings);
        self.refresh(context);
    }

    fn on_shown_changed(&mut self, is_shown: bool, context: &mut Context) {
        self.is_shown = is_shown;
        self.refresh(context);
    }

    fn on_timer(&mut self, timer: TimerId, context: &mut Context) {
        if self.alert.on_timer(timer, context) {
            return;
        }
        if timer == REFRESH_TIMER {
            self.refresh(context);
        }
    }
}
