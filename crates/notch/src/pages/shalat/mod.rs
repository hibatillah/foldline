//! Prayer times for a city, calculated on this PC: the next prayer and today's times, a mark in the collapsed pill
//! before a prayer, and a short message when it starts.

use self::times::PRAYERS;
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
use config::{BADGE_KEY, Config, HANAFI_KEY, MESSAGE_KEY, METHOD_KEY};
use places::CITY_KEY;
use salah::{DateTime, Utc};
use schedule::Schedule;
use text::{badge_text, clock_text, remaining_text};

mod config;
mod schedule;
#[cfg(test)]
mod tests;
mod text;

mod times;

const CONTENT_WIDTH: f32 = 508.0;

const RESTING_WIDTH: f32 = 150.0;

const SIDE_WIDTH: f32 = 150.0;

const DIVIDER_GAP: f32 = 12.0;

const CELL_RADIUS: f32 = 10.0;

const BADGE_MINUTES: i64 = 30;

const BADGE_TRAILING: f32 = 14.0;

const MAX_HITS: usize = 3;

const REFRESH_TIMER: TimerId = TimerId(1);

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
        use places::{COUNTRY_KEY, LATITUDE_KEY, LONGITUDE_KEY};
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
        places::search(query, MAX_HITS)
            .into_iter()
            .map(|city| SearchHit { label: city.label(), values: places::values_of(&city) })
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
