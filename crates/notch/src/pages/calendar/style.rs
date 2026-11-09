use crate::gfx::{Align, Color, FontStyle, UI_FONT, Weight};
use crate::ui::palette::{ACCENT_BLUE, LIGHT_GRAY, SOFT_RED, SURFACE_BADGE, SURFACE_BORDER, TEXT_DIM, TEXT_MUTED};
use crate::ui::type_styles::{MEDIUM_12, MEDIUM_13, REGULAR_10_CENTER, REGULAR_12, REGULAR_12_CENTER, REGULAR_13};

pub(super) const MONTH: FontStyle = FontStyle::new(UI_FONT, 20.0, Weight::Medium, Align::Leading);
pub(super) const YEAR: FontStyle = REGULAR_12;
pub(super) const DAY_LETTER: FontStyle = REGULAR_10_CENTER;
pub(super) const DAY_NUMBER: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Regular, Align::Center).tabular();
/// The design asks for semibold and Geist Mono; only regular and medium Geist are bundled.
pub(super) const TODAY_NUMBER: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Medium, Align::Center).tabular();
pub(super) const EVENT_TIME: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Regular, Align::Leading).tabular();
pub(super) const EVENT_TITLE: FontStyle = REGULAR_13;
pub(super) const EMPTY_TITLE: FontStyle = FontStyle::new(UI_FONT, 14.0, Weight::Medium, Align::Center);
pub(super) const EMPTY_DETAIL: FontStyle = REGULAR_12_CENTER;
pub(super) const MESSAGE_LEADING: FontStyle = MEDIUM_13;
pub(super) const MESSAGE_DETAIL: FontStyle = REGULAR_12;
pub(super) const EMPTY_HINT: FontStyle = FontStyle::new(UI_FONT, 11.5, Weight::Regular, Align::Center).wrapped();
pub(super) const BADGE: FontStyle = MEDIUM_12;
pub(super) const BADGE_TIME: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Medium, Align::Leading).tabular();
pub(super) const GRAY: Color = TEXT_MUTED;
pub(super) const DIM: Color = TEXT_DIM;
pub(super) const LIGHT: Color = LIGHT_GRAY;
pub(super) const BLUE: Color = ACCENT_BLUE;
pub(super) const TODAY_BLUE: Color = Color::rgb(0.184, 0.506, 0.969);
pub(super) const SUNDAY_RED: Color = SOFT_RED;
pub(super) const ROW: Color = SURFACE_BADGE;
pub(super) const ROW_BORDER: Color = SURFACE_BORDER;
