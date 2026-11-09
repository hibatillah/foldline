//! Text styles that several pages and activities share, named `<weight>_<size>` with the alignment last when it is not
//! leading (`REGULAR_12`, `MEDIUM_11_CENTER`).
//!
//! The design asks for semibold (600) in places; only Regular and Medium are bundled, so Medium stands in everywhere.
//! A style used by one page stays in that page.

use crate::gfx::{Align, FontStyle, UI_FONT, Weight};

pub const REGULAR_10: FontStyle = FontStyle::new(UI_FONT, 10.0, Weight::Regular, Align::Leading);
pub const REGULAR_10_CENTER: FontStyle = FontStyle::new(UI_FONT, 10.0, Weight::Regular, Align::Center);
pub const REGULAR_11: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Regular, Align::Leading);
pub const REGULAR_12: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Regular, Align::Leading);
pub const REGULAR_12_CENTER: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Regular, Align::Center);
pub const REGULAR_13: FontStyle = FontStyle::new(UI_FONT, 13.0, Weight::Regular, Align::Leading);
pub const REGULAR_14: FontStyle = FontStyle::new(UI_FONT, 14.0, Weight::Regular, Align::Leading);

pub const MEDIUM_11_CENTER: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Medium, Align::Center);
pub const MEDIUM_12: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Medium, Align::Leading);
pub const MEDIUM_13: FontStyle = FontStyle::new(UI_FONT, 13.0, Weight::Medium, Align::Leading);
pub const MEDIUM_14: FontStyle = FontStyle::new(UI_FONT, 14.0, Weight::Medium, Align::Leading);
pub const MEDIUM_15: FontStyle = FontStyle::new(UI_FONT, 15.0, Weight::Medium, Align::Leading);
