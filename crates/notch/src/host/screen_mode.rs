//! Follow foreground changes and only that window thread's moves/resizes; no polling.

mod classify;
mod location_hook;

use std::sync::{
    Mutex, OnceLock, PoisonError,
    atomic::{AtomicIsize, AtomicU8, Ordering},
};

use windows::Win32::{
    Foundation::{HWND, POINT},
    Graphics::Gdi::{
        GetMonitorInfoW, HMONITOR, MONITOR_DEFAULTTONEAREST, MONITOR_DEFAULTTOPRIMARY, MONITORINFO, MonitorFromPoint,
        MonitorFromWindow,
    },
    UI::{
        Accessibility::{HWINEVENTHOOK, SetWinEventHook},
        WindowsAndMessaging::{
            EVENT_OBJECT_LOCATIONCHANGE, EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MINIMIZEEND, EVENT_SYSTEM_MINIMIZESTART,
            GWL_STYLE, GetForegroundWindow, GetWindowLongW, GetWindowRect, GetWindowThreadProcessId, IsIconic,
            IsWindowVisible, IsZoomed, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS, WS_CAPTION,
        },
    },
};

use super::monitor::MonitorId;
use crate::platform::window_info::{self, class_name};
pub(super) use classify::Bounds;
use classify::{CaptureHost, Facts, capture_host, classify};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ScreenMode {
    Normal,
    Maximized,
    Fullscreen,
    /// A screenshot overlay is in front; the notch hides and ignores the pointer whatever the settings say.
    Capture,
}

const IGNORED_CLASSES: [&str; 6] =
    ["Progman", "WorkerW", "Shell_TrayWnd", "Shell_SecondaryTrayWnd", "NotchWindow", "NotchDropTarget"];

/// Whether a window class is a user window: not the desktop, a taskbar or one of the notch's own windows.
pub(super) fn is_user_class(class: &str) -> bool {
    !IGNORED_CLASSES.contains(&class)
}

pub(super) fn is_user_window(window: HWND) -> bool {
    is_user_class(&class_name(window))
}

static ON_CHANGE: OnceLock<fn(ScreenMode)> = OnceLock::new();
static LAST_MODE: AtomicU8 = AtomicU8::new(0);
/// The monitor the notch is on, which the window in front is judged against; 0 means the primary monitor.
static JUDGED_MONITOR: AtomicIsize = AtomicIsize::new(0);
/// The foreground window's process id and image file name; cleared on every foreground change.
static FOREGROUND_PROCESS: Mutex<Option<(u32, String)>> = Mutex::new(None);

/// The image file name (`SnippingTool.exe`) of a process, or an empty string when it cannot be read.
fn process_file_name(process_id: u32) -> String {
    let path = window_info::process_image_path(process_id).unwrap_or_default();
    path.rsplit('\\').next().unwrap_or_default().to_string()
}

/// Opens the process once per foreground window; moves and resizes of that window reuse the name.
fn foreground_process_name(window: HWND) -> String {
    let mut process_id = 0;
    unsafe { GetWindowThreadProcessId(window, Some(&mut process_id)) };
    let mut cached = FOREGROUND_PROCESS.lock().unwrap_or_else(PoisonError::into_inner);
    match cached.as_ref() {
        Some((cached_id, name)) if *cached_id == process_id => name.clone(),
        _ => {
            let name = process_file_name(process_id);
            *cached = Some((process_id, name.clone()));
            name
        }
    }
}

/// The mode of the window in front, judged against `monitor` (`None`: the primary monitor).
fn read(monitor: Option<MonitorId>) -> ScreenMode {
    let window = unsafe { GetForegroundWindow() };
    if window.0.is_null() {
        return ScreenMode::Normal;
    }
    let mut info = MONITORINFO { cbSize: size_of::<MONITORINFO>() as u32, ..Default::default() };
    let mut judged = monitor.map_or(HMONITOR::default(), |id| HMONITOR(id.0 as *mut _));
    // A monitor unplugged since it was set no longer answers; the primary one stands in.
    if judged.0.is_null() || !unsafe { GetMonitorInfoW(judged, &mut info) }.as_bool() {
        judged = unsafe { MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY) };
        if !unsafe { GetMonitorInfoW(judged, &mut info) }.as_bool() {
            return ScreenMode::Normal;
        }
    }
    let window_monitor = unsafe { MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST) };
    let mut rect = Default::default();
    if unsafe { GetWindowRect(window, &mut rect) }.is_err() {
        return ScreenMode::Normal;
    }
    let class = class_name(window);
    let is_visible = unsafe { IsWindowVisible(window) }.as_bool();
    let capture_host = if is_visible { capture_host(&foreground_process_name(window)) } else { CaptureHost::None };
    let style = unsafe { GetWindowLongW(window, GWL_STYLE) } as u32;
    let facts = Facts {
        window: Bounds { left: rect.left, top: rect.top, right: rect.right, bottom: rect.bottom },
        monitor: Bounds {
            left: info.rcMonitor.left,
            top: info.rcMonitor.top,
            right: info.rcMonitor.right,
            bottom: info.rcMonitor.bottom,
        },
        is_zoomed: unsafe { IsZoomed(window) }.as_bool(),
        has_caption: (style & WS_CAPTION.0) == WS_CAPTION.0,
        capture_host,
        is_ignored: window_monitor != judged
            || !is_visible
            || unsafe { IsIconic(window) }.as_bool()
            || !is_user_class(&class),
    };
    classify(&facts)
}

fn judged_monitor() -> Option<MonitorId> {
    let monitor = JUDGED_MONITOR.load(Ordering::Relaxed);
    (monitor != 0).then_some(MonitorId(monitor))
}

/// Judges the window in front against `monitor` from now on and returns its mode now, without calling `on_change`.
pub(super) fn judge_against(monitor: Option<MonitorId>) -> ScreenMode {
    JUDGED_MONITOR.store(monitor.map_or(0, |id| id.0), Ordering::Relaxed);
    let mode = read(monitor);
    LAST_MODE.store(mode as u8, Ordering::Relaxed);
    mode
}

/// Sets the monitor to judge against before the hook starts.
pub(super) fn set_judged_monitor(monitor: Option<MonitorId>) {
    JUDGED_MONITOR.store(monitor.map_or(0, |id| id.0), Ordering::Relaxed);
}

fn refresh(notify_when_unchanged: bool) {
    let mode = read(judged_monitor());
    if notify_when_unchanged {
        let window = unsafe { GetForegroundWindow() };
        let process = if window.0.is_null() { String::new() } else { foreground_process_name(window) };
        crate::log::debug(&format!("foreground class={} process={process} mode={mode:?}", class_name(window)));
    }
    if (LAST_MODE.swap(mode as u8, Ordering::Relaxed) != mode as u8 || notify_when_unchanged)
        && let Some(on_change) = ON_CHANGE.get()
    {
        on_change(mode);
    }
}

pub(super) fn refresh_foreground() {
    *FOREGROUND_PROCESS.lock().unwrap_or_else(PoisonError::into_inner) = None;
    location_hook::follow_foreground_thread();
    refresh(true);
}

unsafe extern "system" fn on_event(
    _hook: HWINEVENTHOOK,
    event: u32,
    window: HWND,
    object: i32,
    _child: i32,
    _thread: u32,
    event_time: u32,
) {
    match event {
        EVENT_SYSTEM_FOREGROUND => {
            refresh_foreground();
        }
        EVENT_OBJECT_LOCATIONCHANGE => location_hook::location_changed(window, object, event_time),
        _ => refresh(false),
    }
}

/// Called on the notch thread for mode changes and foreground switches. Register once.
pub(super) fn start(on_change: fn(ScreenMode)) {
    let _ = ON_CHANGE.set(on_change);
    // Every process: a foreground change has no one owner; these fire on a switch or minimize, never a mouse move.
    for event in [EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MINIMIZESTART, EVENT_SYSTEM_MINIMIZEEND] {
        let _ = unsafe {
            SetWinEventHook(event, event, None, Some(on_event), 0, 0, WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS)
        };
    }
    refresh_foreground();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Check classification against a real foreground window, beyond the pure geometry tests.
    fn mode_seen_with_window(style: windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE, show: i32) -> ScreenMode {
        use windows::{
            Win32::{
                Foundation::{LPARAM, WPARAM},
                UI::{
                    Input::KeyboardAndMouse::{KEYEVENTF_KEYUP, VK_MENU, keybd_event},
                    WindowsAndMessaging::{
                        CreateWindowExW, DestroyWindow, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN, SW_SHOW,
                        SendMessageW, SetForegroundWindow, ShowWindow, WINDOW_EX_STYLE, WM_NULL,
                    },
                },
            },
            core::w,
        };
        unsafe {
            let (width, height) = (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN));
            let window = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("STATIC"),
                w!("screen mode test"),
                style,
                0,
                0,
                width,
                height,
                None,
                None,
                None,
                None,
            )
            .expect("a test window");
            let _ = ShowWindow(window, windows::Win32::UI::WindowsAndMessaging::SHOW_WINDOW_CMD(show));
            // Windows only lets a process take the front when it has just had input; a tap of Alt counts.
            keybd_event(VK_MENU.0 as u8, 0, Default::default(), 0);
            keybd_event(VK_MENU.0 as u8, 0, KEYEVENTF_KEYUP, 0);
            let _ = SetForegroundWindow(window);
            let _ = SendMessageW(window, WM_NULL, Some(WPARAM(0)), Some(LPARAM(0)));
            std::thread::sleep(std::time::Duration::from_millis(400));
            let mode = read(None);
            let _ = ShowWindow(window, SW_SHOW);
            let _ = DestroyWindow(window);
            mode
        }
    }

    #[test]
    #[ignore = "opens a window over the whole screen for a moment; run on purpose"]
    fn a_real_maximized_window_and_a_real_full_screen_window_are_told_apart() {
        use windows::Win32::UI::WindowsAndMessaging::{
            SW_MAXIMIZE, SW_SHOW, WS_OVERLAPPEDWINDOW, WS_POPUP, WS_VISIBLE,
        };
        let maximized = mode_seen_with_window(WS_OVERLAPPEDWINDOW | WS_VISIBLE, SW_MAXIMIZE.0);
        let fullscreen = mode_seen_with_window(WS_POPUP | WS_VISIBLE, SW_SHOW.0);
        println!("maximized -> {maximized:?}, full screen -> {fullscreen:?}");
        assert_eq!((maximized, fullscreen), (ScreenMode::Maximized, ScreenMode::Fullscreen));
    }
}
