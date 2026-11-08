use crate::gfx::{Align, FontStyle, UI_FONT, Weight};

pub const ARTWORK_NOTE: FontStyle = FontStyle::new(UI_FONT, 16.0, Weight::Medium, Align::Center);
pub const RESTING_LABEL: FontStyle = FontStyle::new(UI_FONT, 13.0, Weight::Medium, Align::Center);
pub const RESTING_LYRIC: FontStyle = FontStyle::new(UI_FONT, 13.0, Weight::Medium, Align::Center);
pub const TITLE: FontStyle = FontStyle::new(UI_FONT, 14.0, Weight::Medium, Align::Leading);
pub const DETAIL: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Regular, Align::Leading);
pub const LYRIC_LINE: FontStyle = FontStyle::new(UI_FONT, 17.0, Weight::Medium, Align::Center);
