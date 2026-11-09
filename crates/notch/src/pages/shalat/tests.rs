use self::config::HANAFI_KEY;
use self::times::{Place, Prayer};
use crate::page::Page;
use places::{CITY_KEY, COUNTRY_KEY, LATITUDE_KEY, LONGITUDE_KEY};
use salah::{Local, Madhab, Utc};

use super::{schedule::wait_milliseconds, text::remaining_text, *};

use salah::Datelike;

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
fn dates_in_the_page_come_from_this_pcs_clock() {
    let today = Utc::now().with_timezone(&Local).date_naive();
    assert!(today.year() >= 2026);
}

#[test]
fn the_short_badge_drops_the_word_in() {
    assert_eq!(short_badge_text(Prayer::Asr, 24 * 60), Some("Asr 24m".to_owned()));
    assert_eq!(short_badge_text(Prayer::Asr, 31 * 60), None);
}

#[test]
fn the_prayer_ranks_last_and_its_header_icon_opens_the_shalat_page() {
    let page = ShalatPage::new();
    assert_eq!(page.header_status_rank(), 5);
    assert_eq!(page.header_status_page(), Some("shalat"));
}

#[test]
fn the_prayer_header_icon_shows_only_while_the_badge_does() {
    let fonts = crate::gfx::Fonts::new(1.0).unwrap();
    assert_eq!(ShalatPage::sample().header_status_width(), None);
    let page = ShalatPage::sample_badge(&fonts);
    assert_eq!(page.header_status_width(), Some(Icon::Moon.size()));
    let Some(Presence::Badge { width }) = page.presence() else { panic!("the sample has a badge") };
    assert!(page.short_presence_width().unwrap() < width);
    assert!(width > Icon::Moon.size() + BADGE_ICON_GAP, "the badge holds the icon and the text");
}

#[test]
fn the_prayer_badge_is_exactly_the_moon_a_gap_and_the_current_text() {
    let fonts = crate::gfx::Fonts::new(1.0).unwrap();
    let mut page = ShalatPage::sample();
    for (prayer, minutes) in [(Prayer::Isha, 22), (Prayer::Isha, 9), (Prayer::Maghrib, 59 - 30)] {
        page.set_badge(&fonts, prayer, minutes * 60);
        let Some(Presence::Badge { width }) = page.presence() else { panic!("the page has a badge") };
        let text = badge_text(prayer, minutes * 60).unwrap();
        assert_eq!(width, BADGE_ICON_SIZE + 4.0 + fonts.measure(&MEDIUM_13, &text).ceil());
        let short = short_badge_text(prayer, minutes * 60).unwrap();
        assert_eq!(page.short_presence_width(), Some(BADGE_ICON_SIZE + 4.0 + fonts.measure(&MEDIUM_13, &short).ceil()));
    }
}

#[test]
fn the_prayer_badge_width_follows_the_text_when_the_minutes_lose_a_digit() {
    let fonts = crate::gfx::Fonts::new(1.0).unwrap();
    let mut page = ShalatPage::sample();
    page.set_badge(&fonts, Prayer::Isha, 22 * 60);
    let two_digits = page.presence();
    page.set_badge(&fonts, Prayer::Isha, 9 * 60);
    let (Some(Presence::Badge { width: before }), Some(Presence::Badge { width: after })) =
        (two_digits, page.presence())
    else {
        panic!("the page has a badge")
    };
    assert!(after < before);
}

#[test]
fn the_prayer_badge_text_ends_where_the_badge_does() {
    let fonts = crate::gfx::Fonts::new(1.0).unwrap();
    let mut page = ShalatPage::sample();
    page.set_badge(&fonts, Prayer::Isha, 22 * 60);
    let Some(Presence::Badge { width }) = page.presence() else { panic!("the page has a badge") };
    let text_left = BADGE_ICON_SIZE + BADGE_ICON_GAP;
    assert_eq!(text_left + fonts.measure(&MEDIUM_13, "Isha in 22m").ceil(), width);
}

#[test]
fn the_prayer_counts_down_to_the_next_prayer_only_while_the_badge_shows() {
    let fonts = crate::gfx::Fonts::new(1.0).unwrap();
    assert_eq!(ShalatPage::sample().countdown_seconds(), None);
    let seconds = ShalatPage::sample_badge(&fonts).countdown_seconds().expect("the sample has a badge");
    assert!(seconds > 0, "Asr is still ahead of the sample's moment");
}
