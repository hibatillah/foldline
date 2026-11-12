//! Fonts, colors and icon shapes of the page, from the approved design.

use crate::gfx::{Align, Color, FontStyle, UI_FONT, Weight};
use crate::ui::palette::{
    ACCENT_BLUE_LIGHT, MINT_GREEN, SOFT_RED, SURFACE_BADGE, SURFACE_BORDER, SURFACE_BUTTON_BORDER, SURFACE_OUTLINE,
    TEXT_DIM, TEXT_LIGHT, TEXT_MUTED,
};
use crate::ui::type_styles::{MEDIUM_15, REGULAR_10, REGULAR_11, REGULAR_12};

/// The design asks for Geist Mono; only Geist Regular and Medium are bundled, so code uses Consolas, which every
/// supported Windows has.
const CODE_FONT: &str = "Consolas";

pub(super) const ROW_TEXT: FontStyle = REGULAR_11;
pub(super) const ROW_CODE: FontStyle = FontStyle::new(CODE_FONT, 10.5, Weight::Regular, Align::Leading);
pub(super) const ROW_CONFIRM: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Medium, Align::Leading);
pub(super) const PREVIEW_TEXT: FontStyle = REGULAR_12;
pub(super) const PREVIEW_CODE: FontStyle = FontStyle::new(CODE_FONT, 12.0, Weight::Regular, Align::Leading);
pub(super) const DETAIL: FontStyle = REGULAR_10;
pub(super) const BUTTON_LABEL: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Medium, Align::Leading);
pub(super) const BAR_TEXT: FontStyle = REGULAR_11;
pub(super) const BAR_ACTION: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Medium, Align::Trailing);
/// The design asks for semibold; only regular and medium are bundled.
pub(super) const EMPTY_TITLE: FontStyle = MEDIUM_15;
pub(super) const EMPTY_HINT: FontStyle = REGULAR_12;

pub(super) const ROW_FILL: Color = Color::rgb(0.071, 0.071, 0.086);
pub(super) const ROW_BORDER: Color = Color::rgb(0.118, 0.118, 0.141);
pub(super) const ROW_HOVER_FILL: Color = SURFACE_BADGE;
pub(super) const SELECTED_FILL: Color = Color::rgb(0.102, 0.102, 0.125);
pub(super) const SELECTED_BORDER: Color = SURFACE_OUTLINE;
pub(super) const TEXT: Color = TEXT_LIGHT;
pub(super) const ICON: Color = Color::rgb(0.541, 0.541, 0.588);
pub(super) const PANEL: Color = Color::rgb(0.043, 0.043, 0.055);
pub(super) const DETAIL_GRAY: Color = TEXT_DIM;
pub(super) const BUTTON_FILL: Color = Color::rgb(0.071, 0.071, 0.086);
pub(super) const BUTTON_BORDER: Color = SURFACE_BUTTON_BORDER;
pub(super) const BUTTON_BORDER_HOVER: Color = Color::rgb(0.294, 0.294, 0.341);
pub(super) const TRASH: Color = SOFT_RED;
pub(super) const COPY_GRAY: Color = TEXT_MUTED;
pub(super) const GREEN: Color = MINT_GREEN;
pub(super) const GREEN_FILL: Color = Color::rgb(0.051, 0.169, 0.141);
pub(super) const BAR: Color = SURFACE_BORDER;
pub(super) const UNDO_BLUE: Color = ACCENT_BLUE_LIGHT;
pub(super) const BADGE_FILL: Color = SURFACE_BADGE;
pub(super) const BADGE_BORDER: Color = SURFACE_BORDER;
pub(super) const HINT: Color = TEXT_MUTED;
/// The pin offered on a hovered row, lighter while the pointer is on it; a pinned row's pin is the accent blue.
pub(super) const PIN_OFFERED: Color = ICON;
pub(super) const PIN_OFFERED_HOVER: Color = TEXT_LIGHT;
pub(super) const PINNED: Color = ACCENT_BLUE_LIGHT;
/// A faint light ring on a row's picture, so a dark, white or see-through picture keeps its edge on the row.
pub(super) const THUMBNAIL_OUTLINE: Color = Color::WHITE.with_alpha(0.16);

pub(super) const TEXT_ICON: &str =
    "M6 3h12a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2zM8 8h8M8 12h8M8 16h5";
pub(super) const LINK_ICON: &str =
    "M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1 1M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1-1";
/// Lucide "image": a frame, the sun and a hill; a row shows it when its picture cannot be drawn.
pub(super) const IMAGE_ICON: &str = "M5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2zM7 9a2 2 0 1 0 4 0a2 2 0 1 0-4 0M21 15l-3.086-3.086a2 2 0 0 0-2.828 0L6 21";
pub(super) const CODE_ICON: &str = "M8 8l-4 4 4 4M16 8l4 4-4 4";
pub(super) const TRASH_ICON: &str = "M4 7h16M10 11v6M14 11v6M6 7l1 13h10l1-13M9 7V4h6v3";
pub(super) const COPY_ICON: &str =
    "M11 9h7a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2h-7a2 2 0 0 1-2-2v-7a2 2 0 0 1 2-2zM5 15V6a2 2 0 0 1 2-2h8";
pub(super) const PASTE_ICON: &str =
    "M8 5h8a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2zM9 5V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v1";
pub(super) const CHECK_ICON: &str = "m5 12 5 5 9-10";
