//! Colors that several pages and activities share, named by the role they play on the black pill.
//!
//! A color used by one page stays in that page. A new color needs a rule in `docs/reference/design-rules.md`.

use crate::gfx::Color;

/// Secondary text and labels.
pub const TEXT_MUTED: Color = Color::rgb(0.604, 0.604, 0.651);
/// Hints, placeholders and inactive controls.
pub const TEXT_DIM: Color = Color::rgb(0.431, 0.431, 0.478);
/// Primary text on a dark surface that is not white.
pub const TEXT_LIGHT: Color = Color::rgb(0.902, 0.902, 0.922);
/// Light gray for the crosses of remove buttons, small icons and short messages.
pub const LIGHT_GRAY: Color = Color::rgb(0.769, 0.769, 0.8);

/// The one accent: selections, highlights and the break color of the timer.
pub const ACCENT_BLUE: Color = Color::rgb(0.302, 0.671, 0.969);
/// The accent as text on black.
pub const ACCENT_BLUE_LIGHT: Color = Color::rgb(0.455, 0.753, 0.988);
/// Full or destructive text (a full shelf, the trash, Sunday).
pub const SOFT_RED: Color = Color::rgb(1.0, 0.42, 0.42);
/// Bluetooth on, the prayer arc and a copied item.
pub const MINT_GREEN: Color = Color::rgb(0.388, 0.902, 0.745);
/// Charging, a done task and a camera in use.
pub const SUCCESS_GREEN: Color = Color::rgb(0.19, 0.82, 0.35);

/// A tile or row on black.
pub const SURFACE_ROW: Color = Color::rgb(0.078, 0.078, 0.094);
/// An empty-state badge, an offline row and a calendar row.
pub const SURFACE_BADGE: Color = Color::rgb(0.09, 0.09, 0.11);
/// A row or a button under the pointer.
pub const SURFACE_HOVER: Color = Color::rgb(0.11, 0.11, 0.133);
/// The fill of a round remove button.
pub const REMOVE_BUTTON_FILL: Color = Color::rgb(0.165, 0.165, 0.188);
/// Hairlines, tracks and the lightest fill raised above a row.
pub const SURFACE_BORDER: Color = Color::rgb(0.149, 0.149, 0.176);
/// The border of a button.
pub const SURFACE_BUTTON_BORDER: Color = Color::rgb(0.173, 0.173, 0.204);
/// A strong outline: a selected row and an empty ring.
pub const SURFACE_OUTLINE: Color = Color::rgb(0.227, 0.227, 0.267);

/// The app name on a notification card, also used by the rain banner.
pub const CARD_APP_NAME_TEXT: Color = Color::rgb(0.557, 0.557, 0.6);
/// The body of a notification card, also used by the rain banner.
pub const CARD_BODY_TEXT: Color = Color::rgb(0.678, 0.678, 0.722);
