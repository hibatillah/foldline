//! Fonts and colors of the page.

use crate::gfx::{Align, Color, FontStyle, UI_FONT, Weight};

pub(super) const MONTH: FontStyle = FontStyle::new(UI_FONT, 20.0, Weight::Medium, Align::Leading);
pub(super) const YEAR: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Regular, Align::Leading);
pub(super) const DAY_LETTER: FontStyle = FontStyle::new(UI_FONT, 10.0, Weight::Regular, Align::Center);
pub(super) const DAY_NUMBER: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Regular, Align::Center).tabular();
/// The design asks for semibold and Geist Mono; only regular and medium Geist are bundled.
pub(super) const TODAY_NUMBER: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Medium, Align::Center).tabular();
pub(super) const EVENT_TIME: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Regular, Align::Leading).tabular();
pub(super) const EVENT_TITLE: FontStyle = FontStyle::new(UI_FONT, 13.0, Weight::Regular, Align::Leading);
pub(super) const EMPTY_TITLE: FontStyle = FontStyle::new(UI_FONT, 14.0, Weight::Medium, Align::Center);
pub(super) const EMPTY_DETAIL: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Regular, Align::Center);
pub(super) const MESSAGE_LEADING: FontStyle = FontStyle::new(UI_FONT, 13.0, Weight::Medium, Align::Leading);
pub(super) const MESSAGE_DETAIL: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Regular, Align::Leading);
pub(super) const EMPTY_HINT: FontStyle = FontStyle::new(UI_FONT, 11.5, Weight::Regular, Align::Center).wrapped();
pub(super) const BADGE: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Medium, Align::Leading);
pub(super) const BADGE_TIME: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Medium, Align::Leading).tabular();
pub(super) const GRAY: Color = Color::rgb(0.604, 0.604, 0.651);
pub(super) const DIM: Color = Color::rgb(0.431, 0.431, 0.478);
pub(super) const LIGHT: Color = Color::rgb(0.769, 0.769, 0.8);
pub(super) const BLUE: Color = Color::rgb(0.302, 0.671, 0.969);
pub(super) const TODAY_BLUE: Color = Color::rgb(0.184, 0.506, 0.969);
pub(super) const SUNDAY_RED: Color = Color::rgb(1.0, 0.42, 0.42);
pub(super) const ROW: Color = Color::rgb(0.090, 0.090, 0.110);
pub(super) const ROW_BORDER: Color = Color::rgb(0.149, 0.149, 0.176);
