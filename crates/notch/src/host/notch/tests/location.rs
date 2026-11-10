//! What a position the worker found does to the saved city.

use std::{path::PathBuf, sync::Mutex};

use super::*;
use crate::{host::location_tracker::leave_answer, platform::geolocation::Reading};

/// The worker's answer is one process-wide slot, so the tests that use it run one at a time.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

fn settings_file(name: &str, text: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("notch-location-{}-{name}", std::process::id())).join("settings.txt");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, text).unwrap();
    path
}

fn tracking_notch(name: &str, text: &str, is_tracking: bool) -> (Notch, PathBuf) {
    let path = settings_file(name, text);
    let mut notch = custom_rig(Vec::new(), Settings::at(Some(path.clone())), Vec::new());
    notch.is_tracking_location = is_tracking;
    (notch, path)
}

fn saved_label(notch: &Notch) -> Option<String> {
    notch.settings.get_string(places::CITY_KEY).map(str::to_owned)
}

const JAKARTA: &str = "location.city=Jakarta, ID\nlocation.lat=-6.175\nlocation.lon=106.827\nlocation.country=ID\n";
const PEKANBARU: Reading = Reading::Found { latitude: 0.528, longitude: 101.438 };

#[test]
fn a_position_in_another_city_saves_that_city() {
    let _guard = ONE_AT_A_TIME.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let (mut notch, path) = tracking_notch("moved", JAKARTA, true);
    leave_answer(PEKANBARU);
    notch.location_found();
    assert_eq!(saved_label(&notch).as_deref(), Some("Pekanbaru, ID"));
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn an_answer_that_arrives_after_tracking_was_switched_off_changes_nothing() {
    let _guard = ONE_AT_A_TIME.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let (mut notch, path) = tracking_notch("off", &format!("{JAKARTA}location.track=off\n"), false);
    leave_answer(PEKANBARU);
    notch.location_found();
    assert_eq!(saved_label(&notch).as_deref(), Some("Jakarta, ID"));
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn windows_location_being_off_keeps_the_city() {
    let _guard = ONE_AT_A_TIME.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let (mut notch, path) = tracking_notch("denied", JAKARTA, true);
    leave_answer(Reading::Denied);
    notch.location_found();
    assert_eq!(saved_label(&notch).as_deref(), Some("Jakarta, ID"));
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn switching_tracking_off_in_the_settings_stops_it() {
    let (mut notch, path) = tracking_notch("switch", "location.track=off\n", true);
    notch.sync_location_tracking();
    assert!(!notch.is_tracking_location);
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}
