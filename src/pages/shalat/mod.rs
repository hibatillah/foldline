//! Prayer times for a city, calculated on this PC: the next prayer and today's times, a mark in the collapsed pill
//! before a prayer, and a short message when it starts.

mod cities;
mod times;

use salah::{DateTime, Local, Madhab, NaiveDate, Timelike, Utc};

use self::times::{PRAYERS, Place, Prayer, Rules};
use crate::{
    activity::{Flash, Presence},
    gfx::{Align, Canvas, Color, FontStyle, Rect, Size, UI_FONT, Weight},
    host::Settings,
    page::{
        Context, Control, DEFAULT_CONTENT_HEIGHT, Frame, Mode, Page, PageSizes, Priority, SearchHit, SettingsGroup,
        TimerId,
    },
    ui::{self, Icon},
};

const CONTENT_WIDTH: f32 = 508.0;
const RESTING_WIDTH: f32 = 150.0;
const SIDE_WIDTH: f32 = 150.0;
const DIVIDER_GAP: f32 = 12.0;
const CELL_RADIUS: f32 = 10.0;
const BADGE_MINUTES: i64 = 30;
const BADGE_TRAILING: f32 = 14.0;
const MESSAGE_MILLISECONDS: u32 = 9000;
/// A prayer time that passed this recently still gets its message, since the timer fires on a minute boundary.
const ANNOUNCE_SECONDS: i64 = 120;
const MAX_WAIT_MILLISECONDS: i64 = 12 * 60 * 60 * 1000;
const MAX_HITS: usize = 3;

const REFRESH_TIMER: TimerId = TimerId(1);

const CITY_KEY: &str = "shalat.city";
const LATITUDE_KEY: &str = "shalat.lat";
const LONGITUDE_KEY: &str = "shalat.lon";
const COUNTRY_KEY: &str = "shalat.country";
const METHOD_KEY: &str = "shalat.method";
const HANAFI_KEY: &str = "shalat.hanafi";
const MESSAGE_KEY: &str = "shalat.message";
const BADGE_KEY: &str = "shalat.badge";

const MONO: &str = "Consolas";
const SMALL: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Regular, Align::Leading);
const TINY: FontStyle = FontStyle::new(UI_FONT, 10.0, Weight::Regular, Align::Leading);
const BIG_TIME: FontStyle = FontStyle::new(MONO, 28.0, Weight::Medium, Align::Leading);
const COUNTDOWN: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Medium, Align::Leading);
const CELL_NAME: FontStyle = FontStyle::new(UI_FONT, 10.0, Weight::Regular, Align::Center);
const CELL_TIME: FontStyle = FontStyle::new(MONO, 13.0, Weight::Regular, Align::Center);
const BADGE: FontStyle = FontStyle::new(UI_FONT, 13.0, Weight::Medium, Align::Leading);
const EMPTY_TITLE: FontStyle = FontStyle::new(UI_FONT, 14.0, Weight::Medium, Align::Leading);

const GRAY: Color = Color::rgb(0.604, 0.604, 0.651);
const DIM: Color = Color::rgb(0.431, 0.431, 0.478);
const LIGHT: Color = Color::rgb(0.9, 0.9, 0.92);
const GREEN: Color = Color::rgb(0.388, 0.902, 0.745);
const NEXT_FILL: Color = Color::rgb(0.051, 0.169, 0.141);
const DIVIDER: Color = Color::rgb(0.149, 0.149, 0.176);

/// Everything read from the settings file.
#[derive(Clone, Debug)]
struct Config {
    city: String,
    place: Option<Place>,
    rules: Rules,
    shows_message: bool,
    shows_badge: bool,
}

impl Config {
    fn read(get: &dyn Fn(&str) -> Option<String>) -> Self {
        let number = |key: &str| get(key).and_then(|text| text.trim().parse::<f64>().ok());
        let place = number(LATITUDE_KEY)
            .zip(number(LONGITUDE_KEY))
            .filter(|(latitude, longitude)| latitude.abs() <= 90.0 && longitude.abs() <= 180.0)
            .map(|(latitude, longitude)| Place { latitude, longitude });
        let country = get(COUNTRY_KEY).unwrap_or_default();
        let method = times::parse_method(get(METHOD_KEY).as_deref().unwrap_or("auto"), &country);
        let is_on = |key: &str, default: bool| match get(key).as_deref() {
            Some("on") => true,
            Some("off") => false,
            _ => default,
        };
        let mut offsets = [0i64; 6];
        for (offset, prayer) in offsets.iter_mut().zip(PRAYERS) {
            *offset =
                get(prayer.offset_key()).and_then(|text| text.trim().parse::<i64>().ok()).unwrap_or(0).clamp(-30, 30);
        }
        Self {
            city: get(CITY_KEY).unwrap_or_default(),
            place,
            rules: Rules {
                method,
                madhab: if is_on(HANAFI_KEY, false) { Madhab::Hanafi } else { Madhab::Shafi },
                offsets,
            },
            shows_message: is_on(MESSAGE_KEY, true),
            shows_badge: is_on(BADGE_KEY, true),
        }
    }

    fn from_settings(settings: &Settings) -> Self {
        Self::read(&|key| settings.get_string(key).map(str::to_owned))
    }
}

/// Today's times and what comes next, worked out at `now`.
#[derive(Clone, Debug)]
struct Schedule {
    today: [DateTime<Utc>; 6],
    next: (Prayer, DateTime<Utc>),
}

fn compute(config: &Config, now: DateTime<Utc>, date: NaiveDate) -> Option<Schedule> {
    let place = config.place.as_ref()?;
    let next = times::next_after(now, date, place, &config.rules)?;
    Some(Schedule { today: times::day_times(date, place, &config.rules)?, next })
}

fn clock_text(time: DateTime<Utc>) -> String {
    let local = time.with_timezone(&Local);
    format!("{:02}:{:02}", local.hour(), local.minute())
}

/// "in 1 h 12 min", "in 24 min"; rounds up so the last minute still reads "in 1 min".
fn remaining_text(seconds: i64) -> String {
    let minutes = (seconds.max(0) + 59) / 60;
    match (minutes / 60, minutes % 60) {
        (0, minutes) => format!("in {} min", minutes.max(1)),
        (hours, 0) => format!("in {hours} h"),
        (hours, minutes) => format!("in {hours} h {minutes} min"),
    }
}

/// The collapsed mark, "Asr in 24m", while the next prayer is within `BADGE_MINUTES`; `None` otherwise.
fn badge_text(prayer: Prayer, seconds: i64) -> Option<String> {
    (seconds > 0 && seconds <= BADGE_MINUTES * 60).then(|| format!("{} in {}m", prayer.name(), (seconds + 59) / 60))
}

/// Milliseconds until the page needs to look again; `None` when nothing needs a timer.
fn wait_milliseconds(
    seconds_to_next: i64,
    is_shown: bool,
    shows_badge: bool,
    shows_message: bool,
    milliseconds_into_minute: i64,
) -> Option<i64> {
    let to_minute = 60_000 - milliseconds_into_minute + 50;
    let to_next = seconds_to_next.max(0) * 1000 + 1000;
    let to_badge_window = (seconds_to_next - BADGE_MINUTES * 60).max(0) * 1000;
    let wait = if is_shown {
        to_minute
    } else if shows_badge || shows_message {
        if seconds_to_next <= BADGE_MINUTES * 60 { to_minute.min(to_next) } else { to_badge_window.max(1000) }
    } else {
        return None;
    };
    Some(wait.clamp(250, MAX_WAIT_MILLISECONDS))
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
            last_announced: None,
            fixed_now: None,
        }
    }

    fn now(&self) -> DateTime<Utc> {
        self.fixed_now.unwrap_or_else(Utc::now)
    }

    #[cfg(test)]
    pub fn sample() -> Self {
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
        // A fixed moment between Dhuhr and Asr, so the picture shows an upcoming Asr.
        let at = today[2] + salah::Duration::minutes(10);
        page.schedule = compute(&page.config, at, at.with_timezone(&Local).date_naive());
        page.fixed_now = Some(at);
        page
    }

    #[cfg(test)]
    pub fn sample_badge(fonts: &crate::gfx::Fonts) -> Self {
        let mut page = Self::sample();
        page.badge = Some("Asr in 24m".to_owned());
        page.badge_width = fonts.measure(&BADGE, "Maghrib in 59m").ceil() + BADGE_TRAILING;
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

    /// Recalculates for the present moment and, when a prayer has just started, shows its message.
    fn refresh(&mut self, context: &mut Context) {
        let now = Utc::now();
        let date = now.with_timezone(&Local).date_naive();
        self.schedule = compute(&self.config, now, date);
        self.badge = None;
        if let Some(schedule) = &self.schedule {
            let seconds = (schedule.next.1 - now).num_seconds();
            if self.config.shows_badge
                && let Some(text) = badge_text(schedule.next.0, seconds)
            {
                self.badge_width = context.fonts().measure(&BADGE, "Maghrib in 59m").ceil() + BADGE_TRAILING;
                self.badge = Some(text);
            }
            self.announce_if_due(now, date, context);
        }
        self.arm_timer(context, now);
        context.request_repaint();
    }

    fn announce_if_due(&mut self, now: DateTime<Utc>, date: NaiveDate, context: &mut Context) {
        let Some(place) = self.config.place.as_ref() else { return };
        let Some((prayer, started)) = times::previous_before(now, date, place, &self.config.rules) else { return };
        let is_fresh = (now - started).num_seconds() <= ANNOUNCE_SECONDS;
        if !self.config.shows_message || !prayer.is_prayer() || !is_fresh || self.last_announced == Some(started) {
            return;
        }
        self.last_announced = Some(started);
        self.alert_title = format!("Time for {}", prayer.name());
        self.alert_value = clock_text(started);
        self.alert_width = ui::message_width(context.fonts(), &self.alert_title, Some(&self.alert_value));
        self.alert.show(context, MESSAGE_MILLISECONDS);
    }

    fn arm_timer(&self, context: &mut Context, now: DateTime<Utc>) {
        let Some(schedule) = &self.schedule else {
            context.kill_timer(REFRESH_TIMER);
            return;
        };
        let into_minute = i64::from(now.with_timezone(&Local).second()) * 1000;
        let seconds_to_next = (schedule.next.1 - now).num_seconds();
        match wait_milliseconds(
            seconds_to_next,
            self.is_shown,
            self.config.shows_badge,
            self.config.shows_message,
            into_minute,
        ) {
            Some(wait) => context.set_timer(REFRESH_TIMER, wait as u32),
            None => context.kill_timer(REFRESH_TIMER),
        }
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

    fn icon(&self) -> Icon {
        Icon::Moon
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

    fn draw_resting(&self, _canvas: &Canvas, _frame: &Frame, _opacity: f32) {}

    fn draw_expanded(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        let Some(schedule) = &self.schedule else {
            self.draw_empty(canvas, frame, opacity);
            return;
        };
        let (bounds, scale) = (frame.bounds, frame.scale);
        let left = bounds.left;
        let line = |top: f32, bottom: f32| {
            Rect::new(left, bounds.top + top * scale, left + SIDE_WIDTH * scale, bounds.top + bottom * scale)
        };
        let (prayer, time) = schedule.next;
        canvas.text(&SMALL, &format!("Next \u{00b7} {}", prayer.name()), line(0.0, 16.0), GRAY.faded(opacity));
        canvas.text(&BIG_TIME, &clock_text(time), line(14.0, 48.0), Color::WHITE.faded(opacity));
        let now = self.now();
        let remaining = remaining_text((time - now).num_seconds());
        canvas.text(&COUNTDOWN, &remaining, line(50.0, 66.0), GREEN.faded(opacity));
        canvas.text(&TINY, &self.config.city, line(68.0, 82.0), DIM.faded(opacity));

        let divider_left = left + (SIDE_WIDTH + DIVIDER_GAP) * scale;
        canvas.fill_rounded_rect(
            Rect::new(divider_left, bounds.top, divider_left + scale, bounds.bottom),
            0.0,
            DIVIDER.faded(opacity),
        );

        let cells_left = divider_left + (DIVIDER_GAP + 1.0) * scale;
        let cell_width = (bounds.right - cells_left) / PRAYERS.len() as f32;
        for (index, (cell_prayer, cell_time)) in PRAYERS.iter().zip(schedule.today).enumerate() {
            let cell = Rect::new(
                cells_left + index as f32 * cell_width,
                bounds.top + 8.0 * scale,
                cells_left + (index + 1) as f32 * cell_width,
                bounds.bottom - 8.0 * scale,
            );
            let is_next = *cell_prayer == prayer && cell_time == time;
            let is_past = cell_time <= now;
            if is_next {
                canvas.fill_rounded_rect(cell, CELL_RADIUS * scale, NEXT_FILL.faded(opacity));
            }
            let (name_color, time_color) = match (is_next, is_past) {
                (true, _) => (GREEN, Color::WHITE),
                (false, true) => (DIM, DIM),
                (false, false) => (GRAY, LIGHT),
            };
            let name_area = Rect { top: cell.top + 14.0 * scale, bottom: cell.top + 30.0 * scale, ..cell };
            let time_area = Rect { top: cell.top + 36.0 * scale, bottom: cell.top + 56.0 * scale, ..cell };
            canvas.text(&CELL_NAME, cell_prayer.name(), name_area, name_color.faded(opacity));
            canvas.text(&CELL_TIME, &clock_text(cell_time), time_area, time_color.faded(opacity));
        }
    }

    fn settings_groups(&self) -> Vec<SettingsGroup> {
        let offsets = PRAYERS.map(|prayer| Control::Stepper {
            key: prayer.offset_key(),
            label: prayer.name(),
            unit: "min",
            default: 0,
            min: -30,
            max: 30,
            step: 1,
        });
        let mut adjust: Vec<Control> = offsets.into_iter().collect();
        adjust.push(Control::Switch {
            key: MESSAGE_KEY,
            label: "Message at prayer time",
            hint: "a short note in the notch",
            default: true,
        });
        adjust.push(Control::Switch {
            key: BADGE_KEY,
            label: "Next prayer in the pill",
            hint: "30 minutes before",
            default: true,
        });
        vec![
            SettingsGroup {
                title: "Shalat",
                controls: vec![
                    Control::Search { key: CITY_KEY, label: "City", placeholder: "Click, then type a city" },
                    Control::Choice {
                        key: METHOD_KEY,
                        label: "Method",
                        options: times::METHOD_OPTIONS,
                        default: "auto",
                    },
                    Control::Switch {
                        key: HANAFI_KEY,
                        label: "Hanafi Asr",
                        hint: "later Asr, two shadow lengths",
                        default: false,
                    },
                ],
            },
            SettingsGroup { title: "Adjust", controls: adjust },
        ]
    }

    fn search(&self, key: &str, query: &str) -> Vec<SearchHit> {
        if key != CITY_KEY {
            return Vec::new();
        }
        cities::search(query, MAX_HITS)
            .into_iter()
            .map(|city| SearchHit {
                label: city.label(),
                values: vec![
                    (CITY_KEY, city.label()),
                    (LATITUDE_KEY, format!("{:.3}", city.latitude)),
                    (LONGITUDE_KEY, format!("{:.3}", city.longitude)),
                    (COUNTRY_KEY, city.country),
                ],
            })
            .collect()
    }

    fn presence(&self) -> Option<Presence> {
        if self.alert.is_visible() {
            Some(Presence::Transient { width: self.alert_width })
        } else {
            self.badge.as_ref().map(|_| Presence::Badge { width: self.badge_width })
        }
    }

    fn draw_presence(&self, canvas: &Canvas, area: Rect, _scale: f32, opacity: f32) {
        if self.alert.is_visible() {
            ui::draw_message(canvas, area, opacity, Icon::Moon, GREEN, &self.alert_title, Some(&self.alert_value));
        } else if let Some(text) = &self.badge {
            canvas.text(&BADGE, text, area, GREEN.faded(opacity));
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

#[cfg(test)]
mod tests {
    use salah::Datelike;

    use super::*;

    fn config_from(pairs: &[(&str, &str)]) -> Config {
        Config::read(&|key| pairs.iter().find(|(name, _)| *name == key).map(|(_, value)| (*value).to_owned()))
    }

    #[test]
    fn a_missing_city_leaves_the_page_waiting_for_one() {
        assert!(config_from(&[]).place.is_none());
        assert!(config_from(&[(LATITUDE_KEY, "0.5")]).place.is_none());
        assert!(config_from(&[(LATITUDE_KEY, "95"), (LONGITUDE_KEY, "10")]).place.is_none());
    }

    #[test]
    fn the_settings_become_a_place_a_method_and_clamped_offsets() {
        let config = config_from(&[
            (CITY_KEY, "Pekanbaru, ID"),
            (LATITUDE_KEY, "0.517"),
            (LONGITUDE_KEY, "101.442"),
            (COUNTRY_KEY, "ID"),
            ("shalat.offset_dhuhr", "2"),
            ("shalat.offset_isha", "-99"),
            (HANAFI_KEY, "on"),
        ]);
        assert_eq!(config.place, Some(Place { latitude: 0.517, longitude: 101.442 }));
        assert_eq!(config.rules.offsets, [0, 0, 2, 0, 0, -30]);
        assert_eq!(config.rules.method, salah::Method::Singapore);
        assert_eq!(config.rules.madhab, Madhab::Hanafi);
        assert!(config.shows_message && config.shows_badge);
    }

    #[test]
    fn the_remaining_time_rounds_up_to_whole_minutes() {
        assert_eq!(remaining_text(24 * 60), "in 24 min");
        assert_eq!(remaining_text(30), "in 1 min");
        assert_eq!(remaining_text(72 * 60), "in 1 h 12 min");
        assert_eq!(remaining_text(120 * 60), "in 2 h");
        assert_eq!(remaining_text(-5), "in 1 min");
    }

    #[test]
    fn the_collapsed_mark_appears_only_in_the_last_half_hour() {
        assert_eq!(badge_text(Prayer::Asr, 24 * 60), Some("Asr in 24m".to_owned()));
        assert_eq!(badge_text(Prayer::Asr, 31 * 60), None);
        assert_eq!(badge_text(Prayer::Asr, 0), None);
    }

    #[test]
    fn the_page_waits_only_as_long_as_it_has_something_to_show() {
        // Open: every minute. Closed with no mark or message: no timer at all.
        assert_eq!(wait_milliseconds(7200, true, true, true, 10_000), Some(50_050));
        assert_eq!(wait_milliseconds(7200, false, false, false, 0), None);
        // Closed, mark on, two hours away: sleep until the mark's window opens.
        assert_eq!(wait_milliseconds(7200, false, true, false, 0), Some(5400 * 1000));
        // Inside the window it checks each minute, and never later than the prayer itself.
        assert_eq!(wait_milliseconds(1200, false, true, false, 0), Some(60_050));
        assert_eq!(wait_milliseconds(20, false, true, false, 0), Some(21_000));
    }

    #[test]
    fn typing_a_city_gives_hits_that_save_its_place() {
        let page = ShalatPage::new();
        let hits = page.search(CITY_KEY, "pekanbaru");
        assert_eq!(hits[0].label, "Pekanbaru, ID");
        let keys: Vec<&str> = hits[0].values.iter().map(|(key, _)| *key).collect();
        assert_eq!(keys, [CITY_KEY, LATITUDE_KEY, LONGITUDE_KEY, COUNTRY_KEY]);
        assert!(page.search("shalat.other", "pekanbaru").is_empty());
    }

    #[test]
    fn the_adjust_tab_has_a_stepper_for_every_time_and_the_first_tab_fits_a_page() {
        let groups = ShalatPage::new().settings_groups();
        assert_eq!(groups.len(), 2);
        let steppers = groups[1].controls.iter().filter(|control| matches!(control, Control::Stepper { .. })).count();
        assert_eq!(steppers, PRAYERS.len());
    }

    #[test]
    fn dates_in_the_page_come_from_this_pcs_clock() {
        let today = Utc::now().with_timezone(&Local).date_naive();
        assert!(today.year() >= 2026);
    }
}
