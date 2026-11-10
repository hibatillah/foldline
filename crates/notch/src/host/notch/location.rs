//! The saved city follows the device while `location.track` is on; the reading itself is in `host/location_tracker.rs`.

use places::Location;
use windows::Win32::UI::WindowsAndMessaging::{KillTimer, SetTimer};

use super::{LOCATION_SETTLE_TIMER_ID, Notch};
use crate::{host::location_tracker, log};

const SETTLE_DELAY_MILLISECONDS: u32 = 10_000;

impl Notch {
    /// Follows the switch: watching for moves starts and the position is read when it goes on, and both stop when it
    /// goes off.
    pub(super) fn sync_location_tracking(&mut self) {
        let wants_tracking = places::is_tracking(&|key| self.settings.get_string(key).map(str::to_owned));
        if wants_tracking == self.is_tracking_location {
            return;
        }
        self.is_tracking_location = wants_tracking;
        if wants_tracking {
            location_tracker::watch(self.window);
            location_tracker::request(self.window);
        } else {
            location_tracker::unwatch();
            let _ = unsafe { KillTimer(Some(self.window), LOCATION_SETTLE_TIMER_ID) };
        }
    }

    /// After waking from sleep the device may be in another place.
    pub fn request_location(&self) {
        if self.is_tracking_location {
            location_tracker::request(self.window);
        }
    }

    /// Every change starts the wait again, so one read follows a burst.
    pub fn network_changed(&self) {
        if self.is_tracking_location {
            unsafe { SetTimer(Some(self.window), LOCATION_SETTLE_TIMER_ID, SETTLE_DELAY_MILLISECONDS, None) };
        }
    }

    pub(super) fn location_settled(&self) {
        let _ = unsafe { KillTimer(Some(self.window), LOCATION_SETTLE_TIMER_ID) };
        self.request_location();
    }

    /// Takes the worker's answer: notes the access, and saves the new city exactly as a manual choice would.
    pub fn location_found(&mut self) {
        let Some(reading) = location_tracker::take_answer() else { return };
        if !self.is_tracking_location {
            return;
        }
        let saved = Location::read(&|key| self.settings.get_string(key).map(str::to_owned));
        let update = location_tracker::update_for(reading, saved.as_ref());
        let Some(city) = update.city else {
            if update.access != "ok" {
                log::debug(&format!("location: no position ({})", update.access));
            }
            return;
        };
        log::debug(&format!("location: the device is in {}", city.label()));
        for (key, value) in places::values_of(&city) {
            self.settings.set_string(key, &value);
        }
        self.reload_settings();
    }
}
