//! Keeps Windows' own notification banners away while the notch runs (`ShowBanner=0` for every app), so the notch's
//! banner is the only one. Windows still lists each toast, which is how the notch reads it; the saved values go back
//! when the setting is switched off, when the notch exits, and at the next start after a crash.

mod backup;
mod passes;

use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use windows::{
    Win32::{Foundation::HWND, UI::WindowsAndMessaging::FindWindowExW},
    core::w,
};

use crate::{
    log,
    platform::notification_settings::{SettingsWatch, Signal, Wake},
};

pub const SETTING_KEY: &str = "notch.hide_windows_notifications";

static WORKER: Mutex<Option<Worker>> = Mutex::new(None);

struct Worker {
    stop: Arc<Signal>,
    thread: JoinHandle<()>,
}

pub fn is_hidden() -> bool {
    WORKER.lock().is_ok_and(|worker| worker.is_some())
}

/// Off also puts back values a crashed notch left behind, so calling it at start cleans up.
pub fn set_hidden(is_hidden: bool) {
    if is_hidden {
        start();
    } else {
        stop();
        passes::restore_banners();
    }
}

/// At exit: a newer notch that replaced this one owns the saved values and keeps banners off.
pub fn release_at_exit(own_window: HWND) {
    stop();
    if is_another_notch_running(own_window) {
        log::debug("notification banners: a newer notch runs; leaving the banners to it");
        return;
    }
    passes::restore_banners();
}

/// Windows is signing out or shutting down: every notch puts the values back.
pub fn release_at_session_end() {
    stop();
    passes::restore_banners();
}

fn start() {
    let Ok(mut worker) = WORKER.lock() else { return };
    if worker.is_some() {
        return;
    }
    let Some(stop) = Signal::new().map(Arc::new) else {
        log::debug("could not create the notification banner stop signal");
        return;
    };
    let thread_stop = stop.clone();
    match thread::Builder::new().name("notification-banners".into()).spawn(move || watch(&thread_stop)) {
        Ok(thread) => *worker = Some(Worker { stop, thread }),
        Err(error) => log::debug(&format!("could not start the notification banner watch: {error}")),
    }
}

fn stop() {
    let Some(worker) = WORKER.lock().ok().and_then(|mut worker| worker.take()) else { return };
    worker.stop.set();
    if worker.thread.join().is_err() {
        log::debug("could not stop the notification banner watch");
    }
}

/// Wakes only when Windows writes a notification setting, e.g. when an app shows its first toast and gets its key.
fn watch(stop: &Signal) {
    let Some(settings) = SettingsWatch::open() else {
        log::debug("notification banners: cannot watch the settings; hiding the current apps once");
        passes::hide_banners();
        return;
    };
    loop {
        if let Err(error) = settings.arm() {
            log::debug(&format!("could not watch the notification settings: {error:?}"));
        }
        // The pass's own writes wake the next round once; that round finds nothing to change.
        passes::hide_banners();
        if settings.wait(stop) == Wake::Stopped {
            return;
        }
    }
}

fn is_another_notch_running(own_window: HWND) -> bool {
    let mut after = None;
    while let Ok(window) = unsafe { FindWindowExW(None, after, w!("NotchWindow"), None) } {
        if window != own_window {
            return true;
        }
        after = Some(window);
    }
    false
}
