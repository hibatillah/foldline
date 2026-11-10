//! A callback on network interface changes; it exists only while the location is tracked.

use std::{
    ffi::c_void,
    sync::atomic::{AtomicIsize, Ordering},
};

use windows::Win32::{
    Foundation::{HANDLE, HWND, LPARAM, WPARAM},
    NetworkManagement::IpHelper::{
        CancelMibChangeNotify2, MIB_IPINTERFACE_ROW, MIB_NOTIFICATION_TYPE, MibInitialNotification,
        NotifyIpInterfaceChange,
    },
    Networking::WinSock::AF_UNSPEC,
    UI::WindowsAndMessaging::PostMessageW,
};

use super::WM_NETWORK_CHANGED;
use crate::log;

/// The notification handle as an integer; 0 while nothing is registered.
static REGISTRATION: AtomicIsize = AtomicIsize::new(0);

pub fn watch(window: HWND) {
    if REGISTRATION.load(Ordering::SeqCst) != 0 {
        return;
    }
    let mut handle = HANDLE::default();
    let result =
        unsafe { NotifyIpInterfaceChange(AF_UNSPEC, Some(on_change), Some(window.0.cast_const()), false, &mut handle) };
    if result.is_err() {
        log::debug(&format!("could not watch network changes: {result:?}"));
        return;
    }
    REGISTRATION.store(handle.0 as isize, Ordering::SeqCst);
}

pub fn unwatch() {
    let address = REGISTRATION.swap(0, Ordering::SeqCst);
    if address != 0 {
        let result = unsafe { CancelMibChangeNotify2(HANDLE(address as *mut c_void)) };
        if result.is_err() {
            log::debug(&format!("could not stop watching network changes: {result:?}"));
        }
    }
}

/// Runs on a thread of Windows' own; it only wakes the UI thread.
unsafe extern "system" fn on_change(
    window: *const c_void,
    _row: *const MIB_IPINTERFACE_ROW,
    kind: MIB_NOTIFICATION_TYPE,
) {
    if kind != MibInitialNotification {
        let _ = unsafe { PostMessageW(Some(HWND(window.cast_mut())), WM_NETWORK_CHANGED, WPARAM(0), LPARAM(0)) };
    }
}
