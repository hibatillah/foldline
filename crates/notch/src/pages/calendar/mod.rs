//! Calendar links and cached ICS data live beside settings; collapsed reminders precede events by 15 minutes.

use self::{
    agenda::{Agenda, Reminder, build, list, reminder},
    dates::{day_of, week_start},
    ics::{Event, Occurrence, occurrences_on, parse},
};
use crate::{
    activity::Presence,
    gfx::{Canvas, Color, Fonts, Point, Rect, Size},
    page::{Context, Control, Frame, Mode, Notifier, Page, PageSizes, PointerEvent, Priority, TimerId},
    platform,
    ui::Icon,
};
use geometry::{day_at, day_cell, events_area, is_in_week_of, row_rect, widest_form};
use local_time::{SystemClock, local_now};
use settings::{LinkChange, link_change, minutes_to_milliseconds};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use style::*;
use windows::Win32::System::SystemInformation::GetLocalTime;

mod draw;
mod geometry;
mod local_time;
mod settings;
mod style;

mod agenda;

mod dates;

mod fetch;

mod ics;

const EXPANDED_SIZE: Size = Size::new(508.0, 86.0);

const RESTING_WIDTH: f32 = 150.0;

const LEFT_WIDTH: f32 = 200.0;

const COLUMN_GAP: f32 = 16.0;

const DAY_TOP_PADDING: f32 = 5.0;

const DAY_LETTER_HEIGHT: f32 = 12.0;

const HEADER_TOP: f32 = 8.0;

const TODAY_SIZE: f32 = 24.0;

const DEFAULT_REFRESH_MINUTES: u64 = 15;

const FIRST_FETCH_MILLISECONDS: u32 = 2000;

const REMINDER_TITLE_WIDTH: f32 = 96.0;

const LINK_SETTING: &str = "calendar.ics_url";

const REFRESH_SETTING: &str = "calendar.refresh_minutes";

/// Wakes once a minute, like the clock, so "in 12 min" and the reminder stay right.
const MINUTE_TIMER: TimerId = TimerId(1);

const FETCH_TIMER: TimerId = TimerId(2);

pub struct CalendarPage {
    events: Vec<Event>,
    link: Option<String>,
    storage: Option<PathBuf>,
    fetched: fetch::Outcome,
    notifier: Option<Notifier>,
    today: i64,
    occurrences: Vec<Occurrence>,
    agenda: Agenda,
    reminder: Option<Reminder>,
    reminder_width: f32,
    hovered_join: Option<usize>,
    selected_day: Option<i64>,
    hovered_day: Option<usize>,
    fetch_interval_milliseconds: u32,
}

impl CalendarPage {
    pub fn new() -> Self {
        let storage =
            std::env::var_os("APPDATA").map(|directory| PathBuf::from(directory).join("notch").join("calendar.ics"));
        Self {
            events: Vec::new(),
            link: None,
            storage,
            fetched: Arc::new(Mutex::new(None)),
            notifier: None,
            today: 0,
            occurrences: Vec::new(),
            agenda: build(&[], 0),
            reminder: None,
            reminder_width: 0.0,
            hovered_join: None,
            selected_day: None,
            hovered_day: None,
            fetch_interval_milliseconds: minutes_to_milliseconds(DEFAULT_REFRESH_MINUTES),
        }
    }

    fn refresh(&mut self, fonts: &Fonts) {
        let now = local_now();
        self.today = day_of(now);
        // A day picked in an earlier week is forgotten when the week turns.
        if self.selected_day.is_some_and(|day| !is_in_week_of(day, self.today)) {
            self.selected_day = None;
        }
        self.occurrences = occurrences_on(&self.events, self.today, &SystemClock);
        self.agenda = match self.selected_day {
            Some(day) if day != self.today => list(&occurrences_on(&self.events, day, &SystemClock)),
            _ => build(&self.occurrences, now),
        };
        self.reminder = reminder(&self.occurrences, now);
        if let Some(reminder) = &self.reminder {
            let title = fonts.measure(&BADGE, &reminder.title).min(REMINDER_TITLE_WIDTH).ceil();
            let time = fonts.measure(&BADGE_TIME, &widest_form(&reminder.text)).ceil();
            self.reminder_width = 14.0 + 6.0 + title + 8.0 + time + 6.0;
        }
    }

    #[cfg(test)]
    pub fn sample(fonts: &Fonts) -> Self {
        use dates::civil_from_days;
        let now = local_now();
        let (year, month, day) = civil_from_days(day_of(now));
        let at = |seconds: i64| {
            let (_, _, date) = civil_from_days(day_of(seconds));
            let of_day = seconds.rem_euclid(dates::SECONDS_PER_DAY);
            format!("{year:04}{month:02}{date:02}T{:02}{:02}00", of_day / 3600, of_day / 60 % 60)
        };
        let _ = day;
        let today = |hour: i64, minute: i64| day_of(now) * dates::SECONDS_PER_DAY + hour * 3600 + minute * 60;
        let event = |start: i64, minutes: i64, summary: &str, place: &str| {
            format!(
                "BEGIN:VEVENT\r\nUID:{summary}\r\nDTSTART:{}\r\nDTEND:{}\r\nSUMMARY:{summary}\r\nLOCATION:{place}\r\nEND:VEVENT\r\n",
                at(start),
                at(start + minutes * 60)
            )
        };
        let text = format!(
            "BEGIN:VCALENDAR\r\n{}{}{}END:VCALENDAR\r\n",
            event(today(9, 0), 30, "Morning standup", ""),
            event(now + 12 * 60, 60, "Design review: notch v3", "https://meet.google.com/abc-defg-hij"),
            event(today(23, 0), 60, "Deep learning reading group", ""),
        );
        let mut page = Self::new();
        // A test that removes the link must never delete the real cached calendar.
        page.storage = None;
        page.link = Some("https://example.invalid/calendar.ics".to_owned());
        page.load_events(&text);
        page.refresh(fonts);
        page
    }

    fn arm_minute_timer(context: &mut Context) {
        let time = unsafe { GetLocalTime() };
        let milliseconds_into_minute = u32::from(time.wSecond) * 1000 + u32::from(time.wMilliseconds);
        context.set_timer(MINUTE_TIMER, 60_000 - milliseconds_into_minute + 50);
    }

    fn start_fetch(&self) {
        if let (Some(link), Some(notifier)) = (&self.link, self.notifier) {
            fetch::start(link.clone(), self.fetched.clone(), notifier);
        }
    }

    fn load_events(&mut self, text: &str) {
        self.events = parse(text);
    }

    fn selected(&self) -> i64 {
        self.selected_day.unwrap_or(self.today)
    }
}

impl Page for CalendarPage {
    fn id(&self) -> &'static str {
        "calendar"
    }

    fn name(&self) -> &'static str {
        "Calendar"
    }

    fn icon(&self) -> Icon {
        Icon::Calendar
    }

    fn settings(&self) -> Vec<Control> {
        vec![
            Control::Text {
                key: LINK_SETTING,
                label: "Address (.ics)",
                placeholder: "Click, then paste the secret iCal address",
                is_secret: true,
            },
            Control::Stepper {
                key: REFRESH_SETTING,
                label: "Refresh",
                unit: "min",
                default: DEFAULT_REFRESH_MINUTES as i32,
                min: 1,
                max: 1440,
                step: 5,
            },
        ]
    }

    fn priority(&self, mode: Mode) -> Option<Priority> {
        (mode == Mode::Expanded).then_some(Priority::BACKGROUND)
    }

    fn sizes(&self) -> PageSizes {
        PageSizes { resting_width: RESTING_WIDTH, expanded: EXPANDED_SIZE }
    }

    fn draw_resting(&self, _canvas: &Canvas, _frame: &Frame, _opacity: f32) {}

    fn draw_expanded(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        let (bounds, scale) = (frame.bounds, frame.scale);
        self.draw_week(canvas, bounds, opacity);
        let divider_left = bounds.left + (LEFT_WIDTH + COLUMN_GAP) * scale;
        canvas.fill_rounded_rect(
            Rect::new(divider_left, bounds.top, divider_left + scale, bounds.bottom),
            0.0,
            DIVIDER.faded(opacity),
        );

        if self.agenda.rows.is_empty() {
            let area = events_area(bounds, scale);
            let (message, hint) = if self.link.is_none() {
                ("No calendar linked", Some("Paste your calendar's iCal address in Settings, under Calendar."))
            } else {
                (if self.selected() == self.today { "Nothing for today" } else { "Nothing on this day" }, None)
            };
            let text_top = if hint.is_some() {
                area.center_y() - 26.0 * scale
            } else {
                let icon_center = Point::new(area.center_x(), area.center_y() - 14.0 * scale);
                Icon::Calendar.draw(canvas, icon_center, 22.0 * scale, DIM, opacity);
                icon_center.y + 14.0 * scale
            };
            let text = Rect::new(area.left, text_top, area.right, text_top + 20.0 * scale);
            canvas.text(&EMPTY, message, text, LIGHT.faded(opacity));
            if let Some(hint) = hint {
                let hint_area = Rect::new(area.left + 8.0 * scale, text.bottom, area.right - 8.0 * scale, area.bottom);
                canvas.text(&EMPTY_HINT, hint, hint_area, GRAY.faded(opacity));
            }
            return;
        }
        for (position, row) in self.agenda.rows.iter().enumerate() {
            self.draw_row(canvas, position, row, bounds, self.hovered_join == Some(position), opacity);
        }
    }

    fn presence(&self) -> Option<Presence> {
        self.reminder.as_ref().map(|_| Presence::Badge { width: self.reminder_width })
    }

    fn draw_presence(&self, canvas: &Canvas, area: Rect, scale: f32, opacity: f32) {
        let Some(reminder) = &self.reminder else { return };
        let icon_center = Point::new(area.left + 7.0 * scale, area.center_y());
        Icon::Calendar.draw(canvas, icon_center, 14.0 * scale, Color::WHITE, opacity);
        let title_width = (canvas.measure(&BADGE, &reminder.title).min(REMINDER_TITLE_WIDTH)).ceil() * scale;
        let title_left = area.left + 20.0 * scale;
        canvas.text(
            &BADGE,
            &reminder.title,
            Rect::new(title_left, area.top, title_left + title_width + 2.0 * scale, area.bottom),
            Color::WHITE.faded(opacity),
        );
        let time_left = title_left + title_width + 8.0 * scale;
        canvas.text(
            &BADGE_TIME,
            &reminder.text,
            Rect::new(time_left, area.top, area.right, area.bottom),
            BLUE.faded(opacity),
        );
    }

    fn on_start(&mut self, context: &mut Context) {
        self.notifier = Some(context.notifier());
        self.link = context.settings.get_string(LINK_SETTING).map(str::to_owned).filter(|link| !link.is_empty());
        self.read_fetch_interval(context);
        // The stored copy belongs to the address; with none, a cleared calendar must not come back.
        let stored =
            self.link.as_ref().and(self.storage.as_ref()).and_then(|storage| std::fs::read_to_string(storage).ok());
        if let Some(text) = stored {
            self.load_events(&text);
        }
        self.refresh(context.fonts());
        Self::arm_minute_timer(context);
        if self.link.is_some() {
            context.set_timer(FETCH_TIMER, FIRST_FETCH_MILLISECONDS);
        }
    }

    fn on_settings_changed(&mut self, context: &mut Context) {
        let previous_interval = self.fetch_interval_milliseconds;
        self.read_fetch_interval(context);
        match link_change(self.link.as_deref(), context.settings.get_string(LINK_SETTING)) {
            LinkChange::Keep => {
                if self.link.is_some() && self.fetch_interval_milliseconds != previous_interval {
                    context.set_timer(FETCH_TIMER, self.fetch_interval_milliseconds);
                }
                return;
            }
            LinkChange::Remove => {
                self.forget_link();
                context.kill_timer(FETCH_TIMER);
            }
            LinkChange::Replace(link) => {
                self.link = Some(link);
                context.set_timer(FETCH_TIMER, FIRST_FETCH_MILLISECONDS);
            }
        }
        self.refresh(context.fonts());
        context.request_repaint();
    }

    fn on_shown_changed(&mut self, is_shown: bool, context: &mut Context) {
        if is_shown {
            self.refresh(context.fonts());
            context.request_repaint();
        } else {
            self.hovered_join = None;
            self.hovered_day = None;
            self.selected_day = None;
        }
    }

    fn on_notify(&mut self, context: &mut Context) {
        let result = self.fetched.lock().ok().and_then(|mut slot| slot.take());
        match result {
            Some((link, _)) if self.link.as_deref() != Some(link.as_str()) => {}
            Some((_, Ok(text))) => {
                if let Some(storage) = &self.storage {
                    if let Some(directory) = storage.parent() {
                        let _ = std::fs::create_dir_all(directory);
                    }
                    let _ = std::fs::write(storage, &text);
                }
                self.load_events(&text);
                self.refresh(context.fonts());
                context.request_repaint();
            }
            Some((_, Err(message))) => crate::log::debug(&format!("calendar: could not download: {message}")),
            None => {}
        }
    }

    fn on_pointer(&mut self, event: PointerEvent, frame: &Frame, context: &mut Context) {
        let (bounds, scale) = (frame.bounds, frame.scale);
        match event {
            PointerEvent::Moved(point) => {
                let hovered = (0..self.agenda.rows.len()).find(|&position| {
                    self.join_button(position, bounds, scale).is_some_and(|button| button.contains(point))
                });
                let hovered_day = day_at(point, bounds, scale);
                if hovered != self.hovered_join || hovered_day != self.hovered_day {
                    self.hovered_join = hovered;
                    self.hovered_day = hovered_day;
                    context.request_repaint();
                }
            }
            PointerEvent::Pressed(point) if frame.is_open() => {
                if let Some(position) = day_at(point, bounds, scale) {
                    let day = week_start(self.today) + position as i64;
                    self.selected_day = (day != self.today).then_some(day);
                    self.hovered_join = None;
                    self.refresh(context.fonts());
                    context.request_repaint();
                    return;
                }
                for (position, row) in self.agenda.rows.iter().enumerate() {
                    if let (Some(button), Some(meeting)) = (self.join_button(position, bounds, scale), &row.meeting)
                        && button.contains(point)
                    {
                        platform::open(&meeting.url);
                    }
                }
            }
            PointerEvent::Left | PointerEvent::CaptureLost => {
                let was_hovered = self.hovered_join.take().is_some() | self.hovered_day.take().is_some();
                if was_hovered {
                    context.request_repaint();
                }
            }
            _ => {}
        }
    }

    fn on_timer(&mut self, timer: TimerId, context: &mut Context) {
        match timer {
            MINUTE_TIMER => {
                self.refresh(context.fonts());
                Self::arm_minute_timer(context);
                context.request_repaint();
            }
            FETCH_TIMER => {
                self.start_fetch();
                context.set_timer(FETCH_TIMER, self.fetch_interval_milliseconds);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{agenda::State, dates::MONTH_NAMES, *};

    #[test]
    fn the_month_is_written_in_full() {
        assert_eq!(MONTH_NAMES[9], "October");
        assert_eq!(MONTH_NAMES[11], "December");
    }

    #[test]
    fn a_picked_day_lists_its_own_events_and_today_is_the_fallback() {
        let fonts = Fonts::new(1.0).unwrap();
        let mut page = CalendarPage::sample(&fonts);
        assert_eq!(page.selected(), page.today);
        assert!(!page.agenda.rows.is_empty());
        page.selected_day = Some(week_start(page.today) + (page.today - week_start(page.today) + 1) % 7);
        page.refresh(&fonts);
        assert!(page.agenda.rows.iter().all(|row| row.state == State::Later));
        page.selected_day = Some(week_start(page.today) - 3);
        page.refresh(&fonts);
        assert_eq!(page.selected(), page.today);
    }

    #[test]
    fn the_calendar_is_a_page_to_open_with_no_collapsed_state() {
        let page = CalendarPage::new();
        assert_eq!(page.priority(Mode::Resting), None);
        assert!(page.priority(Mode::Expanded).is_some());
        assert_eq!(page.presence(), None);
    }
}
