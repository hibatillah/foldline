use super::{
    dates::{civil_from_days, month_name, weekday, weekday_name},
    *,
};
use crate::page::DEFAULT_CONTENT_HEIGHT;

#[test]
fn a_new_page_has_a_label_for_every_date_of_its_month_before_any_refresh() {
    let page = CalendarPage::new();
    assert_eq!(page.labels.day_numbers.len(), page.grid.day_count as usize);
    assert_eq!(page.labels.year, "1970");
}

#[test]
fn the_month_is_written_in_full() {
    assert_eq!(month_name(10), "October");
    assert_eq!(month_name(12), "December");
}

fn another_day_of_the_month(page: &CalendarPage) -> i64 {
    if page.grid.contains(page.today + 1) { page.today + 1 } else { page.today - 1 }
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
    page.selected_day = Some(page.grid.first_day - 3);
    page.refresh(&fonts);
    assert_eq!(page.selected(), page.today);
}

#[test]
fn a_day_picked_in_an_earlier_month_is_forgotten() {
    let fonts = Fonts::new(1.0).unwrap();
    let mut page = CalendarPage::sample(&fonts);
    page.selected_day = Some(page.grid.first_day + i64::from(page.grid.day_count));
    page.refresh(&fonts);
    assert_eq!(page.selected_day, None);
}

#[test]
fn the_page_is_the_viewport_high_and_taller_only_for_a_six_row_month() {
    let fonts = Fonts::new(1.0).unwrap();
    let page = CalendarPage::sample(&fonts);
    assert_eq!(page.sizes().expanded.height, content_height(page.grid.rows(), page.view));
    assert!(page.sizes().expanded.height >= geometry::MONTH_VIEWPORT_HEIGHT);
    assert_eq!(page.sizes().expanded.width, EXPANDED_WIDTH);
}

#[test]
fn the_month_labels_cover_every_date_of_the_month() {
    let fonts = Fonts::new(1.0).unwrap();
    let page = CalendarPage::sample(&fonts);
    assert_eq!(page.labels.day_numbers.len(), page.grid.day_count as usize);
}

#[test]
fn the_empty_day_message_names_the_selected_weekday_and_date_and_follows_the_pick() {
    let fonts = Fonts::new(1.0).unwrap();
    let mut page = CalendarPage::sample(&fonts);
    let expected = |day: i64| {
        let (_, _, date) = civil_from_days(day);
        format!("No events on {} {date}", weekday_name(weekday(day)))
    };
    assert_eq!(page.labels.empty_title, expected(page.today));
    let other_day = another_day_of_the_month(&page);
    page.selected_day = Some(other_day);
    page.refresh(&fonts);
    assert_eq!(page.labels.empty_title, expected(other_day));
}

#[test]
fn only_a_list_of_more_than_four_cards_claims_the_wheel_in_the_month_view() {
    let fonts = Fonts::new(1.0).unwrap();
    let mut page = CalendarPage::sample(&fonts);
    let bounds = Rect::new(0.0, 0.0, EXPANDED_WIDTH, content_height(page.grid.rows(), page.view));
    let frame = Frame { bounds, expansion: 1.0, expanded: Size::new(EXPANDED_WIDTH, bounds.height()), lead_gap: 0.0 };
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

fn week_page(fonts: &Fonts) -> CalendarPage {
    let mut page = CalendarPage::sample(fonts);
    page.view = View::Week;
    page.refresh(fonts);
    page
}

#[test]
fn the_week_view_draws_one_row_of_seven_days_that_holds_today() {
    let fonts = Fonts::new(1.0).unwrap();
    let page = week_page(&fonts);
    assert_eq!((page.grid.rows(), page.grid.day_count, page.labels.day_numbers.len()), (1, 7, 7));
    assert!(page.grid.contains(page.today));
}

#[test]
fn the_week_view_is_as_tall_as_the_default_page_and_as_wide_as_the_month_view() {
    let fonts = Fonts::new(1.0).unwrap();
    let page = week_page(&fonts);
    assert_eq!(page.sizes().expanded.height, DEFAULT_CONTENT_HEIGHT);
    assert_eq!(page.sizes().expanded.width, EXPANDED_WIDTH);
}

#[test]
fn a_day_picked_in_another_week_is_forgotten_when_the_view_turns_to_week() {
    let fonts = Fonts::new(1.0).unwrap();
    let mut page = CalendarPage::sample(&fonts);
    page.selected_day = Some(page.today - 14);
    page.view = View::Week;
    page.refresh(&fonts);
    assert_eq!(page.selected_day, None);
}

#[test]
fn a_week_list_of_more_than_two_cards_claims_the_wheel() {
    let fonts = Fonts::new(1.0).unwrap();
    let mut page = week_page(&fonts);
    let bounds = Rect::new(0.0, 0.0, EXPANDED_WIDTH, page.sizes().expanded.height);
    let frame = Frame { bounds, expansion: 1.0, expanded: Size::new(EXPANDED_WIDTH, bounds.height()), lead_gap: 0.0 };
    let inside = Point::new(bounds.right - 20.0, bounds.bottom - 20.0);
    assert!(!page.claims_scroll(inside, &frame), "two events fit the week list");
    page.agenda.rows = (0..3)
        .map(|index| agenda::Row { occurrence: page.agenda.rows[0].occurrence.clone(), time: format!("{index}") })
        .collect();
    assert!(page.claims_scroll(inside, &frame));
}

#[test]
fn a_refresh_in_indonesian_rebuilds_the_labels_the_page_caches() {
    let fonts = Fonts::new(1.0).unwrap();
    let mut page = CalendarPage::sample(&fonts);
    assert!(page.labels.empty_title.starts_with("No events on"));
    store::language::with_override(store::language::Language::Indonesian, || {
        page.refresh(&fonts);
        assert!(page.labels.empty_title.starts_with("Tidak ada acara hari"), "{}", page.labels.empty_title);
    });
}

#[test]
fn the_all_day_text_fits_the_time_column_in_both_languages() {
    let fonts = Fonts::new(1.0).unwrap();
    for language in [store::language::Language::English, store::language::Language::Indonesian] {
        store::language::with_override(language, || {
            let width = fonts.measure(&EVENT_TIME, &crate::i18n::tr!("calendar.all_day"));
            assert!(width <= draw::TIME_WIDTH, "{language:?}: All day is {width} px in {}", draw::TIME_WIDTH);
        });
    }
}
