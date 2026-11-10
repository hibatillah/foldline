use self::{
    agenda::{Agenda, Reminder, build, list, reminder},
    dates::{DayGrid, day_of},
    ics::{Event, Occurrence, occurrences_on, parse},
};
use crate::{
    activity::Presence,
    gfx::{Canvas, Color, Fonts, Point, Rect, Size},
    page::{Context, Frame, Mode, Notifier, Page, PageSizes, PointerEvent, Priority, TimerId},
    ui::{Icon, ScrollOffset},
};
use geometry::{content_height, events_area, max_scroll, widest_form};
use labels::Labels;
use local_time::{SystemClock, local_now};
use settings::{LinkChange, link_change, minutes_to_milliseconds};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Instant,
};
use style::*;
use view::View;

mod agenda;
mod badge;
mod dates;
mod draw;
mod fetch;
mod geometry;
mod ics;
mod input;
mod labels;
mod local_time;
mod settings;
mod style;
#[cfg(test)]
mod tests;
mod view;
mod wake;

const EXPANDED_WIDTH: f32 = 508.0;
const RESTING_WIDTH: f32 = 150.0;
const LEFT_WIDTH: f32 = 182.0;
const COLUMN_GAP: f32 = 16.0;

/// The collapsed badge: a 14 px icon, 4 px (design-rules), the title, 8 px, the time; no padding after it.
const BADGE_ICON_SIZE: f32 = 14.0;
const BADGE_ICON_GAP: f32 = 4.0;
const BADGE_TITLE_GAP: f32 = 8.0;
const REMINDER_TITLE_WIDTH: f32 = 96.0;

const DEFAULT_REFRESH_MINUTES: u64 = 15;
const FIRST_FETCH_MILLISECONDS: u32 = 2000;

const DONE_ICON_SIZE: f32 = 22.0;
const DONE_GAP: f32 = 10.0;
const EMPTY_ICON_SIZE: f32 = 22.0;
const EMPTY_GAP: f32 = 6.0;

const LINK_SETTING: &str = "calendar.ics_url";
const REFRESH_SETTING: &str = "calendar.refresh_minutes";
const VIEW_SETTING: &str = "calendar.view";

/// Wakes once a minute, like the clock, while shown or counting down; see `wake.rs` for the hidden schedule.
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
    reminder_short_width: f32,
    grid: DayGrid,
    view: View,
    /// How far the event list is scrolled up, in device-independent pixels.
    scroll: ScrollOffset,
    /// The pointer height while a drag scrolls the list.
    drag_last_y: Option<f32>,
    selected_day: Option<i64>,
    hovered_cell: Option<usize>,
    fetch_interval_milliseconds: u32,
    is_shown: bool,
    last_fetch: Option<Instant>,
    labels: Labels,
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
            reminder_short_width: 0.0,
            grid: DayGrid::month_of(0),
            view: View::default(),
            scroll: ScrollOffset::default(),
            drag_last_y: None,
            selected_day: None,
            hovered_cell: None,
            fetch_interval_milliseconds: minutes_to_milliseconds(DEFAULT_REFRESH_MINUTES),
            is_shown: false,
            last_fetch: None,
            labels: Labels::new(0, 0, &DayGrid::month_of(0)),
        }
    }

    fn refresh(&mut self, fonts: &Fonts) {
        let now = local_now();
        self.today = day_of(now);
        self.grid = self.view.grid_of(self.today);
        // A day picked outside the grid is forgotten when the month or the week turns.
        if self.selected_day.is_some_and(|day| !self.grid.contains(day)) {
            self.selected_day = None;
        }
        self.occurrences = occurrences_on(&self.events, self.today, &SystemClock);
        self.agenda = match self.selected_day {
            Some(day) if day != self.today => list(&occurrences_on(&self.events, day, &SystemClock)),
            _ => build(&self.occurrences, now),
        };
        self.scroll.clamp_to(max_scroll(self.agenda.rows.len(), self.view));
        self.labels = Labels::new(self.today, self.selected(), &self.grid);
        self.reminder = reminder(&self.occurrences, now);
        if let Some(reminder) = &self.reminder {
            let title = fonts.measure(&BADGE, &reminder.title).min(REMINDER_TITLE_WIDTH).ceil();
            let time = fonts.measure(&BADGE_TIME, &widest_form(&reminder.text)).ceil();
            self.reminder_width = BADGE_ICON_SIZE + BADGE_ICON_GAP + title + BADGE_TITLE_GAP + time;
            let short_time = fonts.measure(&BADGE_TIME, &widest_form(reminder.short_text())).ceil();
            self.reminder_short_width = BADGE_ICON_SIZE + BADGE_ICON_GAP + short_time;
        }
    }

    #[cfg(test)]
    pub fn sample(fonts: &Fonts) -> Self {
        use dates::civil_from_days;
        let now = local_now();
        let (year, month, _) = civil_from_days(day_of(now));
        let at = |seconds: i64| {
            let (_, _, date) = civil_from_days(day_of(seconds));
            let of_day = seconds.rem_euclid(dates::SECONDS_PER_DAY);
            format!("{year:04}{month:02}{date:02}T{:02}{:02}00", of_day / 3600, of_day / 60 % 60)
        };
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

    #[cfg(test)]
    pub fn sample_week(fonts: &Fonts) -> Self {
        let mut page = Self::sample(fonts);
        page.view = View::Week;
        page.refresh(fonts);
        page
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

    fn name(&self) -> String {
        crate::i18n::tr!("page.calendar").into_owned()
    }

    fn priority(&self, mode: Mode) -> Option<Priority> {
        (mode == Mode::Expanded).then_some(Priority::BACKGROUND)
    }

    fn sizes(&self) -> PageSizes {
        PageSizes {
            resting_width: RESTING_WIDTH,
            expanded: Size::new(EXPANDED_WIDTH, content_height(self.grid.rows(), self.view)),
        }
    }

    /// Lowest rank sits nearest the clock: wave 0, Caps Lock 1, muted 2, timer 3, event 4, prayer 5.
    fn header_status_rank(&self) -> u8 {
        4
    }

    fn header_status_width(&self) -> Option<f32> {
        self.reminder.as_ref().map(|_| Icon::Calendar.size())
    }

    fn draw_header_status(&self, canvas: &Canvas, area: Rect, scale: f32, opacity: f32) {
        if self.reminder.is_none() {
            return;
        }
        let center = Point::new(area.center_x(), area.center_y());
        Icon::Calendar.draw(canvas, center, Icon::Calendar.size() * scale, Color::WHITE, opacity);
    }

    fn header_status_page(&self) -> Option<&'static str> {
        Some("calendar")
    }

    fn countdown_seconds(&self) -> Option<u64> {
        self.reminder.as_ref().map(|reminder| (reminder.start - local_now()).max(0) as u64)
    }

    fn draw_resting(&self, _canvas: &Canvas, _frame: &Frame, _opacity: f32) {}

    fn draw_expanded(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        let (bounds, scale) = (frame.bounds, frame.scale);
        self.draw_month(canvas, bounds, opacity);
        if self.agenda.rows.is_empty() {
            self.draw_events_message(canvas, events_area(bounds, scale, self.view), opacity);
        } else {
            self.draw_rows(canvas, bounds, opacity);
        }
    }

    fn presence(&self) -> Option<Presence> {
        self.reminder.as_ref().map(|_| Presence::Badge { width: self.reminder_width })
    }

    fn draw_presence(&self, canvas: &Canvas, area: Rect, scale: f32, opacity: f32) {
        self.draw_badge(canvas, area, scale, opacity);
    }

    fn short_presence_width(&self) -> Option<f32> {
        self.reminder.as_ref().map(|_| self.reminder_short_width)
    }

    fn draw_short_presence(&self, canvas: &Canvas, area: Rect, scale: f32, opacity: f32) {
        self.draw_short_badge(canvas, area, scale, opacity);
    }

    fn on_start(&mut self, context: &mut Context) {
        self.notifier = Some(context.notifier());
        self.link = context.settings.get_string(LINK_SETTING).map(str::to_owned).filter(|link| !link.is_empty());
        self.read_fetch_interval(context);
        self.view = View::from_setting(context.settings.get_string(VIEW_SETTING), self.view);
        // The stored copy belongs to the address; with none, a cleared calendar must not come back.
        let stored =
            self.link.as_ref().and(self.storage.as_ref()).and_then(|storage| std::fs::read_to_string(storage).ok());
        if let Some(text) = stored {
            self.load_events(&text);
        }
        self.refresh(context.fonts());
        self.arm_wake_timer(context);
        if self.link.is_some() {
            context.set_timer(FETCH_TIMER, FIRST_FETCH_MILLISECONDS);
        }
    }

    fn on_settings_changed(&mut self, context: &mut Context) {
        let previous_interval = self.fetch_interval_milliseconds;
        self.read_fetch_interval(context);
        let previous_view = self.view;
        self.view = View::from_setting(context.settings.get_string(VIEW_SETTING), self.view);
        let view_changed = self.view != previous_view;
        if view_changed {
            self.scroll.reset();
        }
        match link_change(self.link.as_deref(), context.settings.get_string(LINK_SETTING)) {
            LinkChange::Keep => {
                if self.link.is_some() && self.fetch_interval_milliseconds != previous_interval {
                    context.set_timer(FETCH_TIMER, self.fetch_interval_milliseconds);
                }
                if !view_changed {
                    return;
                }
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

    fn on_language_changed(&mut self, context: &mut Context) {
        self.refresh(context.fonts());
        context.request_repaint();
    }

    fn on_shown_changed(&mut self, is_shown: bool, context: &mut Context) {
        self.is_shown = is_shown;
        if is_shown {
            self.refresh(context.fonts());
            self.fetch_when_due(context);
            context.request_repaint();
        } else {
            self.hovered_cell = None;
            self.drag_last_y = None;
            self.scroll.reset();
            self.selected_day = None;
            self.refresh(context.fonts());
        }
        self.arm_wake_timer(context);
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
        self.handle_pointer(event, frame, context);
    }

    fn claims_scroll(&self, point: Point, frame: &Frame) -> bool {
        max_scroll(self.agenda.rows.len(), self.view) > 0.0
            && events_area(frame.bounds, frame.scale, self.view).contains(point)
    }

    fn on_timer(&mut self, timer: TimerId, context: &mut Context) {
        match timer {
            MINUTE_TIMER => {
                self.refresh(context.fonts());
                self.arm_wake_timer(context);
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
