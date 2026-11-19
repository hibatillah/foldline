//! Fine-tuning each prayer by up to 30 minutes: one card with a day bar that shows where each prayer falls, then one row
//! per prayer with its name at the left and a minus and plus stepper at the right.

use crate::i18n::tr;
use std::rc::Rc;

use gpui::{AnyElement, App, SharedString, div, linear_color_stop, linear_gradient, prelude::*, px, relative};
use store::Settings;

use crate::{
    field,
    layout::{Item, Render},
    picture::color,
    stepper::{self, Step},
};

const OFFSET_LIMIT: i64 = 30;
// The bar covers 03:30 to 20:30, where the prayers of a city near the equator fall.
const DAY_START_MINUTE: i64 = 3 * 60 + 30;
const DAY_MINUTES: i64 = 17 * 60;
const DOT_SIZE: f32 = 10.0;
const DOT_RING: f32 = 3.0;
const BAR_HEIGHT: f32 = 12.0;

struct Prayer {
    name: fn() -> &'static str,
    key: &'static str,
    // Where the prayer typically falls, in minutes after midnight; the bar moves the dot from here.
    typical_minute: i64,
}

// The keys are `Prayer::offset_key` in `crates/notch/src/pages/shalat/times.rs`; a test compares them.
const PRAYERS: [Prayer; 6] = [
    Prayer { name: || tr!("shalat.fajr"), key: "shalat.offset_fajr", typical_minute: 4 * 60 + 38 },
    Prayer { name: || tr!("shalat.sunrise"), key: "shalat.offset_sunrise", typical_minute: 5 * 60 + 56 },
    Prayer { name: || tr!("shalat.dhuhr"), key: "shalat.offset_dhuhr", typical_minute: 11 * 60 + 52 },
    Prayer { name: || tr!("shalat.asr"), key: "shalat.offset_asr", typical_minute: 15 * 60 + 14 },
    Prayer { name: || tr!("shalat.maghrib"), key: "shalat.offset_maghrib", typical_minute: 17 * 60 + 51 },
    Prayer { name: || tr!("shalat.isha"), key: "shalat.offset_isha", typical_minute: 19 * 60 + 3 },
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

/// "0", "+2", "−5": the minutes with their sign; the stepper adds the unit.
pub fn offset_value(offset: i64) -> String {
    match offset {
        0 => "0".to_owned(),
        offset if offset > 0 => format!("+{offset}"),
        offset => format!("−{}", -offset),
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

fn step_by(key: &'static str, delta: i64, offset: i64) -> Option<Step> {
    (stepped(offset, delta) != offset).then(|| {
        Rc::new(move |cx: &mut App| {
            let next = stepped(read_offset(&Settings::load(), key), delta);
            field::write(key, &next.to_string(), cx);
        }) as Step
    })
}

fn prayer_row(prayer: &'static Prayer) -> Item {
    let control: Render = Rc::new(move |_window, cx| {
        let offset = read_offset(&Settings::load(), prayer.key);
        stepper::stepper(
            prayer.key,
            SharedString::from(offset_value(offset)),
            tr!("stepper.minutes"),
            step_by(prayer.key, -1, offset),
            step_by(prayer.key, 1, offset),
            stepper::REGULAR,
            cx,
        )
    });
    Item::new((prayer.name)(), control)
}

/// The day bar, then a row for each prayer; the section draws them as one card with a divider between rows.
pub fn items() -> Vec<Item> {
    let day_bar = Item::picture_row(|_window, _cx| {
        let settings = Settings::load();
        let offsets: [i64; 6] = std::array::from_fn(|index| read_offset(&settings, PRAYERS[index].key));
        day_bar(&offsets)
    });
    std::iter::once(day_bar).chain(PRAYERS.iter().map(prayer_row)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings_with(contents: &str) -> Settings {
        let path = std::env::temp_dir().join(format!("foldline-settings-adjust-{}.txt", std::process::id()));
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
            let line = format!("Prayer::{} => \"{}\"", (prayer.name)(), prayer.key);
            assert!(source.contains(&line), "{line} missing in times.rs");
        }
    }

    #[test]
    fn the_card_holds_the_day_bar_then_one_row_for_each_prayer() {
        assert_eq!(items().len(), 1 + PRAYERS.len());
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
    fn the_offset_reads_with_its_sign() {
        assert_eq!(offset_value(0), "0");
        assert_eq!(offset_value(2), "+2");
        assert_eq!(offset_value(-30), "−30");
    }

    #[test]
    fn an_offset_moves_the_dot_along_the_bar_and_never_off_it() {
        let plain = bar_position(PRAYERS[2].typical_minute, 0);
        assert!(bar_position(PRAYERS[2].typical_minute, 30) > plain);
        assert!(bar_position(PRAYERS[2].typical_minute, -30) < plain);
        assert!((0.0..=1.0).contains(&bar_position(0, -30)));
        assert!((0.0..=1.0).contains(&bar_position(24 * 60, 30)));
    }
}
