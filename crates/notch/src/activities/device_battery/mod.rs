//! Warns when a connected Bluetooth device's battery crosses 20 % or 10 %; Windows pushes the levels, nothing polls.

mod naming;
mod tracker;
mod watcher;
mod worker;

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    thread,
};

use tracker::Warning;
use worker::Shared;

use crate::{
    activity::{Activity, Flash, Presence},
    gfx::{Canvas, Color, Rect},
    page::{Context, TimerId},
    ui::{self, Icon},
};

const SHOW_MILLISECONDS: u32 = 3000;
const LOW_RED: Color = Color::rgb(1.0, 0.27, 0.23);

pub struct DeviceBatteryActivity {
    shared: Option<Arc<Shared>>,
    current: Option<Warning>,
    width: f32,
    flash: Flash,
}

impl DeviceBatteryActivity {
    pub fn new() -> Self {
        Self { shared: None, current: None, width: 0.0, flash: Flash::default() }
    }
}

fn percent_text(warning: &Warning) -> String {
    format!("{}%", warning.percent)
}

impl Activity for DeviceBatteryActivity {
    fn id(&self) -> &'static str {
        "device_battery"
    }

    fn uses_units(&self) -> bool {
        true
    }

    fn is_optional(&self) -> bool {
        true
    }

    fn presence(&self) -> Option<Presence> {
        self.flash.is_visible().then_some(Presence::Transient { width: self.width })
    }

    fn draw(&self, canvas: &Canvas, area: Rect, _scale: f32, opacity: f32) {
        let Some(warning) = &self.current else { return };
        let value = percent_text(warning);
        ui::draw_message(
            canvas,
            area,
            opacity,
            Icon::BatteryLow,
            LOW_RED,
            &naming::message(&warning.name),
            Some(&value),
        );
    }

    fn on_start(&mut self, context: &mut Context) {
        let shared = Arc::new(Shared { warnings: Mutex::new(VecDeque::new()), notifier: context.notifier() });
        self.shared = Some(shared.clone());
        thread::spawn(move || worker::run(shared));
    }

    fn on_timer(&mut self, timer: TimerId, context: &mut Context) {
        self.flash.on_timer(timer, context);
    }

    fn on_notify(&mut self, context: &mut Context) {
        let Some(shared) = &self.shared else { return };
        let latest = shared.warnings.lock().ok().and_then(|mut warnings| {
            let latest = warnings.pop_back();
            warnings.clear();
            latest
        });
        if let Some(warning) = latest {
            let value = percent_text(&warning);
            self.width = ui::message_width(context.fonts(), &naming::message(&warning.name), Some(&value));
            self.current = Some(warning);
            self.flash.show(context, SHOW_MILLISECONDS);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_shows_before_a_warning() {
        assert_eq!(DeviceBatteryActivity::new().presence(), None);
    }

    #[test]
    fn a_visible_warning_takes_over_the_pill_at_its_width() {
        let mut activity = DeviceBatteryActivity::new();
        activity.flash = Flash::visible();
        activity.width = 150.0;
        assert_eq!(activity.presence(), Some(Presence::Transient { width: 150.0 }));
    }

    #[test]
    fn the_activity_is_optional_and_keyed_by_its_id() {
        let activity = DeviceBatteryActivity::new();
        assert!(activity.is_optional());
        assert_eq!(activity.id(), "device_battery");
    }

    #[test]
    fn the_value_is_the_percentage() {
        assert_eq!(percent_text(&Warning { name: "MX".to_owned(), level: 20, percent: 18 }), "18%");
    }
}
