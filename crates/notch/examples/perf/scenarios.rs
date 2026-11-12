//! The scenarios after the shelf check: the devices and stats pages, the pinned notch, and the opt-in screen-mode
//! run.

use std::{
    path::Path,
    thread::sleep,
    time::{Duration, Instant},
};

use windows::Win32::{
    Foundation::{HWND, LPARAM, RECT, WPARAM},
    UI::{
        HiDpi::GetDpiForWindow,
        WindowsAndMessaging::{
            CreateWindowExW, DestroyWindow, DispatchMessageW, GetForegroundWindow, GetWindowRect, MSG, PM_REMOVE,
            PeekMessageW, PostMessageW, SW_RESTORE, SW_SHOWMAXIMIZED, SetForegroundWindow, ShowWindow,
            TranslateMessage, WINDOW_EX_STYLE, WM_LBUTTONDOWN, WM_LBUTTONUP, WS_OVERLAPPEDWINDOW,
        },
    },
};
use windows::core::w;

use super::{
    HOVER_Y, MAX_CPU_MS_PER_CYCLE, MAX_HANDLE_GROWTH, MAX_IDLE_CPU_PERCENT, Report, RunningNotch, move_pointer,
    park_pointer,
};

const DEVICES_CYCLES: u32 = 5;
const STATS_OPEN_SECONDS: u64 = 3;
const SCREEN_MODE_TOGGLES: u32 = 6;
/// Longer than the notch's one-second window reveal delay, so every restore reaches the reveal.
const SCREEN_MODE_HOLD: Duration = Duration::from_millis(1500);
const MAX_CPU_MS_PER_SCREEN_MODE_TOGGLE: f64 = 100.0;

fn hover_cycles(center_x: i32, count: u32) {
    for _ in 0..count {
        move_pointer(center_x, HOVER_Y);
        sleep(Duration::from_millis(1000));
        park_pointer();
        sleep(Duration::from_millis(1000));
    }
}

/// The devices page reads the connected devices when it opens; those reads must cost little and stop afterwards.
pub(super) fn devices_page(executable: &Path, log: &Path, center_x: i32, report: &mut Report) {
    println!("Shelf and devices pages, opening on devices, {DEVICES_CYCLES} open/close cycles:");
    let notch = RunningNotch::start_with_settings(executable, log, Some("shelf,devices"), "navigation.page=devices\n");
    report.require("devices: settles after start-up", notch.wait_until_quiet(), "");
    let before = notch.measure();
    hover_cycles(center_x, DEVICES_CYCLES);
    let after = notch.measure();
    let cpu_per_cycle = ((after.cpu_ms - before.cpu_ms) / f64::from(DEVICES_CYCLES)).round();
    report.check("devices: CPU per open+close cycle", cpu_per_cycle, MAX_CPU_MS_PER_CYCLE, "ms");
    report.require("devices: settles after the animations", notch.wait_until_quiet(), "");
    report.check("devices: CPU while idle after cycles", notch.idle_cpu_percent(), MAX_IDLE_CPU_PERCENT, "%");
    let end = notch.measure();
    report.check("devices: handle growth", f64::from(end.handles) - f64::from(before.handles), MAX_HANDLE_GROWTH, "");
}

/// The stats page samples once a second while it shows; once it closes the timer and the counters must stop.
pub(super) fn stats_page(executable: &Path, log: &Path, center_x: i32, report: &mut Report) {
    println!("Stats page only, open for {STATS_OPEN_SECONDS} s, then closed:");
    let notch = RunningNotch::start(executable, log, Some("stats"));
    report.require("stats: settles after start-up", notch.wait_until_quiet(), "");
    let before = notch.measure();
    move_pointer(center_x, HOVER_Y);
    sleep(Duration::from_secs(STATS_OPEN_SECONDS));
    park_pointer();
    report.require("stats: settles after closing", notch.wait_until_quiet(), "");
    report.check("stats: CPU while idle after closing", notch.idle_cpu_percent(), MAX_IDLE_CPU_PERCENT, "%");
    let end = notch.measure();
    report.check("stats: handle growth", f64::from(end.handles) - f64::from(before.handles), MAX_HANDLE_GROWTH, "");
}

/// The open window's margin past the pill's right edge, as `pill_region` builds it (`host/notch/animation.rs`):
/// 2 px, then the 12 DIP flare and the 20 DIP shadow (10 layers of 2), each rounded up to whole pixels.
const WINDOW_MARGIN_PIXELS: i32 = 2;
const EXPANDED_FLARE: f32 = 12.0;
const SHADOW_SIDE: f32 = 20.0;
/// From the pill's right edge to the pin's center, in DIP (`host/chrome.rs`, `host/pill_row.rs`): the side padding 16,
/// the gear 16, the 4 gap and half the 13 wide pin square.
const PIN_CENTER_FROM_PILL_RIGHT: f32 = 16.0 + 16.0 + 4.0 + 13.0 / 2.0;
/// The header's center line below the top edge: the top padding 10 and half the 16 high header.
const PIN_CENTER_FROM_TOP: f32 = 18.0;
/// How long the open window may keep resizing before the press gives up on it settling.
const WINDOW_SETTLE_TIMEOUT: Duration = Duration::from_secs(3);

fn window_bounds(window: HWND) -> RECT {
    let mut bounds = RECT::default();
    unsafe { GetWindowRect(window, &mut bounds) }.expect("test window bounds");
    bounds
}

/// The window's bounds once two reads 100 ms apart agree, so the pill's open springs have settled.
fn settled_window_bounds(window: HWND) -> RECT {
    let deadline = Instant::now() + WINDOW_SETTLE_TIMEOUT;
    let mut previous = window_bounds(window);
    while Instant::now() < deadline {
        sleep(Duration::from_millis(100));
        let current = window_bounds(window);
        if current == previous {
            return current;
        }
        previous = current;
    }
    previous
}

/// The pin's center relative to the window, whose right edge is `window_width` from its left, at `scale`.
fn pin_center_in_window(window_width: i32, scale: f32) -> (i32, i32) {
    let margin = WINDOW_MARGIN_PIXELS + (EXPANDED_FLARE * scale).ceil() as i32 + (SHADOW_SIDE * scale).ceil() as i32;
    let x = window_width - margin - (PIN_CENTER_FROM_PILL_RIGHT * scale).round() as i32;
    (x, (PIN_CENTER_FROM_TOP * scale).round() as i32)
}

/// Waits for the open pill to settle, rests the pointer on the pin, then presses it in window coordinates (the notch
/// reads clicks relative to its window). Resting there first keeps the notch open and the press on a still target.
fn press_pin(window: HWND) {
    let scale = unsafe { GetDpiForWindow(window) } as f32 / 96.0;
    let bounds = settled_window_bounds(window);
    let (x, y) = pin_center_in_window(bounds.right - bounds.left, scale);
    move_pointer(bounds.left + x, bounds.top + y);
    sleep(Duration::from_millis(300));
    let position = LPARAM(((y as u32 as isize) << 16) | (x as u32 as isize & 0xffff));
    for message in [WM_LBUTTONDOWN, WM_LBUTTONUP] {
        unsafe { PostMessageW(Some(window), message, WPARAM(0), position) }.expect("press the pin");
    }
}

/// A pinned notch stays open with the pointer away; it must arm no timer and use no CPU while it idles. The stats page
/// is switched on but not shown, so its sampler must stay off while the notch is pinned on the shelf.
pub(super) fn pinned_idle(executable: &Path, log: &Path, center_x: i32, report: &mut Report) {
    println!("Shelf and stats pages, pinned open on the shelf, pointer away:");
    let notch = RunningNotch::start_with_settings(executable, log, Some("shelf,stats"), "navigation.page=shelf\n");
    report.require("pinned: settles after start-up", notch.wait_until_quiet(), "");
    // Read from before the pointer opens it, so the `open=true` of that opening is among the lines checked.
    let log_length_before = notch.log_text().len();
    move_pointer(center_x, HOVER_Y);
    sleep(Duration::from_millis(1000));
    press_pin(notch.window());
    sleep(Duration::from_millis(500));
    park_pointer();
    sleep(Duration::from_millis(1000));
    let log_text = notch.log_text();
    let new_lines = log_text.get(log_length_before..).unwrap_or_default();
    let is_pinned = new_lines.contains("pinned=true");
    report.require("pinned: the press on the pin pinned it", is_pinned, "");
    if !is_pinned {
        println!("    log:\n{new_lines}");
        return;
    }
    let opened = new_lines.contains("open=true") && !new_lines.contains("open=false");
    report.require("pinned: the notch is open and held open", opened, "");
    let before = notch.measure();
    report.require("pinned: settles while pinned", notch.wait_until_quiet(), "");
    report.check("pinned: CPU while idle", notch.idle_cpu_percent(), MAX_IDLE_CPU_PERCENT, "%");
    let end = notch.measure();
    report.check("pinned: handle growth", f64::from(end.handles) - f64::from(before.handles), MAX_HANDLE_GROWTH, "");
    let log_text = notch.log_text();
    let stayed_open = !log_text.get(log_length_before..).unwrap_or_default().contains("open=false");
    report.require("pinned: stays open with the pointer away", stayed_open, "");
}

/// Handles the messages of the test window for `duration`, so it answers while the notch reacts to it.
fn pump_for(duration: Duration) {
    let deadline = Instant::now() + duration;
    let mut message = MSG::default();
    while Instant::now() < deadline {
        while unsafe { PeekMessageW(&mut message, None, 0, 0, PM_REMOVE) }.as_bool() {
            unsafe {
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
        sleep(Duration::from_millis(15));
    }
}

/// A visible window of this process; destroyed when it goes out of scope.
struct TestWindow(HWND);

impl Drop for TestWindow {
    fn drop(&mut self) {
        let _ = unsafe { DestroyWindow(self.0) };
        pump_for(Duration::from_millis(200));
    }
}

/// Opt-in: opens a real window and maximizes and restores it, so the notch hides and comes back each time.
pub(super) fn screen_mode(executable: &Path, log: &Path, report: &mut Report) {
    println!("Screen mode, {SCREEN_MODE_TOGGLES} maximize/restore toggles of a test window:");
    // The default showing, so the maximized test window hides the notch; a later line wins in the settings file.
    let notch = RunningNotch::start_with_settings(executable, log, Some("shelf"), "notch.maximized=lyrics\n");
    report.require("screen mode: settles after start-up", notch.wait_until_quiet(), "");
    let created = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("STATIC"),
            w!("notch perf screen mode"),
            WS_OVERLAPPEDWINDOW,
            100,
            100,
            640,
            480,
            None,
            None,
            None,
            None,
        )
    };
    let Ok(window) = created.map(TestWindow) else {
        println!("Screen mode NOT RUN: could not create the test window.");
        return;
    };
    unsafe {
        let _ = ShowWindow(window.0, SW_RESTORE);
        let _ = SetForegroundWindow(window.0);
    }
    pump_for(SCREEN_MODE_HOLD);
    if unsafe { GetForegroundWindow() } != window.0 {
        println!("Screen mode NOT RUN: Windows kept another window in front of the test window.");
        return;
    }
    let before = notch.measure();
    for _ in 0..SCREEN_MODE_TOGGLES {
        unsafe {
            let _ = ShowWindow(window.0, SW_SHOWMAXIMIZED);
        }
        pump_for(SCREEN_MODE_HOLD);
        unsafe {
            let _ = ShowWindow(window.0, SW_RESTORE);
        }
        pump_for(SCREEN_MODE_HOLD);
    }
    let after = notch.measure();
    drop(window);
    let cpu_per_toggle = ((after.cpu_ms - before.cpu_ms) / f64::from(SCREEN_MODE_TOGGLES)).round();
    report.check("screen mode: CPU per toggle", cpu_per_toggle, MAX_CPU_MS_PER_SCREEN_MODE_TOGGLE, "ms");
    report.require("screen mode: settles after the toggles", notch.wait_until_quiet(), "");
    report.check("screen mode: CPU while idle after", notch.idle_cpu_percent(), MAX_IDLE_CPU_PERCENT, "%");
    let end = notch.measure();
    let handle_growth = f64::from(end.handles) - f64::from(before.handles);
    report.check("screen mode: handle growth", handle_growth, MAX_HANDLE_GROWTH, "");
}
