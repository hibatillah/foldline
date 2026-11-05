//! The panel's tabs and the controls on the general and behavior tabs.

use super::START_WITH_WINDOWS_KEY;
use crate::page::Control;

pub(super) const SHOWING_OPTIONS: &[(&str, &str)] =
    &[("hidden", "Hidden"), ("activities", "Activities"), ("lyrics", "Lyrics"), ("music", "Always")];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    /// The settings of the page that was open.
    /// One of the open page's settings tabs, by position.
    Page(usize),
    General,
    Behavior,
    Pages,
}

pub fn general_controls() -> Vec<Control> {
    vec![
        Control::Switch {
            key: START_WITH_WINDOWS_KEY,
            label: "Start with Windows",
            hint: "when you sign in",
            default: true,
        },
        Control::Switch {
            key: "notch.hide_when_idle",
            label: "Hide when idle",
            hint: "point at the top to show it",
            default: false,
        },
        Control::Switch {
            key: "activity.notifications",
            label: "Notifications",
            hint: "banners from Windows",
            default: true,
        },
        Control::Switch {
            key: "notifications.show_text",
            label: "Notification text",
            hint: "off reads \"New message\"",
            default: true,
        },
    ]
}

pub fn behavior_controls() -> Vec<Control> {
    vec![
        Control::Choice {
            key: "notch.maximized",
            label: "Over a maximized window",
            options: SHOWING_OPTIONS,
            default: "lyrics",
        },
        Control::Choice {
            key: "notch.fullscreen",
            label: "Over a full-screen app",
            options: SHOWING_OPTIONS,
            default: "hidden",
        },
        Control::Switch {
            key: "notch.hover_maximized",
            label: "Open on hover",
            hint: "over a maximized window",
            default: true,
        },
        Control::Switch {
            key: "notch.hover_fullscreen",
            label: "Open on hover",
            hint: "over a full-screen app",
            default: true,
        },
    ]
}
