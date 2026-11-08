//! Calendar links and cached ICS data live beside settings; collapsed reminders precede events by 15 minutes.

use self::{
    agenda::{Agenda, Reminder, build, list, reminder},
    dates::{MonthGrid, day_of},
    ics::{Event, Occurrence, occurrences_on, parse},
};
use crate::{
    activity::Presence,
    gfx::{Canvas, Color, Fonts, Point, Rect, Size},
    page::{Context, Frame, Mode, Notifier, Page, PageSizes, PointerEvent, Priority, TimerId},
    ui::Icon,
};
use geometry::{WHEEL_NOTCH_DISTANCE, content_height, day_at, events_area, max_scroll, widest_form};
use labels::Labels;
use local_time::{SystemClock, local_now};
use settings::{LinkChange, link_change, minutes_to_milliseconds};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Instant,
};
use style::*;

mod draw;
mod geometry;
mod local_time;
mod settings;
mod style;

mod agenda;

mod dates;

mod fetch;

mod labels;

mod wake;

mod ics;

const EXPANDED_WIDTH: f32 = 508.0;

const RESTING_WIDTH: f32 = 150.0;

const LEFT_WIDTH: f32 = 182.0;

const COLUMN_GAP: f32 = 16.0;
/// The collapsed badge: a 14 px icon, 4 px (design-rules), the title, 8 px, the time; no padding after it.
const BADGE_ICON_SIZE: f32 = 14.0;
const BADGE_ICON_GAP: f32 = 4.0;
const BADGE_TITLE_GAP: f32 = 8.0;

const DEFAULT_REFRESH_MINUTES: u64 = 15;

const FIRST_FETCH_MILLISECONDS: u32 = 2000;

const REMINDER_TITLE_WIDTH: f32 = 96.0;

const DONE_ICON_SIZE: f32 = 22.0;
const DONE_GAP: f32 = 10.0;
const EMPTY_ICON_SIZE: f32 = 22.0;
const EMPTY_GAP: f32 = 6.0;

const LINK_SETTING: &str = "calendar.ics_url";

const REFRESH_SETTING: &str = "calendar.refresh_minutes";

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
    month: MonthGrid,
    /// How far the event list is scrolled up, in device-independent pixels.
    scroll: f32,
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
            month: MonthGrid::of(0),
            scroll: 0.0,
            drag_last_y: None,
            selected_day: None,
            hovered_cell: None,
            fetch_interval_milliseconds: minutes_to_milliseconds(DEFAULT_REFRESH_MINUTES),
            is_shown: false,
            last_fetch: None,
            labels: Labels::new(0, 0),
        }
    }

    fn refresh(&mut self, fonts: &Fonts) {
        let now = local_now();
        self.today = day_of(now);
        self.month = MonthGrid::of(self.today);
        // A day picked in an earlier month is forgotten when the month turns.
        if self.selected_day.is_some_and(|day| !self.month.contains(day)) {
            self.selected_day = None;
        }
        self.occurrences = occurrences_on(&self.events, self.today, &SystemClock);
        self.agenda = match self.selected_day {
            Some(day) if day != self.today => list(&occurrences_on(&self.events, day, &SystemClock)),
            _ => build(&self.occurrences, now),
        };
        self.scroll = self.scroll.min(max_scroll(self.agenda.rows.len()));
        self.labels = Labels::new(self.today, self.selected());
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

    fn load_events(&mut self, text: &str) {
        self.events = parse(text);
    }

    fn selected(&self) -> i64 {
        self.selected_day.unwrap_or(self.today)
    }

    fn scroll_by(&mut self, distance: f32, max_scroll: f32, context: &mut Context) {
        let scroll = (self.scroll + distance).clamp(0.0, max_scroll);
        if scroll != self.scroll {
            self.scroll = scroll;
            context.request_repaint();
        }
    }
}

impl Page for CalendarPage {
    fn id(&self) -> &'static str {
        "calendar"
    }

    fn name(&self) -> &'static str {
        "Calendar"
    }

    fn priority(&self, mode: Mode) -> Option<Priority> {
        (mode == Mode::Expanded).then_some(Priority::BACKGROUND)
    }

    fn sizes(&self) -> PageSizes {
        PageSizes {
            resting_width: RESTING_WIDTH,
            expanded: Size::new(EXPANDED_WIDTH, content_height(self.month.rows())),
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
            self.draw_events_message(canvas, events_area(bounds, scale), opacity);
        } else {
            self.draw_rows(canvas, bounds, opacity);
        }
    }

    fn presence(&self) -> Option<Presence> {
        self.reminder.as_ref().map(|_| Presence::Badge { width: self.reminder_width })
    }

    fn draw_presence(&self, canvas: &Canvas, area: Rect, scale: f32, opacity: f32) {
        let Some(reminder) = &self.reminder else { return };
        let icon_center = Point::new(area.left + BADGE_ICON_SIZE * scale / 2.0, area.center_y());
        Icon::Calendar.draw(canvas, icon_center, BADGE_ICON_SIZE * scale, Color::WHITE, opacity);
        let title_width = (canvas.measure(&BADGE, &reminder.title).min(REMINDER_TITLE_WIDTH)).ceil() * scale;
        let title_left = area.left + (BADGE_ICON_SIZE + BADGE_ICON_GAP) * scale;
        canvas.text(
            &BADGE,
            &reminder.title,
            Rect::new(title_left, area.top, title_left + title_width + 2.0 * scale, area.bottom),
            Color::WHITE.faded(opacity),
        );
        let time_left = title_left + title_width + BADGE_TITLE_GAP * scale;
        canvas.text(
            &BADGE_TIME,
            &reminder.text,
            Rect::new(time_left, area.top, area.right, area.bottom),
            BLUE.faded(opacity),
        );
    }

    fn short_presence_width(&self) -> Option<f32> {
        self.reminder.as_ref().map(|_| self.reminder_short_width)
    }

    fn draw_short_presence(&self, canvas: &Canvas, area: Rect, scale: f32, opacity: f32) {
        let Some(reminder) = &self.reminder else { return };
        let icon_center = Point::new(area.left + BADGE_ICON_SIZE * scale / 2.0, area.center_y());
        Icon::Calendar.draw(canvas, icon_center, BADGE_ICON_SIZE * scale, Color::WHITE, opacity);
        let time_area =
            Rect::new(area.left + (BADGE_ICON_SIZE + BADGE_ICON_GAP) * scale, area.top, area.right, area.bottom);
        canvas.text(&BADGE_TIME, reminder.short_text(), time_area, BLUE.faded(opacity));
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
        self.arm_wake_timer(context);
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
        self.is_shown = is_shown;
        if is_shown {
            self.refresh(context.fonts());
            self.fetch_when_due(context);
            context.request_repaint();
        } else {
            self.hovered_cell = None;
            self.drag_last_y = None;
            self.scroll = 0.0;
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
        let (bounds, scale) = (frame.bounds, frame.scale);
        let scroll_limit = max_scroll(self.agenda.rows.len());
        match event {
            PointerEvent::Moved(point) => {
                if let Some(last_y) = self.drag_last_y {
                    self.drag_last_y = Some(point.y);
                    self.scroll_by((last_y - point.y) / scale, scroll_limit, context);
                }
                let hovered_cell = day_at(point, &self.month, bounds, scale);
                if hovered_cell != self.hovered_cell {
                    self.hovered_cell = hovered_cell;
                    context.request_repaint();
                }
            }
            PointerEvent::Pressed(point) if frame.is_open() => {
                if let Some(cell) = day_at(point, &self.month, bounds, scale) {
                    let day = self.month.first_day + (cell - self.month.leading) as i64;
                    self.selected_day = (day != self.today).then_some(day);
                    self.scroll = 0.0;
                    self.refresh(context.fonts());
                    context.request_repaint();
                } else if scroll_limit > 0.0 && events_area(bounds, scale).contains(point) {
                    self.drag_last_y = Some(point.y);
                }
            }
            PointerEvent::Scrolled { notches } => {
                self.scroll_by(notches * WHEEL_NOTCH_DISTANCE, scroll_limit, context);
            }
            PointerEvent::Released(_) => self.drag_last_y = None,
            PointerEvent::Left | PointerEvent::CaptureLost => {
                self.drag_last_y = None;
                if self.hovered_cell.take().is_some() {
                    context.request_repaint();
                }
            }
            _ => {}
        }
    }

    fn claims_scroll(&self, point: Point, frame: &Frame) -> bool {
        max_scroll(self.agenda.rows.len()) > 0.0 && events_area(frame.bounds, frame.scale).contains(point)
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

#[cfg(test)]
mod tests {
    use super::{
        dates::{MONTH_NAMES, WEEKDAY_NAMES, civil_from_days, weekday},
        *,
    };

    #[test]
    fn a_new_page_has_a_label_for_every_date_of_its_month_before_any_refresh() {
        let page = CalendarPage::new();
        assert_eq!(page.labels.day_numbers.len(), page.month.day_count as usize);
        assert_eq!(page.labels.year, "1970");
    }

    #[test]
    fn the_month_is_written_in_full() {
        assert_eq!(MONTH_NAMES[9], "October");
        assert_eq!(MONTH_NAMES[11], "December");
    }

    fn another_day_of_the_month(page: &CalendarPage) -> i64 {
        if page.month.contains(page.today + 1) { page.today + 1 } else { page.today - 1 }
    }

    #[test]
    fn a_picked_day_lists_its_own_events_and_today_is_the_fallback() {
        let fonts = Fonts::new(1.0).unwrap();
        let mut page = CalendarPage::sample(&fonts);
        assert_eq!(page.selected(), page.today);
        assert!(!page.agenda.rows.is_empty());
        page.selected_day = Some(another_day_of_the_month(&page));
        page.refresh(&fonts);
        assert!(!page.agenda.is_done);
        assert!(page.agenda.rows.is_empty(), "the sample has events only today");
        page.selected_day = Some(page.month.first_day - 3);
        page.refresh(&fonts);
        assert_eq!(page.selected(), page.today);
    }

    #[test]
    fn a_day_picked_in_an_earlier_month_is_forgotten() {
        let fonts = Fonts::new(1.0).unwrap();
        let mut page = CalendarPage::sample(&fonts);
        page.selected_day = Some(page.month.first_day + i64::from(page.month.day_count));
        page.refresh(&fonts);
        assert_eq!(page.selected_day, None);
    }

    #[test]
    fn the_page_is_the_viewport_high_and_taller_only_for_a_six_row_month() {
        let fonts = Fonts::new(1.0).unwrap();
        let page = CalendarPage::sample(&fonts);
        assert_eq!(page.sizes().expanded.height, content_height(page.month.rows()));
        assert!(page.sizes().expanded.height >= geometry::VIEWPORT_HEIGHT);
        assert_eq!(page.sizes().expanded.width, EXPANDED_WIDTH);
    }

    #[test]
    fn the_month_labels_cover_every_date_of_the_month() {
        let fonts = Fonts::new(1.0).unwrap();
        let page = CalendarPage::sample(&fonts);
        assert_eq!(page.labels.day_numbers.len(), page.month.day_count as usize);
    }

    #[test]
    fn the_empty_day_message_names_the_selected_weekday_and_date_and_follows_the_pick() {
        let fonts = Fonts::new(1.0).unwrap();
        let mut page = CalendarPage::sample(&fonts);
        let expected = |day: i64| {
            let (_, _, date) = civil_from_days(day);
            format!("No events on {} {date}", WEEKDAY_NAMES[weekday(day) as usize])
        };
        assert_eq!(page.labels.empty_title, expected(page.today));
        let other_day = another_day_of_the_month(&page);
        page.selected_day = Some(other_day);
        page.refresh(&fonts);
        assert_eq!(page.labels.empty_title, expected(other_day));
    }

    #[test]
    fn only_a_list_of_more_than_five_cards_claims_the_wheel() {
        let fonts = Fonts::new(1.0).unwrap();
        let mut page = CalendarPage::sample(&fonts);
        let bounds = Rect::new(0.0, 0.0, EXPANDED_WIDTH, content_height(page.month.rows()));
        let frame = Frame {
            scale: 1.0,
            bounds,
            expansion: 1.0,
            expanded: Size::new(EXPANDED_WIDTH, bounds.height()),
            lead_gap: 0.0,
        };
        let inside = Point::new(bounds.right - 20.0, bounds.bottom - 20.0);
        assert!(!page.claims_scroll(inside, &frame), "three events fit the viewport");
        page.agenda.rows = (0..9)
            .map(|index| agenda::Row { occurrence: page.agenda.rows[0].occurrence.clone(), time: format!("{index}") })
            .collect();
        assert!(page.claims_scroll(inside, &frame));
        assert!(!page.claims_scroll(Point::new(20.0, bounds.bottom - 20.0), &frame), "the month does not scroll");
    }

    #[test]
    fn the_calendar_is_a_page_to_open_with_no_collapsed_state() {
        let page = CalendarPage::new();
        assert_eq!(page.priority(Mode::Resting), None);
        assert!(page.priority(Mode::Expanded).is_some());
        assert_eq!(page.presence(), None);
    }

    #[test]
    fn the_event_ranks_fifth_and_its_header_icon_opens_the_calendar() {
        let page = CalendarPage::new();
        assert_eq!(page.header_status_rank(), 4);
        assert_eq!(page.header_status_page(), Some("calendar"));
    }

    #[test]
    fn the_event_counts_down_only_while_a_reminder_is_up() {
        let fonts = Fonts::new(1.0).unwrap();
        assert_eq!(CalendarPage::new().countdown_seconds(), None);
        assert!(CalendarPage::sample(&fonts).countdown_seconds().is_some());
    }

    #[test]
    fn the_event_shows_in_the_header_only_while_a_reminder_is_up_and_has_a_shorter_badge() {
        let fonts = Fonts::new(1.0).unwrap();
        assert_eq!(CalendarPage::new().header_status_width(), None);
        assert_eq!(CalendarPage::new().short_presence_width(), None);
        let page = CalendarPage::sample(&fonts);
        assert_eq!(page.header_status_width(), Some(Icon::Calendar.size()));
        let Some(Presence::Badge { width }) = page.presence() else { panic!("the sample has a reminder") };
        assert!(page.short_presence_width().unwrap() < width);
    }

    #[test]
    fn the_event_badge_is_exactly_its_content_with_no_padding_after_the_time() {
        let fonts = Fonts::new(1.0).unwrap();
        let page = CalendarPage::sample(&fonts);
        let reminder = page.reminder.as_ref().expect("the sample has a reminder");
        let title = fonts.measure(&BADGE, &reminder.title).min(REMINDER_TITLE_WIDTH).ceil();
        let time = fonts.measure(&BADGE_TIME, &reminder.text).ceil();
        let Some(Presence::Badge { width }) = page.presence() else { panic!("the sample has a reminder") };
        assert_eq!(width, 14.0 + 4.0 + title + 8.0 + time);
        let short = fonts.measure(&BADGE_TIME, reminder.short_text()).ceil();
        assert_eq!(page.short_presence_width(), Some(14.0 + 4.0 + short));
    }
}
