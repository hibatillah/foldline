use std::ops::RangeInclusive;

use crate::i18n::tr;
use crate::{
    field::{self, Kind},
    layout::{Group, Item, Page},
};

const TOP_RANGE: RangeInclusive<u32> = 0..=2000;

pub fn page() -> Page {
    Page::new(
        tr!("advanced.title"),
        tr!("advanced.description"),
        vec![Group::new(vec![
            Item::new(
                tr!("advanced.debug_log"),
                field::input("advanced.debug_log", Kind::Text { width: 280.0 }, tr!("advanced.debug_log_placeholder")),
            )
            .description(tr!("advanced.debug_log_note")),
            Item::new(
                tr!("advanced.move_down"),
                field::input("notch.top", Kind::WholeNumber { default: 0, range: TOP_RANGE }, "0"),
            )
            .description(tr!("advanced.move_down_note")),
        ])],
    )
}
