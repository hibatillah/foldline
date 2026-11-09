//! Hides Windows' own volume flyout each time Explorer shows it, so the notch's volume activity is the only one.
//! Nothing about Explorer's window is changed for good: when the notch exits or crashes, the hook goes with it.

mod hook_thread;
mod identity;

use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU32, Ordering};

use windows::{
    Win32::{
        Foundation::{HWND, RECT},
        Graphics::Gdi::{GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow},
        UI::{
            Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent},
            WindowsAndMessaging::{
                EVENT_OBJECT_SHOW, FindWindowExW, GWL_EXSTYLE, GetShellWindow, GetWindowLongW, GetWindowRect,
                GetWindowTextLengthW, GetWindowThreadProcessId, IsWindowVisible, OBJID_WINDOW, SW_HIDE, ShowWindow,
                WINEVENT_OUTOFCONTEXT,
            },
        },
    },
    core::{HSTRING, PCWSTR},
};

use super::screen_mode::Bounds;
use crate::{log, platform::window_info};
use hook_thread::Command;
use identity::{FLYOUT_CLASS, WindowFacts, is_volume_flyout};

pub(super) const SETTING_KEY: &str = "notch.hide_windows_volume_popup";

/// On until the user switches it off.
pub(super) fn wants_hidden(settings: &store::Settings) -> bool {
    settings.get_bool(SETTING_KEY, true)
}

static IS_ON: AtomicBool = AtomicBool::new(false);
static SHOW_HOOK: AtomicIsize = AtomicIsize::new(0);
static FLYOUT_WINDOW: AtomicIsize = AtomicIsize::new(0);
/// The thread the hook listens to; 0 while it listens to all of Explorer, looking for the flyout.
static HOOKED_THREAD: AtomicU32 = AtomicU32::new(0);

pub(super) fn is_hidden() -> bool {
    IS_ON.load(Ordering::Relaxed)
}

pub(super) fn set_hidden(is_hidden: bool) {
    if IS_ON.swap(is_hidden, Ordering::Relaxed) == is_hidden {
        return;
    }
    hook_thread::send(if is_hidden { Command::Follow } else { Command::StopHiding });
}

/// Explorer restarted (`TaskbarCreated`): its old flyout and thread are gone.
pub(super) fn explorer_restarted() {
    if is_hidden() {
        hook_thread::send(Command::FollowNewExplorer);
    }
}

// The functions below run on the hook thread, which also owns the hook: UnhookWinEvent must come from its thread.

fn stop_hiding() {
    unhook();
    FLYOUT_WINDOW.store(0, Ordering::Relaxed);
}

fn follow_new_explorer() {
    FLYOUT_WINDOW.store(0, Ordering::Relaxed);
    follow_explorer();
}

fn follow_explorer() {
    unhook();
    let shell = unsafe { GetShellWindow() };
    if shell.is_invalid() {
        log::debug("volume flyout: Explorer is not running; waiting for TaskbarCreated");
        return;
    }
    let mut explorer_process = 0;
    unsafe { GetWindowThreadProcessId(shell, Some(&mut explorer_process)) };
    match find_flyout(explorer_process) {
        Some(flyout) => adopt(flyout),
        // Explorer creates the flyout at the first volume change; until then every Explorer show is checked.
        None => hook(explorer_process, 0),
    }
}

fn find_flyout(explorer_process: u32) -> Option<HWND> {
    let class = HSTRING::from(FLYOUT_CLASS);
    let mut after = None;
    while let Ok(window) = unsafe { FindWindowExW(None, after, &class, PCWSTR::null()) } {
        let mut process = 0;
        unsafe { GetWindowThreadProcessId(window, Some(&mut process)) };
        if process == explorer_process && is_flyout(window) {
            return Some(window);
        }
        after = Some(window);
    }
    None
}

/// Listens only to the flyout's own thread, which shows nothing else, and hides the flyout if it is up now.
fn adopt(flyout: HWND) {
    FLYOUT_WINDOW.store(flyout.0 as isize, Ordering::Relaxed);
    let mut process = 0;
    let thread = unsafe { GetWindowThreadProcessId(flyout, Some(&mut process)) };
    if thread != HOOKED_THREAD.load(Ordering::Relaxed) || SHOW_HOOK.load(Ordering::Relaxed) == 0 {
        unhook();
        hook(process, thread);
    }
    log::debug(&format!("volume flyout: hiding window {:?} on thread {thread}", flyout.0));
    if unsafe { IsWindowVisible(flyout) }.as_bool() {
        hide(flyout);
    }
}

fn hook(process: u32, thread: u32) {
    let hook = unsafe {
        SetWinEventHook(
            EVENT_OBJECT_SHOW,
            EVENT_OBJECT_SHOW,
            None,
            Some(on_show),
            process,
            thread,
            WINEVENT_OUTOFCONTEXT,
        )
    };
    if hook.is_invalid() {
        log::debug("could not hook the volume flyout");
    }
    SHOW_HOOK.store(hook.0 as isize, Ordering::Relaxed);
    HOOKED_THREAD.store(thread, Ordering::Relaxed);
}

fn unhook() {
    let previous = SHOW_HOOK.swap(0, Ordering::Relaxed);
    HOOKED_THREAD.store(0, Ordering::Relaxed);
    if previous != 0 && !unsafe { UnhookWinEvent(HWINEVENTHOOK(previous as *mut _)) }.as_bool() {
        log::debug("could not unhook the volume flyout");
    }
}

fn hide(flyout: HWND) {
    // Explorer shows the flyout again at the next volume change; hiding it is never remembered.
    let _ = unsafe { ShowWindow(flyout, SW_HIDE) };
}

unsafe extern "system" fn on_show(
    _hook: HWINEVENTHOOK,
    _event: u32,
    window: HWND,
    object: i32,
    _child: i32,
    _thread: u32,
    _event_time: u32,
) {
    if object != OBJID_WINDOW.0 || !is_hidden() {
        return;
    }
    if window.0 as isize == FLYOUT_WINDOW.load(Ordering::Relaxed) {
        hide(window);
    } else if is_flyout(window) {
        // The first show after start, or a flyout Explorer created again.
        adopt(window);
    }
}

fn is_flyout(window: HWND) -> bool {
    let class_name = window_info::class_name(window);
    if class_name != FLYOUT_CLASS {
        return false;
    }
    let (Some(window_bounds), Some(monitor)) = (window_bounds(window), monitor_bounds(window)) else {
        return false;
    };
    is_volume_flyout(&WindowFacts {
        class_name: &class_name,
        has_title: unsafe { GetWindowTextLengthW(window) } > 0,
        extended_style: unsafe { GetWindowLongW(window, GWL_EXSTYLE) } as u32,
        window: window_bounds,
        monitor,
    })
}

fn window_bounds(window: HWND) -> Option<Bounds> {
    let mut rect = RECT::default();
    unsafe { GetWindowRect(window, &mut rect) }.ok()?;
    Some(Bounds { left: rect.left, top: rect.top, right: rect.right, bottom: rect.bottom })
}

fn monitor_bounds(window: HWND) -> Option<Bounds> {
    let monitor = unsafe { MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST) };
    let mut info = MONITORINFO { cbSize: size_of::<MONITORINFO>() as u32, ..Default::default() };
    if !unsafe { GetMonitorInfoW(monitor, &mut info) }.as_bool() {
        return None;
    }
    let area = info.rcMonitor;
    Some(Bounds { left: area.left, top: area.top, right: area.right, bottom: area.bottom })
}

#[cfg(test)]
mod default_tests {
    use super::*;

    fn settings_with(name: &str, text: Option<&str>) -> store::Settings {
        let path = std::env::temp_dir().join(format!("notch-{name}-{}.txt", std::process::id()));
        let _ = std::fs::remove_file(&path);
        if let Some(text) = text {
            std::fs::write(&path, text).unwrap();
        }
        let settings = store::Settings::at(Some(path.clone()));
        let _ = std::fs::remove_file(&path);
        settings
    }

    #[test]
    fn an_unset_key_hides_the_popup() {
        assert!(wants_hidden(&settings_with("volume_flyout-unset", None)));
    }

    #[test]
    fn an_explicit_off_shows_the_popup() {
        assert!(!wants_hidden(&settings_with("volume_flyout-off", Some("notch.hide_windows_volume_popup=off\n"))));
    }

    #[test]
    fn an_explicit_false_shows_the_popup() {
        assert!(!wants_hidden(&settings_with("volume_flyout-false", Some("notch.hide_windows_volume_popup=false\n"))));
    }
}
