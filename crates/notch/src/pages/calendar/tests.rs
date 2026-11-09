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
