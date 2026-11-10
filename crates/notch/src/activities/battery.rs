//! Charger and low-battery changes arrive through Windows power settings without polling.

use crate::i18n::tr;
use windows::core::GUID;

use crate::ui::palette::SUCCESS_GREEN;
use crate::{
    activity::{Activity, Flash, Presence, Slot, SystemMessage},
    gfx::{Canvas, Color, Point, Rect},
    page::{Context, TimerId},
    ui::{self, Icon, Ring},
};

/// GUID_ACDC_POWER_SOURCE: 0 is the mains, 1 the battery, 2 a short-term source such as a UPS.
const POWER_SOURCE_SETTING: GUID = GUID::from_u128(0x5d3e9a59_e9d5_4b00_a6bd_ff34ff516548);
const BATTERY_PERCENTAGE_SETTING: GUID = GUID::from_u128(0xa7ad8041_b45a_4cae_87a3_eecbb468a9e1);
const ON_BATTERY: u32 = 1;
// Lowest first so crossing both thresholds reports the more urgent warning.
const LOW_LEVELS: [u32; 2] = [10, 20];
const SHOW_MILLISECONDS: u32 = 3000;
const LOW_RED: Color = Color::rgb(1.0, 0.27, 0.23);
const IDLE_GREEN: Color = Color::rgb(0.18, 0.62, 0.27);
/// Diameter of the ring beside the clock, which is also the width it asks for.
const RING_DIAMETER: f32 = 14.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Event {
    PluggedIn,
    Unplugged,
    Low(u32),
}

/// Turns Windows' reports into events; the first report of each kind is only the starting state.
#[derive(Default)]
struct Tracker {
    is_on_battery: Option<bool>,
    percent: Option<u32>,
}

impl Tracker {
    fn power_source(&mut self, value: u32) -> Option<Event> {
        let is_on_battery = value == ON_BATTERY;
        let previous = self.is_on_battery.replace(is_on_battery)?;
        match (previous, is_on_battery) {
            (false, true) => Some(Event::Unplugged),
            (true, false) => Some(Event::PluggedIn),
            _ => None,
        }
    }

    fn percentage(&mut self, value: u32) -> Option<Event> {
        let previous = self.percent.replace(value)?;
        if self.is_on_battery != Some(true) {
            return None;
        }
        LOW_LEVELS.into_iter().find(|&level| previous > level && value <= level).map(Event::Low)
    }
}

/// Charging wins over low; the low color needs the battery to be the source.
fn ring_color(percent: u32, is_on_battery: Option<bool>) -> Color {
    match is_on_battery {
        Some(false) => SUCCESS_GREEN,
        Some(true) if percent <= LOW_LEVELS[LOW_LEVELS.len() - 1] => LOW_RED,
        _ => IDLE_GREEN,
    }
}

fn ring_fraction(percent: u32) -> f32 {
    percent.min(100) as f32 / 100.0
}

pub struct BatteryActivity {
    tracker: Tracker,
    event: Event,
    flash: Flash,
    width: f32,
    has_battery: bool,
}

impl BatteryActivity {
    pub fn new() -> Self {
        Self {
            tracker: Tracker::default(),
            event: Event::Unplugged,
            flash: Flash::default(),
            width: 0.0,
            has_battery: crate::platform::has_system_battery(),
        }
    }

    fn percent_text(&self) -> Option<String> {
        self.tracker.percent.map(|percent| format!("{percent}%"))
    }

    fn message(&self) -> std::borrow::Cow<'static, str> {
        match self.event {
            Event::PluggedIn => tr!("battery.charging"),
            Event::Unplugged => tr!("battery.on_battery"),
            Event::Low(_) => tr!("battery.low"),
        }
    }

    fn announce(&mut self, event: Event, context: &mut Context) {
        self.event = event;
        self.width = ui::message_width(context.fonts(), &self.message(), self.percent_text().as_deref());
        self.flash.show(context, SHOW_MILLISECONDS);
    }
}

impl Activity for BatteryActivity {
    fn id(&self) -> &'static str {
        "battery"
    }

    fn uses_units(&self) -> bool {
        true
    }

    fn presence(&self) -> Option<Presence> {
        if self.flash.is_visible() {
            return Some(Presence::Transient { width: self.width });
        }
        self.tracker
            .percent
            .filter(|_| self.has_battery)
            .map(|_| Presence::Companion { slot: Slot::Ring, width: RING_DIAMETER })
    }

    fn draw(&self, canvas: &Canvas, area: Rect, _scale: f32, opacity: f32) {
        if !self.flash.is_visible() {
            let Some(percent) = self.tracker.percent else { return };
            let center = Point::new(area.center_x(), area.center_y());
            let color = ring_color(percent, self.tracker.is_on_battery);
            Ring { center, diameter: RING_DIAMETER }.draw(canvas, ring_fraction(percent), color, opacity);
            return;
        }
        let (icon, tint) = match self.event {
            Event::PluggedIn => (Icon::Bolt, SUCCESS_GREEN),
            Event::Unplugged => (Icon::Battery, Color::WHITE),
            Event::Low(_) => (Icon::BatteryLow, LOW_RED),
        };
        ui::draw_message(canvas, area, opacity, icon, tint, &self.message(), self.percent_text().as_deref());
    }

    fn on_start(&mut self, context: &mut Context) {
        context.listen_to_power_setting(&POWER_SOURCE_SETTING);
        context.listen_to_power_setting(&BATTERY_PERCENTAGE_SETTING);
    }

    fn on_timer(&mut self, timer: TimerId, context: &mut Context) {
        self.flash.on_timer(timer, context);
    }

    fn on_system_message(&mut self, message: &SystemMessage, context: &mut Context) {
        let SystemMessage::PowerSettingChanged { setting, value } = message else { return };
        let event = if *setting == POWER_SOURCE_SETTING {
            self.tracker.power_source(*value)
        } else if *setting == BATTERY_PERCENTAGE_SETTING {
            self.tracker.percentage(*value)
        } else {
            None
        };
        if let Some(event) = event {
            self.announce(event, context);
        } else {
            context.request_repaint();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn on_battery(percent: u32) -> Tracker {
        let mut tracker = Tracker::default();
        tracker.power_source(ON_BATTERY);
        tracker.percentage(percent);
        tracker
    }

    #[test]
    fn the_ring_is_green_while_idle_and_bright_green_while_charging() {
        assert_eq!(ring_color(80, None), IDLE_GREEN);
        assert_eq!(ring_color(80, Some(true)), IDLE_GREEN);
        assert_eq!(ring_color(80, Some(false)), SUCCESS_GREEN);
    }

    #[test]
    fn the_ring_turns_red_at_the_low_threshold_only_on_battery() {
        assert_eq!(ring_color(20, Some(true)), LOW_RED);
        assert_eq!(ring_color(21, Some(true)), IDLE_GREEN);
        assert_eq!(ring_color(10, Some(false)), SUCCESS_GREEN);
    }

    #[test]
    fn the_ring_arc_is_the_charge_fraction() {
        assert_eq!(ring_fraction(0), 0.0);
        assert_eq!(ring_fraction(50), 0.5);
        assert_eq!(ring_fraction(120), 1.0);
    }

    #[test]
    fn a_known_battery_percent_shows_a_ring_before_the_clock() {
        let mut activity = BatteryActivity::new();
        activity.has_battery = true;
        assert_eq!(activity.presence(), None);
        activity.tracker.percentage(64);
        assert_eq!(activity.presence(), Some(Presence::Companion { slot: Slot::Ring, width: RING_DIAMETER }));
    }

    #[test]
    fn a_pc_without_a_battery_shows_no_ring() {
        let mut activity = BatteryActivity::new();
        activity.has_battery = false;
        activity.tracker.percentage(100);
        assert_eq!(activity.presence(), None);
    }

    #[test]
    fn a_visible_message_keeps_priority_over_the_ring() {
        let mut activity = BatteryActivity::new();
        activity.tracker.percentage(64);
        activity.flash = Flash::visible();
        activity.width = 120.0;
        assert_eq!(activity.presence(), Some(Presence::Transient { width: 120.0 }));
    }

    #[test]
    fn the_starting_power_source_is_not_an_event() {
        assert_eq!(Tracker::default().power_source(0), None);
        assert_eq!(Tracker::default().power_source(ON_BATTERY), None);
    }

    #[test]
    fn unplugging_and_plugging_in_are_events() {
        let mut tracker = Tracker::default();
        tracker.power_source(0);
        assert_eq!(tracker.power_source(ON_BATTERY), Some(Event::Unplugged));
        assert_eq!(tracker.power_source(0), Some(Event::PluggedIn));
    }

    #[test]
    fn a_repeated_power_source_is_not_an_event() {
        let mut tracker = Tracker::default();
        tracker.power_source(0);
        assert_eq!(tracker.power_source(0), None);
    }

    #[test]
    fn a_short_term_source_counts_as_plugged_in() {
        let mut tracker = Tracker::default();
        tracker.power_source(ON_BATTERY);
        assert_eq!(tracker.power_source(2), Some(Event::PluggedIn));
    }

    #[test]
    fn crossing_20_percent_on_battery_warns_once() {
        let mut tracker = on_battery(22);
        assert_eq!(tracker.percentage(21), None);
        assert_eq!(tracker.percentage(20), Some(Event::Low(20)));
        assert_eq!(tracker.percentage(19), None);
    }

    #[test]
    fn crossing_10_percent_warns_again() {
        let mut tracker = on_battery(12);
        assert_eq!(tracker.percentage(10), Some(Event::Low(10)));
    }

    #[test]
    fn a_drop_through_both_levels_reports_the_lower_one() {
        let mut tracker = on_battery(25);
        assert_eq!(tracker.percentage(8), Some(Event::Low(10)));
    }

    #[test]
    fn a_low_charge_does_not_warn_while_plugged_in() {
        let mut tracker = Tracker::default();
        tracker.power_source(0);
        tracker.percentage(25);
        assert_eq!(tracker.percentage(15), None);
    }

    #[test]
    fn the_first_percentage_is_only_the_starting_point() {
        let mut tracker = Tracker::default();
        tracker.power_source(ON_BATTERY);
        assert_eq!(tracker.percentage(5), None);
    }
}
