use super::{
    layout::{self, CELL_WIDTH, CONTENT_HEIGHT, MESSAGE_WIDTH},
    model::{Listing, shown},
    sample::{self, BLUETOOTH_OFF, EMPTY, MANY},
    *,
};
use crate::gfx::{Align, FontStyle, Fonts, Rect, UI_FONT, Weight};

fn contains(outer: Rect, inner: Rect) -> bool {
    inner.left >= outer.left - 0.01
        && inner.right <= outer.right + 0.01
        && inner.top >= outer.top - 0.01
        && inner.bottom <= outer.bottom + 0.01
}

#[test]
fn the_page_is_named_for_the_settings_window_and_starts_switched_off() {
    let page = DevicesPage::new();
    assert_eq!(page.id(), "devices");
    assert_eq!(page.name(), "Connected devices");
    assert!(page.is_opt_in());
}

#[test]
fn the_page_has_no_collapsed_state_and_asks_for_the_message_size_until_it_has_tiles() {
    let page = DevicesPage::new();
    assert_eq!(page.priority(Mode::Resting), None);
    assert_eq!(page.priority(Mode::Expanded), Some(Priority::BACKGROUND));
    let sizes = page.sizes();
    assert_eq!((sizes.expanded.width, sizes.expanded.height), (MESSAGE_WIDTH, 86.0));
    assert!(page.presence().is_none());
}

#[test]
fn the_page_reads_only_while_it_shows_and_once_a_minute() {
    assert_eq!(REFRESH_INTERVAL_MILLISECONDS, 60_000);
}

#[test]
fn every_state_of_the_design_is_a_sample() {
    let count = |scenario: u8| match sample::listing(scenario) {
        Listing::Devices(devices) => devices.len(),
        Listing::BluetoothOff => usize::MAX,
    };
    assert_eq!(count(sample::NORMAL), 5);
    assert_eq!(count(sample::ATTENTION), 4);
    assert_eq!(count(sample::UNKNOWN), 2);
    assert_eq!(count(MANY), 7);
    assert_eq!(count(EMPTY), 0);
    assert_eq!(count(BLUETOOTH_OFF), usize::MAX);
}

#[test]
fn the_crowded_sample_shows_five_tiles_and_nothing_for_the_other_two() {
    let Listing::Devices(devices) = sample::listing(MANY) else { panic!("devices") };
    assert_eq!(shown(&devices).devices.len(), 5);
    assert_eq!(DevicesPage::sample(MANY).sizes().expanded.width, 5.0 * CELL_WIDTH);
}

#[test]
fn the_page_is_as_wide_as_its_tiles() {
    assert_eq!(DevicesPage::sample(sample::NORMAL).sizes().expanded.width, 5.0 * CELL_WIDTH);
    assert_eq!(DevicesPage::sample(sample::ATTENTION).sizes().expanded.width, 4.0 * CELL_WIDTH);
    assert_eq!(DevicesPage::sample(sample::UNKNOWN).sizes().expanded.width, layout::tiles_width(2));
    assert_eq!(DevicesPage::sample(EMPTY).sizes().expanded.width, MESSAGE_WIDTH);
}

#[test]
fn every_part_of_every_tile_stays_inside_the_page_area() {
    for count in 1..=5 {
        let area = Rect::new(0.0, 0.0, layout::tiles_width(count), CONTENT_HEIGHT);
        for index in 0..count {
            let cell = layout::cell(area, index, count);
            assert!(contains(area, cell));
            for line in [layout::percent_line(cell), layout::name_line(cell)] {
                assert!(contains(area, line), "{line:?}");
            }
            let ring = layout::ring_center(cell);
            let radius = layout::RING_DIAMETER / 2.0;
            assert!(ring.y - radius >= area.top && ring.y + radius <= area.bottom);
            assert!(ring.x - radius >= cell.left - 0.01, "a ring fits its cell");
        }
    }
    let area = Rect::new(0.0, 0.0, MESSAGE_WIDTH, CONTENT_HEIGHT);
    let text_width = 226.0;
    let (badge, title, detail) = layout::message_lines(area, text_width);
    assert!(contains(area, title) && contains(area, detail));
    assert!(badge.x - layout::MESSAGE_BADGE_DIAMETER / 2.0 >= area.left);
}

#[test]
fn the_longest_message_lines_fit_the_message_page_in_both_languages() {
    let fonts = Fonts::new(1.0).unwrap();
    let style = FontStyle::new(UI_FONT, 12.0, Weight::Regular, Align::Leading);
    let title = FontStyle::new(UI_FONT, 15.0, Weight::Medium, Align::Leading);
    for language in [store::language::Language::English, store::language::Language::Indonesian] {
        store::language::with_override(language, || {
            for key in ["devices.none_detail", "devices.bluetooth_off_detail"] {
                let width = fonts.measure(&style, &crate::i18n::tr!(key));
                assert!(width <= MESSAGE_WIDTH, "{language:?}: {key} is {width} px");
            }
            for key in ["devices.none_title", "devices.bluetooth_off_title"] {
                let width = fonts.measure(&title, &crate::i18n::tr!(key));
                assert!(width <= MESSAGE_WIDTH, "{language:?}: {key} is {width} px");
            }
        });
    }
}

#[test]
fn a_new_listing_repaints_and_the_same_one_does_not() {
    let page = DevicesPage::sample(sample::NORMAL);
    assert_eq!(page.listing, Some(sample::listing(sample::NORMAL)));
    assert_ne!(sample::listing(sample::NORMAL), sample::listing(sample::ATTENTION));
}

#[test]
fn opening_again_within_a_minute_of_a_read_reuses_its_answer_and_a_later_open_reads() {
    let read_at = Instant::now();
    let after = |seconds: u64| read_at + Duration::from_secs(seconds);
    assert!(should_read_on_open(None, false, read_at), "the first open reads");
    assert!(!should_read_on_open(Some(read_at), true, after(30)));
    assert!(should_read_on_open(Some(read_at), true, after(60)));
    assert!(should_read_on_open(Some(read_at), true, after(300)));
}

#[test]
fn a_page_showing_a_message_reads_again_on_every_open() {
    let read_at = Instant::now();
    assert!(should_read_on_open(Some(read_at), false, read_at + Duration::from_secs(5)));
}

#[test]
fn the_stored_words_match_the_devices_and_a_message_state_stores_none() {
    for scenario in [sample::NORMAL, sample::ATTENTION, sample::UNKNOWN, MANY] {
        let page = DevicesPage::sample(scenario);
        let prepared = page.prepared.as_ref().expect("tiles");
        let expected: Vec<String> = prepared.shown.devices.iter().map(|device| device.percent_text()).collect();
        assert_eq!(prepared.percent_texts, expected);
    }
    let attention = DevicesPage::sample(sample::ATTENTION);
    assert_eq!(attention.prepared.as_ref().unwrap().percent_texts, ["12%", "64%", "Off", "91%"]);
    assert!(DevicesPage::sample(EMPTY).prepared.is_none());
    assert!(DevicesPage::sample(BLUETOOTH_OFF).prepared.is_none());
}
