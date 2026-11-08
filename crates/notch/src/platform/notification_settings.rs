//! The per-app notification settings Windows keeps in the user's registry, one `ShowBanner` value at a time.

use std::time::Duration;

use windows::{
    Win32::{
        Foundation::{
            CloseHandle, ERROR_NO_MORE_ITEMS, ERROR_SUCCESS, HANDLE, WAIT_ABANDONED, WAIT_EVENT, WAIT_OBJECT_0,
            WIN32_ERROR,
        },
        System::{
            Registry::{
                HKEY, HKEY_CURRENT_USER, KEY_NOTIFY, KEY_READ, REG_DWORD, REG_NOTIFY_CHANGE_LAST_SET,
                REG_NOTIFY_CHANGE_NAME, REG_SAM_FLAGS, RRF_RT_REG_DWORD, RegCloseKey, RegDeleteKeyValueW,
                RegEnumKeyExW, RegGetValueW, RegNotifyChangeKeyValue, RegOpenKeyExW, RegSetKeyValueW,
            },
            Threading::{
                CreateEventW, CreateMutexW, INFINITE, ReleaseMutex, SetEvent, WaitForMultipleObjects,
                WaitForSingleObject,
            },
        },
    },
    core::{HSTRING, PCWSTR, PWSTR, w},
};

const SETTINGS_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Notifications\Settings";
const SHOW_BANNER: PCWSTR = w!("ShowBanner");
/// Registry key names are at most 255 characters.
const MAX_KEY_NAME_LENGTH: usize = 256;
/// Another notch holding the lock longer than this has hung; going on beats blocking the notch's exit.
const LOCK_TIMEOUT: Duration = Duration::from_secs(5);

/// The app ids Windows keeps notification settings for, one subkey each.
pub fn app_ids() -> Vec<String> {
    let Some(key) = OpenKey::open(KEY_READ) else { return Vec::new() };
    let mut ids = Vec::new();
    let mut name = [0u16; MAX_KEY_NAME_LENGTH];
    for index in 0.. {
        let mut length = name.len() as u32;
        let status =
            unsafe { RegEnumKeyExW(key.0, index, Some(PWSTR(name.as_mut_ptr())), &mut length, None, None, None, None) };
        if status == ERROR_NO_MORE_ITEMS {
            break;
        }
        if status == ERROR_SUCCESS {
            ids.push(String::from_utf16_lossy(&name[..length as usize]));
        }
    }
    ids
}

/// The app's `ShowBanner` value; `None` when the value or the app's key is missing.
pub fn show_banner(app_id: &str) -> Option<u32> {
    let mut value = 0u32;
    let mut size = size_of::<u32>() as u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            &app_key(app_id),
            SHOW_BANNER,
            RRF_RT_REG_DWORD,
            None,
            Some((&mut value as *mut u32).cast()),
            Some(&mut size),
        )
    };
    (status == ERROR_SUCCESS).then_some(value)
}

/// Writes the app's `ShowBanner` value, or deletes it for `None`.
pub fn set_show_banner(app_id: &str, value: Option<u32>) -> Result<(), WIN32_ERROR> {
    let key = app_key(app_id);
    let status = match value {
        Some(value) => unsafe {
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                &key,
                SHOW_BANNER,
                REG_DWORD.0,
                Some((&value as *const u32).cast()),
                size_of::<u32>() as u32,
            )
        },
        None => unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, &key, SHOW_BANNER) },
    };
    if status == ERROR_SUCCESS { Ok(()) } else { Err(status) }
}

fn app_key(app_id: &str) -> HSTRING {
    HSTRING::from(format!(r"{SETTINGS_PATH}\{app_id}"))
}

struct OpenKey(HKEY);

impl OpenKey {
    fn open(access: REG_SAM_FLAGS) -> Option<Self> {
        let mut key = HKEY::default();
        let status = unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, &HSTRING::from(SETTINGS_PATH), None, access, &mut key) };
        (status == ERROR_SUCCESS).then_some(Self(key))
    }
}

impl Drop for OpenKey {
    fn drop(&mut self) {
        let _ = unsafe { RegCloseKey(self.0) };
    }
}

/// A Windows event another thread sets to wake a waiting one.
pub struct Signal(isize);

impl Signal {
    pub fn new() -> Option<Self> {
        let event = unsafe { CreateEventW(None, false, false, PCWSTR::null()) }.ok()?;
        Some(Self(event.0 as isize))
    }

    pub fn set(&self) {
        if let Err(error) = unsafe { SetEvent(self.handle()) } {
            crate::log::debug(&format!("could not set a signal: {error}"));
        }
    }

    fn handle(&self) -> HANDLE {
        HANDLE(self.0 as *mut _)
    }
}

impl Drop for Signal {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.handle()) };
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wake {
    Changed,
    Stopped,
}

/// Wakes on any change under the notification settings: a new app's key, or a value written in any app's key.
pub struct SettingsWatch {
    key: OpenKey,
    changed: Signal,
}

impl SettingsWatch {
    pub fn open() -> Option<Self> {
        Some(Self { key: OpenKey::open(KEY_NOTIFY)?, changed: Signal::new()? })
    }

    /// Asks Windows for one signal at the next change; arm before reading so a change made meanwhile is not lost.
    pub fn arm(&self) -> Result<(), WIN32_ERROR> {
        let status = unsafe {
            RegNotifyChangeKeyValue(
                self.key.0,
                true,
                REG_NOTIFY_CHANGE_NAME | REG_NOTIFY_CHANGE_LAST_SET,
                Some(self.changed.handle()),
                true,
            )
        };
        if status == ERROR_SUCCESS { Ok(()) } else { Err(status) }
    }

    pub fn wait(&self, stop: &Signal) -> Wake {
        let wake = unsafe { WaitForMultipleObjects(&[stop.handle(), self.changed.handle()], false, INFINITE) };
        // A failed wait counts as a stop, so the watch can never spin.
        if wake == WAIT_EVENT(WAIT_OBJECT_0.0 + 1) { Wake::Changed } else { Wake::Stopped }
    }
}

/// Held while a notch reads or changes the banner values, so two running copies never interleave.
pub struct ExclusiveAccess(HANDLE);

impl ExclusiveAccess {
    pub fn acquire() -> Option<Self> {
        let mutex = unsafe { CreateMutexW(None, false, w!(r"Local\NotchNotificationBanners")) }.ok()?;
        let wait = unsafe { WaitForSingleObject(mutex, LOCK_TIMEOUT.as_millis() as u32) };
        // An abandoned lock belonged to a notch that crashed; it is ours now.
        if wait == WAIT_OBJECT_0 || wait == WAIT_ABANDONED {
            return Some(Self(mutex));
        }
        let _ = unsafe { CloseHandle(mutex) };
        None
    }
}

impl Drop for ExclusiveAccess {
    fn drop(&mut self) {
        let _ = unsafe { ReleaseMutex(self.0) };
        let _ = unsafe { CloseHandle(self.0) };
    }
}
