//! Reads the devices on a short-lived worker thread; nothing runs between reads.

use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

use super::{
    merge::combine,
    model::{Device, Listing, assemble},
};
use crate::{
    page::Notifier,
    platform::{bluetooth_devices, dongle_devices},
};

/// Where the worker leaves its answer for the UI thread, and whether a read is running.
#[derive(Clone, Default)]
pub(super) struct Reader {
    answer: Arc<Mutex<Option<Listing>>>,
    is_reading: Arc<AtomicBool>,
}

impl Reader {
    /// Starts a read unless one is still running.
    pub(super) fn start(&self, notifier: Notifier) {
        if self.is_reading.swap(true, Ordering::SeqCst) {
            return;
        }
        let reader = self.clone();
        thread::spawn(move || {
            let listing = read_listing();
            if let Ok(mut answer) = reader.answer.lock() {
                *answer = Some(listing);
            }
            reader.is_reading.store(false, Ordering::SeqCst);
            notifier.notify();
        });
    }

    pub(super) fn take(&self) -> Option<Listing> {
        self.answer.lock().ok().and_then(|mut answer| answer.take())
    }
}

fn read_listing() -> Listing {
    let is_radio_on = bluetooth_devices::is_radio_on();
    let bluetooth: Vec<Device> = if is_radio_on == Some(false) {
        Vec::new()
    } else {
        assemble(&bluetooth_devices::paired_devices(), &bluetooth_devices::battery_levels())
    };
    combine(is_radio_on, bluetooth, &dongle_devices::connected_dongle_devices())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "asks Windows about this machine's Bluetooth devices; run on purpose with --nocapture"]
    fn reading_this_machine_gives_a_listing() {
        let started = std::time::Instant::now();
        let paired = bluetooth_devices::paired_devices();
        let batteries = bluetooth_devices::battery_levels();
        println!("radio on: {:?}", bluetooth_devices::is_radio_on());
        println!("paired: {paired:#?}");
        println!("batteries: {batteries:?}");
        println!("listing: {:?}", read_listing());
        println!("took {:?}", started.elapsed());
    }
}
