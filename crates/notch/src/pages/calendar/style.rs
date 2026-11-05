//! Fonts and colors of the page.

use crate::gfx::{Align, Color, FontStyle, UI_FONT, Weight};

pub(super) const MONTH: FontStyle = FontStyle::new(UI_FONT, 22.0, Weight::Medium, Align::Leading);
pub(super) const YEAR: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Regular, Align::Leading);
pub(super) const DAY_LETTER: FontStyle = FontStyle::new(UI_FONT, 9.0, Weight::Medium, Align::Center);
pub(super) const DAY_NUMBER: FontStyle = FontStyle::new("Consolas", 13.0, Weight::Regular, Align::Center);
pub(super) const EVENT_TIME: FontStyle = FontStyle::new("Consolas", 12.0, Weight::Regular, Align::Leading);
pub(super) const EVENT_TITLE: FontStyle = FontStyle::new(UI_FONT, 13.0, Weight::Medium, Align::Leading);
pub(super) const EVENT_DETAIL: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Regular, Align::Leading);
pub(super) const EVENT_TRAILING: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Regular, Align::Trailing);
pub(super) const JOIN: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Medium, Align::Center);
pub(super) const EMPTY: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Regular, Align::Center).wrapped();
pub(super) const BADGE: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Medium, Align::Leading);
pub(super) const BADGE_TIME: FontStyle = FontStyle::new("Consolas", 12.0, Weight::Medium, Align::Leading);
pub(super) const GRAY: Color = Color::rgb(0.604, 0.604, 0.651);
pub(super) const DIM: Color = Color::rgb(0.431, 0.431, 0.478);
pub(super) const LIGHT: Color = Color::rgb(0.769, 0.769, 0.8);
pub(super) const BLUE: Color = Color::rgb(0.302, 0.671, 0.969);
pub(super) const PALE_BLUE: Color = Color::rgb(0.455, 0.753, 0.988);
pub(super) const TODAY_FILL: Color = Color::rgb(0.110, 0.494, 0.839);
pub(super) const SUNDAY_RED: Color = Color::rgb(1.0, 0.42, 0.42);
pub(super) const ROW: Color = Color::rgb(0.078, 0.078, 0.094);
pub(super) const ROW_NEXT: Color = Color::rgb(0.063, 0.141, 0.227);
pub(super) const DIVIDER: Color = Color::rgb(0.149, 0.149, 0.176);
pub(super) const EMPTY_HINT: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Regular, Align::Center).wrapped();
