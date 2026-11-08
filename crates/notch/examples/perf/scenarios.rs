//! The scenarios after the shelf check: the devices and stats pages, and the opt-in screen-mode run.

use std::{
    path::Path,
    thread::sleep,
    time::{Duration, Instant},
};

use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
        CreateWindowExW, DestroyWindow, DispatchMessageW, GetForegroundWindow, MSG, PM_REMOVE, PeekMessageW,
        SW_RESTORE, SW_SHOWMAXIMIZED, SetForegroundWindow, ShowWindow, TranslateMessage, WINDOW_EX_STYLE,
        WS_OVERLAPPEDWINDOW,
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
