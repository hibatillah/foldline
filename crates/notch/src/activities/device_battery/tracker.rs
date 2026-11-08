use std::collections::HashMap;

use super::naming::device_name;

/// Lowest first, so crossing both levels at once reports the more urgent warning.
pub(super) const LOW_LEVELS: [u8; 2] = [10, 20];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Warning {
    pub name: String,
    pub level: u8,
    pub percent: u8,
}

/// What a watcher event says about one node; `None` means the event did not carry that property.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Change {
    pub percent: Option<u8>,
    pub is_connected: Option<bool>,
    /// Part of the first listing when the watcher starts, which is a starting point and not news.
    pub is_initial: bool,
}

#[derive(Default)]
struct Device {
    name: String,
    /// `None` until Windows reports the connection, which not every node does.
    is_connected: Option<bool>,
    percent: Option<u8>,
    /// Set when the device connects live; its next reading is judged on its own, not against the last one.
    judges_next_reading: bool,
}

/// Turns battery readings into warnings; the first reading of a device is only its starting point.
/// A device has several nodes that share its name, so readings are tracked by that name.
#[derive(Default)]
pub(super) struct Tracker {
    names_by_node: HashMap<String, String>,
    devices: HashMap<String, Device>,
}

fn crossed_level(previous: u8, value: u8) -> Option<u8> {
    LOW_LEVELS.into_iter().find(|&level| previous > level && value <= level)
}

fn level_at(value: u8) -> Option<u8> {
    LOW_LEVELS.into_iter().find(|&level| value <= level)
}

impl Tracker {
    /// `name` comes with the node's first event only; later events of the node carry just its id.
    pub(super) fn apply(&mut self, node_id: &str, name: Option<&str>, change: Change) -> Option<Warning> {
        if let Some(name) = name {
            self.names_by_node.insert(node_id.to_owned(), device_name(name));
        }
        let key = self.names_by_node.get(node_id)?;
        let device = self.devices.entry(key.clone()).or_default();
        device.name.clone_from(key);
        if let Some(is_connected) = change.is_connected {
            let was_connected = device.is_connected.replace(is_connected);
            if !is_connected {
                device.percent = None;
                device.judges_next_reading = false;
            } else if was_connected != Some(true) && !change.is_initial {
                device.percent = None;
                device.judges_next_reading = true;
            }
        }
        let percent = change.percent?;
        let previous = device.percent.replace(percent);
        if device.is_connected == Some(false) {
            return None;
        }
        // Without a connection report, a first reading after the start is the device connecting.
        let is_first_after_start = previous.is_none() && device.is_connected.is_none() && !change.is_initial;
        let level = if std::mem::take(&mut device.judges_next_reading) || is_first_after_start {
            level_at(percent)
        } else {
            crossed_level(previous?, percent)
        };
        level.map(|level| Warning { name: device.name.clone(), level, percent })
    }

    pub(super) fn forget(&mut self, node_id: &str) {
        self.names_by_node.remove(node_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reading(percent: u8) -> Change {
        Change { percent: Some(percent), ..Change::default() }
    }

    fn starting_reading(percent: u8) -> Change {
        Change { percent: Some(percent), is_initial: true, ..Change::default() }
    }

    fn connection(is_connected: bool, is_initial: bool) -> Change {
        Change { is_connected: Some(is_connected), is_initial, ..Change::default() }
    }

    fn level(warning: Option<Warning>) -> Option<u8> {
        warning.map(|warning| warning.level)
    }

    fn connected_at(percent: u8) -> Tracker {
        let mut tracker = Tracker::default();
        tracker.apply("mouse", Some("MX Master"), connection(true, true));
        tracker.apply("mouse", None, starting_reading(percent));
        tracker
    }

    #[test]
    fn the_first_reading_is_only_the_starting_point() {
        let mut tracker = Tracker::default();
        tracker.apply("mouse", Some("MX Master"), connection(true, true));
        assert_eq!(tracker.apply("mouse", None, starting_reading(5)), None);
        assert_eq!(tracker.apply("mouse", None, reading(4)), None);
    }

    #[test]
    fn crossing_20_percent_warns_once() {
        let mut tracker = connected_at(22);
        assert_eq!(tracker.apply("mouse", None, reading(21)), None);
        let warning = tracker.apply("mouse", None, reading(20)).expect("warns at 20");
        assert_eq!((warning.name.as_str(), warning.level), ("MX Master", 20));
        assert_eq!(tracker.apply("mouse", None, reading(19)), None);
    }

    #[test]
    fn crossing_10_percent_warns_again() {
        let mut tracker = connected_at(12);
        assert_eq!(level(tracker.apply("mouse", None, reading(10))), Some(10));
        assert_eq!(tracker.apply("mouse", None, reading(9)), None);
    }

    #[test]
    fn a_drop_through_both_levels_reports_the_lower_one() {
        let mut tracker = connected_at(25);
        assert_eq!(level(tracker.apply("mouse", None, reading(8))), Some(10));
    }

    #[test]
    fn recovering_above_a_level_arms_the_warning_again() {
        let mut tracker = connected_at(22);
        tracker.apply("mouse", None, reading(19));
        assert_eq!(tracker.apply("mouse", None, reading(60)), None);
        assert_eq!(level(tracker.apply("mouse", None, reading(20))), Some(20));
    }

    #[test]
    fn a_repeated_reading_does_not_warn() {
        let mut tracker = connected_at(30);
        tracker.apply("mouse", None, reading(18));
        assert_eq!(tracker.apply("mouse", None, reading(18)), None);
    }

    #[test]
    fn devices_are_tracked_apart() {
        let mut tracker = connected_at(22);
        tracker.apply("keyboard", Some("K380"), connection(true, true));
        tracker.apply("keyboard", None, starting_reading(50));
        assert_eq!(tracker.apply("keyboard", None, reading(40)), None);
        assert!(tracker.apply("mouse", None, reading(19)).is_some());
    }

    #[test]
    fn the_nodes_of_one_device_share_its_battery_state() {
        let mut tracker = Tracker::default();
        tracker.apply("headset-a", Some("DAXA SL1"), starting_reading(22));
        tracker.apply("headset-b", Some("DAXA SL1 Hands-Free"), starting_reading(22));
        assert_eq!(level(tracker.apply("headset-a", None, reading(20))), Some(20));
        assert_eq!(tracker.apply("headset-b", None, reading(20)), None);
    }

    #[test]
    fn a_disconnected_device_never_warns() {
        let mut tracker = connected_at(25);
        tracker.apply("mouse", None, connection(false, false));
        assert_eq!(tracker.apply("mouse", None, reading(15)), None);
    }

    #[test]
    fn a_device_that_connects_already_low_warns_on_its_first_reading() {
        let mut tracker = Tracker::default();
        tracker.apply("mouse", Some("MX Master"), connection(false, true));
        tracker.apply("mouse", None, connection(true, false));
        assert_eq!(level(tracker.apply("mouse", None, reading(18))), Some(20));
        assert_eq!(tracker.apply("mouse", None, reading(17)), None);
    }

    #[test]
    fn a_device_that_connects_below_10_percent_warns_with_the_lower_level() {
        let mut tracker = Tracker::default();
        tracker.apply("mouse", Some("MX Master"), connection(false, true));
        tracker.apply("mouse", None, connection(true, false));
        assert_eq!(level(tracker.apply("mouse", None, reading(7))), Some(10));
    }

    #[test]
    fn a_device_that_connects_with_a_healthy_battery_does_not_warn() {
        let mut tracker = Tracker::default();
        tracker.apply("mouse", Some("MX Master"), connection(false, true));
        tracker.apply("mouse", None, connection(true, false));
        assert_eq!(tracker.apply("mouse", None, reading(60)), None);
    }

    #[test]
    fn a_low_device_already_connected_when_the_watcher_starts_does_not_warn() {
        let mut tracker = Tracker::default();
        tracker.apply("mouse", Some("MX Master"), connection(true, true));
        assert_eq!(tracker.apply("mouse", None, reading(5)), None);
    }

    #[test]
    fn a_reconnect_judges_the_new_reading_not_the_old_one() {
        let mut tracker = connected_at(50);
        tracker.apply("mouse", None, connection(false, false));
        tracker.apply("mouse", None, connection(true, false));
        assert_eq!(tracker.apply("mouse", None, reading(40)), None);
        assert_eq!(level(tracker.apply("mouse", None, reading(19))), Some(20));
    }

    #[test]
    fn without_a_connection_report_a_first_reading_after_the_start_counts_as_connecting() {
        let mut tracker = Tracker::default();
        tracker.apply("mouse", Some("MX Master"), Change { is_initial: true, ..Change::default() });
        assert_eq!(level(tracker.apply("mouse", None, reading(15))), Some(20));
        assert_eq!(tracker.apply("mouse", None, reading(14)), None);
    }

    #[test]
    fn a_node_that_was_never_listed_is_ignored() {
        assert_eq!(Tracker::default().apply("unknown", None, reading(5)), None);
    }

    #[test]
    fn a_forgotten_node_is_ignored_until_it_is_listed_again() {
        let mut tracker = connected_at(25);
        tracker.forget("mouse");
        assert_eq!(tracker.apply("mouse", None, reading(5)), None);
    }
}
