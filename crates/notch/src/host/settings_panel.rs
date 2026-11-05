//! The settings panel inside the notch: tabs in the header, controls below; sizes are device-independent pixels.

pub use controls::{Tab, behavior_controls, general_controls};
pub use draw::{Editing, draw_control, draw_tabs};
pub use layout::{Part, Placed, TabsLayout, content_height, hit, layout, tabs_layout};
use style::*;
use values::{is_on, number};
pub use values::{picked, pressed_value};

mod controls;
mod draw;
mod layout;
mod style;
#[cfg(test)]
mod tests;
mod values;

pub const PANEL_WIDTH: f32 = 540.0;

pub const TEXT_LIMIT: usize = 400;

/// Startup lives in the registry, not in settings.txt; the host reads and writes it through `startup`.
pub const START_WITH_WINDOWS_KEY: &str = "notch.start_with_windows";
