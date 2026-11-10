//! Keeps the saved city where the device is: one read of Windows' position when something suggests a move.

mod network;

use std::{
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

use places::{City, Location};
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::{PostMessageW, WM_APP},
};

use crate::{
    log,
    platform::geolocation::{Reading, read_position},
};

pub use network::{unwatch, watch};

/// The worker found out where the device is (or why not); the answer waits in `take_answer`.
pub const WM_LOCATION_FOUND: u32 = WM_APP + 31;
/// A network interface changed: a new Wi-Fi, a cable, a VPN.
pub const WM_NETWORK_CHANGED: u32 = WM_APP + 32;

static ANSWER: Mutex<Option<Reading>> = Mutex::new(None);
static IS_READING: AtomicBool = AtomicBool::new(false);

/// What an answer changes in the settings: the access note, and the city when the device has moved to another one.
#[derive(Debug, PartialEq)]
pub struct Update {
    pub access: &'static str,
    pub city: Option<City>,
}

pub fn update_for(reading: Reading, saved: Option<&Location>) -> Update {
    match reading {
        Reading::Found { latitude, longitude } => {
            Update { access: "ok", city: places::city_to_save(saved, latitude, longitude) }
        }
        Reading::Denied => Update { access: "denied", city: None },
        Reading::Unavailable => Update { access: "unavailable", city: None },
    }
}

/// Reads the position on a worker thread, unless a read is already running, and posts `WM_LOCATION_FOUND`.
pub fn request(window: HWND) {
    if IS_READING.swap(true, Ordering::SeqCst) {
        return;
    }
    // The window handle crosses threads as an integer, since the raw handle type is not thread-safe.
    let window_address = window.0 as isize;
    let spawned = thread::Builder::new().name("location".into()).spawn(move || {
        let reading = read_position();
        if let Ok(mut answer) = ANSWER.lock() {
            *answer = Some(reading);
        }
        IS_READING.store(false, Ordering::SeqCst);
        let _ = unsafe { PostMessageW(Some(HWND(window_address as *mut _)), WM_LOCATION_FOUND, WPARAM(0), LPARAM(0)) };
    });
    if let Err(error) = spawned {
        IS_READING.store(false, Ordering::SeqCst);
        log::debug(&format!("could not start the location read: {error}"));
    }
}

/// What the worker would leave, for the tests of what the notch does with it.
#[cfg(test)]
pub fn leave_answer(reading: Reading) {
    *ANSWER.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(reading);
}

pub fn take_answer() -> Option<Reading> {
    ANSWER.lock().ok().and_then(|mut answer| answer.take())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn saved(label: &str, latitude: f64, longitude: f64) -> Location {
        Location { label: label.to_owned(), country: "ID".to_owned(), latitude, longitude }
    }

    #[test]
    fn a_position_with_no_saved_city_saves_the_nearest_one() {
        let update = update_for(Reading::Found { latitude: 0.528, longitude: 101.438 }, None);
        assert_eq!(update.access, "ok");
        assert_eq!(update.city.unwrap().label(), "Pekanbaru, ID");
    }

    #[test]
    fn a_position_in_the_saved_city_changes_nothing_but_the_access() {
        let jakarta = saved("Jakarta, ID", -6.175, 106.827);
        let update = update_for(Reading::Found { latitude: -6.2, longitude: 106.8 }, Some(&jakarta));
        assert_eq!(update, Update { access: "ok", city: None });
    }

    #[test]
    fn a_move_to_another_city_gives_that_city() {
        let jakarta = saved("Jakarta, ID", -6.175, 106.827);
        let update = update_for(Reading::Found { latitude: 0.528, longitude: 101.438 }, Some(&jakarta));
        assert_eq!(update.city.unwrap().name, "Pekanbaru");
    }

    #[test]
    fn denied_or_missing_access_keeps_the_saved_city_and_says_why() {
        let jakarta = saved("Jakarta, ID", -6.175, 106.827);
        assert_eq!(update_for(Reading::Denied, Some(&jakarta)), Update { access: "denied", city: None });
        assert_eq!(update_for(Reading::Unavailable, None), Update { access: "unavailable", city: None });
    }
}
