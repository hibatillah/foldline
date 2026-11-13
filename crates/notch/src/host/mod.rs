//! Window orchestration stays independent of individual page implementations.

mod activity_layout;
mod chrome;
mod claimed_row;
mod clock;
mod display;
mod drop_target;
mod icon;
mod location_tracker;
mod monitor;
mod motion;
mod navigation;
mod notch;
mod notification_banners;
mod page_swap;
mod pill_row;
mod registry;
mod screen_mode;
mod scroll_gate;
mod settings;
mod settings_launcher;
mod tray;
mod visibility;
mod volume_flyout;
mod window;

pub use settings::Settings;
pub use window::run;
