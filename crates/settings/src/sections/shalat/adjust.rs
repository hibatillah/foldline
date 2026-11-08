//! Fine-tuning each prayer by up to 30 minutes, with a day bar that shows where each prayer falls.

use gpui::{AnyElement, App, SharedString, div, linear_color_stop, linear_gradient, prelude::*, px, relative};
use gpui_component::{ActiveTheme as _, Icon, IconName, h_flex, v_flex};
use store::Settings;

use crate::{card, field, layout::Item, picture::color};

const OFFSET_LIMIT: i64 = 30;
// The bar covers 03:30 to 20:30, where the prayers of a city near the equator fall.
const DAY_START_MINUTE: i64 = 3 * 60 + 30;
const DAY_MINUTES: i64 = 17 * 60;
const COLUMN_MIN_WIDTH: f32 = 104.0;
const DOT_SIZE: f32 = 10.0;
const DOT_RING: f32 = 3.0;
const BAR_HEIGHT: f32 = 12.0;

struct Prayer {
    name: &'static str,
    key: &'static str,
    // Where the prayer typically falls, in minutes after midnight; the bar moves the dot from here.
    typical_minute: i64,
}

// The keys are `Prayer::offset_key` in `crates/notch/src/pages/shalat/times.rs`; a test compares them.
const PRAYERS: [Prayer; 6] = [
    Prayer { name: "Fajr", key: "shalat.offset_fajr", typical_minute: 4 * 60 + 38 },
    Prayer { name: "Sunrise", key: "shalat.offset_sunrise", typical_minute: 5 * 60 + 56 },
    Prayer { name: "Dhuhr", key: "shalat.offset_dhuhr", typical_minute: 11 * 60 + 52 },
    Prayer { name: "Asr", key: "shalat.offset_asr", typical_minute: 15 * 60 + 14 },
    Prayer { name: "Maghrib", key: "shalat.offset_maghrib", typical_minute: 17 * 60 + 51 },
    Prayer { name: "Isha", key: "shalat.offset_isha", typical_minute: 19 * 60 + 3 },
];

// Color stops of the day bar, as a fraction of the day: night, dawn, noon blue, dusk, night.
const DAY_STOPS: [(f32, u32); 8] = [
    (0.0, 0x14213d),
    (0.08, 0x6a4a7a),
    (0.14, 0xf2b36b),
    (0.30, 0x8fc1e8),
    (0.62, 0x8fc1e8),
    (0.76, 0xf08a4b),
    (0.86, 0x3a2a5a),
    (1.0, 0x14213d),
];

/// The saved offset as the notch reads it: whole minutes, 0 when missing or unreadable, kept within the limit.
pub fn read_offset(settings: &Settings, key: &str) -> i64 {
    settings
        .get_string(key)
        .and_then(|text| text.trim().parse::<i64>().ok())
        .unwrap_or(0)
        .clamp(-OFFSET_LIMIT, OFFSET_LIMIT)
}

/// One step of the minus or plus button, stopping at the limit.
pub fn stepped(offset: i64, delta: i64) -> i64 {
    (offset + delta).clamp(-OFFSET_LIMIT, OFFSET_LIMIT)
}

/// "0 min", "+2 min", "−5 min".
pub fn offset_text(offset: i64) -> String {
    match offset {
        0 => "0 min".to_owned(),
        offset if offset > 0 => format!("+{offset} min"),
        offset => format!("−{} min", -offset),
    }
}

/// Where a dot sits on the bar, 0 to 1, after moving a typical time by its offset.
pub fn bar_position(typical_minute: i64, offset: i64) -> f32 {
    let from_start = typical_minute + offset - DAY_START_MINUTE;
    (from_start as f32 / DAY_MINUTES as f32).clamp(0.0, 1.0)
}

fn day_strip() -> AnyElement {
    let segments = DAY_STOPS.windows(2).map(|stops| {
        let ((from, from_color), (to, to_color)) = (stops[0], stops[1]);
        // Each segment runs one pixel under the next so rounding at their edges leaves no seam.
        let segment = div().absolute().top_0().bottom_0().left(relative(from)).right(relative(1.0 - to)).mr(px(-1.0));
        if from_color == to_color {
            segment.bg(color(from_color))
        } else {
            segment.bg(linear_gradient(
                90.0,
                linear_color_stop(color(from_color), 0.0),
                linear_color_stop(color(to_color), 1.0),
            ))
        }
    });
    div()
        .relative()
        .w_full()
        .h(px(BAR_HEIGHT))
        .rounded(px(BAR_HEIGHT / 2.0))
        .overflow_hidden()
        .children(segments)
        .into_any_element()
}

fn day_bar(offsets: &[i64; 6]) -> AnyElement {
    let ring = DOT_SIZE + DOT_RING * 2.0;
    let dots = PRAYERS.iter().zip(offsets).map(|(prayer, offset)| {
        div()
            .absolute()
            .left(relative(bar_position(prayer.typical_minute, *offset)))
            .top(px((BAR_HEIGHT - ring) / 2.0))
            .ml(px(-ring / 2.0))
            .size(px(ring))
            .rounded_full()
            .bg(color(0x000000).opacity(0.45))
            .flex()
            .items_center()
            .justify_center()
            .child(div().size(px(DOT_SIZE)).rounded_full().bg(color(0xffffff)))
    });
    div().relative().w_full().child(day_strip()).children(dots).into_any_element()
}

fn step_button(
    key: &'static str,
    label: &'static str,
    icon: IconName,
    delta: i64,
    offset: i64,
    cx: &App,
) -> AnyElement {
    let available = stepped(offset, delta) != offset;
    div()
        .id(SharedString::from(format!("{key}-{label}")))
        .size(px(24.0))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(6.0))
        .border_1()
        .border_color(cx.theme().border)
        .bg(crate::palette::input_background(cx))
        .when(available, |button| {
            button.cursor_pointer().on_click(move |_event, _window, cx| {
                let next = stepped(read_offset(&Settings::load(), key), delta);
                field::write(key, &next.to_string(), cx);
            })
        })
        .when(!available, |button| button.opacity(0.4))
        .child(Icon::new(icon).size(px(12.0)))
        .into_any_element()
}

fn prayer_column(prayer: &'static Prayer, offset: i64, cx: &App) -> AnyElement {
    v_flex()
        .flex_1()
        .min_w(px(COLUMN_MIN_WIDTH))
        .items_center()
        .gap(px(6.0))
        .child(div().text_size(px(13.0)).child(prayer.name))
        .child(
            h_flex()
                .items_center()
                .gap(px(4.0))
                .child(step_button(prayer.key, "minus", IconName::Minus, -1, offset, cx))
                .child(
                    div().w(px(48.0)).text_center().text_size(px(12.0)).child(SharedString::from(offset_text(offset))),
                )
                .child(step_button(prayer.key, "plus", IconName::Plus, 1, offset, cx)),
        )
        .into_any_element()
}

pub fn item() -> Item {
    Item::custom(|_window, cx| {
        let settings = Settings::load();
        let offsets: [i64; 6] = std::array::from_fn(|index| read_offset(&settings, PRAYERS[index].key));
        let columns = PRAYERS.iter().zip(offsets).map(|(prayer, offset)| prayer_column(prayer, offset, cx));
        v_flex()
            .gap_3()
            .child(card::described(
                "Fine-tune each prayer",
                "Add or subtract minutes, from −30 to 30, to match your local mosque. The dots show where each prayer falls in the day.",
                cx,
            ))
            .child(v_flex().gap(px(14.0)).px(px(16.0)).py(px(8.0)).child(day_bar(&offsets)).child(h_flex().flex_wrap().gap(px(4.0)).children(columns)))
            .into_any_element()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings_with(contents: &str) -> Settings {
        let path = std::env::temp_dir().join(format!("notch-settings-adjust-{}.txt", std::process::id()));
        std::fs::write(&path, contents).unwrap();
        let settings = Settings::at(Some(path.clone()));
        let _ = std::fs::remove_file(&path);
        settings
    }

    #[test]
    fn the_keys_are_the_ones_the_notch_reads_for_each_prayer() {
        let source =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../notch/src/pages/shalat/times.rs"))
                .expect("notch times source");
        for prayer in &PRAYERS {
            let line = format!("Prayer::{} => \"{}\"", prayer.name, prayer.key);
            assert!(source.contains(&line), "{line} missing in times.rs");
        }
    }

    #[test]
    fn the_prayers_come_in_the_order_of_the_day() {
        let minutes: Vec<i64> = PRAYERS.iter().map(|prayer| prayer.typical_minute).collect();
        assert!(minutes.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn a_step_stops_at_minus_and_plus_thirty() {
        assert_eq!(stepped(0, 1), 1);
        assert_eq!(stepped(0, -1), -1);
        assert_eq!(stepped(30, 1), 30);
        assert_eq!(stepped(-30, -1), -30);
    }

    #[test]
    fn a_stored_offset_is_read_like_the_notch_reads_it() {
        let settings = settings_with(
            "shalat.offset_fajr=-5\nshalat.offset_asr=99\nshalat.offset_isha=abc\nshalat.offset_dhuhr= 2 \n",
        );
        assert_eq!(read_offset(&settings, "shalat.offset_fajr"), -5);
        assert_eq!(read_offset(&settings, "shalat.offset_asr"), 30);
        assert_eq!(read_offset(&settings, "shalat.offset_isha"), 0);
        assert_eq!(read_offset(&settings, "shalat.offset_dhuhr"), 2);
        assert_eq!(read_offset(&settings, "shalat.offset_maghrib"), 0);
    }

    #[test]
    fn the_offset_reads_with_a_sign_and_the_unit() {
        assert_eq!(offset_text(0), "0 min");
        assert_eq!(offset_text(2), "+2 min");
        assert_eq!(offset_text(-30), "−30 min");
    }

    #[test]
    fn an_offset_moves_the_dot_along_the_bar_and_never_off_it() {
        let plain = bar_position(PRAYERS[2].typical_minute, 0);
        assert!(bar_position(PRAYERS[2].typical_minute, 30) > plain);
        assert!(bar_position(PRAYERS[2].typical_minute, -30) < plain);
        assert!((0.0..=1.0).contains(&bar_position(0, -30)));
        assert!((0.0..=1.0).contains(&bar_position(24 * 60, 30)));
    }

    #[test]
    fn the_day_bar_stops_run_from_the_start_to_the_end_of_the_bar() {
        assert_eq!(DAY_STOPS.first().map(|stop| stop.0), Some(0.0));
        assert_eq!(DAY_STOPS.last().map(|stop| stop.0), Some(1.0));
        assert!(DAY_STOPS.windows(2).all(|pair| pair[0].0 < pair[1].0));
    }
}
