use super::*;

#[test]
fn the_timer_is_a_page_to_open_but_never_the_collapsed_page() {
    let page = TimerPage::new();
    assert_eq!(page.priority(Mode::Resting), None);
    assert_eq!(page.priority(Mode::Expanded), Some(Priority::BACKGROUND));
}

#[test]
fn the_timer_ranks_fourth_and_its_header_mark_opens_the_timer() {
    let page = TimerPage::new();
    assert_eq!(page.header_status_rank(), 3);
    assert_eq!(page.header_status_page(), Some("timer"));
}

#[test]
fn the_time_column_is_as_wide_as_the_widest_time() {
    let fonts = crate::gfx::Fonts::new(1.0).unwrap();
    let widest = fonts.measure(&TIME, "00:00");
    assert!(widest <= layout::TIME_WIDTH && layout::TIME_WIDTH - widest < 2.0, "{widest}");
    assert_eq!(widest, fonts.measure(&TIME, "88:88"), "tabular digits");
}

#[test]
fn an_idle_timer_has_no_countdown() {
    assert_eq!(TimerPage::new().countdown_seconds(), None);
}

#[test]
fn the_timer_badge_has_no_shorter_form() {
    assert_eq!(TimerPage::new().short_presence_width(), None);
}

#[test]
fn the_timer_badge_is_exactly_the_ring_a_gap_and_the_text() {
    let fonts = crate::gfx::Fonts::new(1.0).unwrap();
    let mut page = TimerPage::new();
    page.timer.start_focus(now_ms());
    page.refresh_compact(&fonts);
    let text = page.compact.as_ref().expect("a running timer has a badge").text.clone();
    let Some(Presence::Badge { width }) = page.presence() else { panic!("a running timer shows a badge") };
    assert_eq!(width, RING_DIAMETER + 4.0 + fonts.measure(&BADGE, &text).ceil());
    assert_eq!(fonts.measure(&BADGE, &text), fonts.measure(&BADGE, &view::widest_form(&text)), "tabular digits");
}

#[test]
fn an_idle_timer_shows_nothing_in_the_collapsed_pill() {
    assert_eq!(TimerPage::new().presence(), None);
}

#[test]
fn the_main_button_is_the_timer_orange_for_focus_and_blue_for_a_break() {
    let mut page = TimerPage::new();
    assert_eq!(page.accent(), FOCUS_ORANGE);
    page.timer.start_focus(now_ms());
    assert_eq!(page.accent(), FOCUS_ORANGE);
    page.timer.tick(now_ms() + 26 * 60_000);
    assert_eq!(page.accent(), BREAK_BLUE, "a finished focus session offers the break");
}

#[test]
fn the_timer_orange_is_the_pill_orange() {
    assert_eq!(FOCUS_ORANGE, Color::rgb(1.0, 0.62, 0.04));
    assert_eq!(tone_color(Tone::Focus), FOCUS_ORANGE);
}
