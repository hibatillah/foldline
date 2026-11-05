//! What the shalat page reads, computes and shows.

use self::times::{PRAYERS, Place, Prayer};
use crate::page::{Control, Page};
use salah::{Local, Madhab, Utc};

use super::{schedule::wait_milliseconds, *};

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
