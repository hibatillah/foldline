//! Run through scripts/perf.sh; this check moves the pointer and enforces the limits below.

use std::{
    fs,
    os::windows::io::AsRawHandle,
    path::{Path, PathBuf},
    process::{Child, Command, ExitCode},
    thread::sleep,
    time::{Duration, Instant},
};

use windows::Win32::{
    Foundation::{FILETIME, HANDLE, LPARAM, WPARAM},
    System::{
        ProcessStatus::K32QueryWorkingSet,
        Threading::{GetProcessHandleCount, GetProcessTimes},
    },
    UI::{
        HiDpi::GetDpiForWindow,
        WindowsAndMessaging::{
            FindWindowW, GetSystemMetrics, GetWindowRect, GetWindowThreadProcessId, PostMessageW, SM_CXSCREEN,
            SetCursorPos, SetProcessDPIAware, WM_MOUSEHWHEEL,
        },
    },
};
use windows::{
    Media::Control::{
        GlobalSystemMediaTransportControlsSessionManager, GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    },
    Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize},
    core::w,
};

#[path = "perf/scenarios.rs"]
mod scenarios;

fn media_is_playing() -> windows::core::Result<bool> {
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.join()?;
    if manager.GetSessions()?.Size()? == 0 {
        return Ok(false);
    }
    let status = manager.GetCurrentSession()?.GetPlaybackInfo()?.PlaybackStatus()?;
    Ok(status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing)
}

const IDLE_SECONDS: u64 = 20;
const CYCLES: u32 = 10;
const MAX_IDLE_CPU_PERCENT: f64 = 0.1;
const MAX_CPU_MS_PER_CYCLE: f64 = 400.0;
const MAX_MEMORY_MB: f64 = 30.0;
const MAX_MEMORY_GROWTH_MB: f64 = 3.0;
const MAX_HANDLE_GROWTH: f64 = 20.0;
const SETTLE_SECONDS: u64 = 20;
const PARKED_POINTER: (i32, i32) = (50, 600);
const PAGE_SIZE_BYTES: usize = 4096;
const BYTES_PER_MEGABYTE: f64 = 1024.0 * 1024.0;
const HOVER_Y: i32 = 1;
/// A maximized or full-screen window in front leaves the pill up, so app switches by others do not animate it.
const ISOLATED_SETTINGS: &str = "notch.hide_when_idle=off\nnotch.fullscreen=music\nnotch.maximized=music\n";
/// Idle samples taken at most when outside events keep disturbing them.
const IDLE_ATTEMPTS: u32 = 3;
/// What the notch logs when something outside it made it work: Windows wrote its notification database (the listener
/// read it, or found or showed a toast), or an activity or page changed what the pill shows (`host/notch/layout.rs`).
const OUTSIDE_EVENT_LOGS: [&str; 4] = ["notifications: read ", "notifications: card for", " new by id ", "pill shows "];

/// The first line of this part of the notch's debug log that tells of an outside event, without its timestamp.
fn outside_event(log_text: &str) -> Option<&str> {
    log_text
        .lines()
        .find(|line| OUTSIDE_EVENT_LOGS.iter().any(|marker| line.contains(marker)))
        .map(|line| line.split_once(" ms  ").map_or(line, |(_, message)| message))
}

struct Measurement {
    private_working_set_mb: f64,
    handles: u32,
    cpu_ms: f64,
}

/// The notch process and its debug log; the process is stopped when this goes out of scope, even after a failed check.
struct RunningNotch {
    process: Child,
    log: PathBuf,
}

impl RunningNotch {
    /// Runs on its own settings so a maximized or full-screen window in front cannot hide the notch mid-check.
    fn start(executable: &Path, log: &Path, pages: Option<&str>) -> Self {
        Self::start_with_settings(executable, log, pages, "")
    }

    /// Like `start`, with `extra_settings` (whole `key=value` lines) appended to the isolated settings file.
    fn start_with_settings(executable: &Path, log: &Path, pages: Option<&str>, extra_settings: &str) -> Self {
        let profile = log.parent().expect("log directory").join("perf-default");
        let settings = profile.join("notch");
        fs::create_dir_all(&settings).expect("create isolated test profile");
        let contents = format!("{ISOLATED_SETTINGS}{extra_settings}");
        fs::write(settings.join("settings.txt"), contents).expect("write isolated test settings");
        Self::start_with_profile(executable, log, pages, Some(&profile))
    }

    fn start_with_profile(executable: &Path, log: &Path, pages: Option<&str>, profile: Option<&Path>) -> Self {
        park_pointer();
        let _ = fs::remove_file(log);
        let mut command = Command::new(executable);
        command.env("NOTCH_DEBUG", log);
        if let Some(profile) = profile {
            command.env("APPDATA", profile).env_remove("NOTCH_LYRICS");
        }
        match pages {
            Some(pages) => command.env("NOTCH_PAGES", pages),
            None => command.env_remove("NOTCH_PAGES"),
        };
        let process = command.spawn().expect("the notch executable should start");
        let notch = Self { process, log: log.to_path_buf() };
        sleep(Duration::from_secs(3));
        notch
    }

    fn measure(&self) -> Measurement {
        let process = HANDLE(self.process.as_raw_handle());
        let (mut creation, mut exit, mut kernel, mut user) =
            (FILETIME::default(), FILETIME::default(), FILETIME::default(), FILETIME::default());
        let mut handles = 0;
        unsafe {
            let _ = GetProcessTimes(process, &mut creation, &mut exit, &mut kernel, &mut user);
            let _ = GetProcessHandleCount(process, &mut handles);
        }
        // File times count 100 ns ticks.
        let cpu_ms = (ticks(kernel) + ticks(user)) as f64 / 10_000.0;
        Measurement { private_working_set_mb: private_working_set_mb(process), handles, cpu_ms }
    }

    fn wait_until_quiet(&self) -> bool {
        let deadline = Instant::now() + Duration::from_secs(SETTLE_SECONDS);
        let mut last = self.measure().cpu_ms;
        while Instant::now() < deadline {
            sleep(Duration::from_secs(2));
            let now = self.measure().cpu_ms;
            if now - last < 1.0 {
                return true;
            }
            last = now;
        }
        false
    }

    /// CPU use, as a percentage of one core, while the pointer is away and nothing should be moving. A sample during
    /// which an outside event made the notch work (`outside_event`) is taken again; the last one counts.
    fn idle_cpu_percent(&self) -> f64 {
        let mut attempt = 1;
        loop {
            park_pointer();
            let log_length_before = self.log_text().len();
            let before = self.measure().cpu_ms;
            sleep(Duration::from_secs(IDLE_SECONDS));
            let used = self.measure().cpu_ms - before;
            let percent = (100.0 * used / (IDLE_SECONDS as f64 * 1000.0) * 1000.0).round() / 1000.0;
            let log_text = self.log_text();
            let Some(event) = outside_event(log_text.get(log_length_before..).unwrap_or_default()) else {
                return percent;
            };
            if attempt == IDLE_ATTEMPTS {
                println!("    still disturbed after {IDLE_ATTEMPTS} samples: {event}");
                return percent;
            }
            println!("    disturbed ({percent} %) by: {event}; measuring again");
            attempt += 1;
        }
    }

    fn log_text(&self) -> String {
        fs::read_to_string(&self.log).unwrap_or_default()
    }

    fn observe(&self, label: &str) -> Measurement {
        let before = self.measure();
        let started = Instant::now();
        sleep(Duration::from_secs(IDLE_SECONDS));
        let after = self.measure();
        let cpu = 100.0 * (after.cpu_ms - before.cpu_ms) / started.elapsed().as_secs_f64() / 1000.0;
        println!(
            "  {label}: CPU {cpu:.3} % of one core; RAM {:.1} MB private working set; {} handles; {} s sample",
            after.private_working_set_mb, after.handles, IDLE_SECONDS
        );
        after
    }

    fn window(&self) -> windows::Win32::Foundation::HWND {
        let window = unsafe { FindWindowW(w!("NotchWindow"), None) }.expect("test Notch window");
        let mut process_id = 0;
        unsafe { GetWindowThreadProcessId(window, Some(&mut process_id)) };
        assert_eq!(process_id, self.process.id(), "refuse to drive a window owned by another process");
        window
    }

    fn open_width_dip(&self) -> f32 {
        let window = self.window();
        let mut bounds = Default::default();
        unsafe { GetWindowRect(window, &mut bounds) }.expect("test window bounds");
        let scale = unsafe { GetDpiForWindow(window) } as f32 / 96.0;
        (bounds.right - bounds.left) as f32 / scale
    }

    fn swipe_workload(&self) {
        let window = self.window();
        let before = self.measure();
        let started = Instant::now();
        let mut peak_memory = before.private_working_set_mb;
        let mut minimum_width = self.open_width_dip();
        let mut maximum_width = minimum_width;
        for _ in 0..10 {
            for direction in [1, -1] {
                for _ in 0..12 {
                    let amount = (20 * direction) as i16;
                    let scroll = WPARAM(usize::from(amount as u16) << 16);
                    unsafe { PostMessageW(Some(window), WM_MOUSEHWHEEL, scroll, LPARAM(0)) }
                        .expect("send touchpad-style input to the test window");
                    sleep(Duration::from_millis(16));
                }
                sleep(Duration::from_millis(250));
                peak_memory = peak_memory.max(self.measure().private_working_set_mb);
                let width = self.open_width_dip();
                minimum_width = minimum_width.min(width);
                maximum_width = maximum_width.max(width);
            }
        }
        unsafe { PostMessageW(Some(window), WM_MOUSEHWHEEL, WPARAM(140usize << 16), LPARAM(0)) }
            .expect("end the workload with a partial touchpad gesture");
        sleep(Duration::from_millis(750));
        let after = self.measure();
        let cpu = 100.0 * (after.cpu_ms - before.cpu_ms) / started.elapsed().as_secs_f64() / 1000.0;
        println!(
            "  swiping 10 round trips: CPU {cpu:.3} % of one core; peak sampled RAM {peak_memory:.1} MB; RAM growth {:.1} MB; handle growth {}; width range {minimum_width:.1}..{maximum_width:.1} DIP",
            after.private_working_set_mb - before.private_working_set_mb,
            i64::from(after.handles) - i64::from(before.handles)
        );
        assert!(
            maximum_width - minimum_width > 1.0,
            "the swipe workload must actually move between different-width pages"
        );
    }
}

fn lyrics_comparison(executable: &Path, log: &Path, center_x: i32) -> bool {
    if !media_is_playing().expect("inspect the playback state") {
        println!("Lyrics comparison NOT RUN: Windows has no currently playing media session.");
        return false;
    }
    let mut plain_width = 0.0;
    for enabled in [false, true] {
        let state = if enabled { "on" } else { "off" };
        let profile = log.parent().expect("log directory").join(format!("perf-lyrics-{state}"));
        let settings = profile.join("notch");
        fs::create_dir_all(&settings).expect("create isolated lyrics test profile");
        fs::write(
            settings.join("settings.txt"),
            format!("media.lyrics={state}\nnotch.hide_when_idle=off\nnotch.fullscreen=music\nnavigation.page=media\n"),
        )
        .expect("write isolated lyrics test settings");
        println!("Lyrics {state}: isolated default-page profile, real Windows media playback");
        let notch = RunningNotch::start_with_profile(executable, log, None, Some(&profile));
        move_pointer(center_x, HOVER_Y);
        sleep(Duration::from_secs(8));
        let deadline = Instant::now() + Duration::from_secs(30);
        if enabled {
            // The real lyric pane is 582 DIP wide; the plain player is at most 285 before padding.
            let has_lyrics_width = |width: f32| width >= 500.0 && width >= plain_width + 100.0;
            while !has_lyrics_width(notch.open_width_dip()) && Instant::now() < deadline {
                sleep(Duration::from_secs(1));
            }
            if !has_lyrics_width(notch.open_width_dip()) {
                println!(
                    "Lyrics on NOT VERIFIED: the live synced-lyrics pane did not appear; no active-lyrics numbers reported."
                );
                return false;
            }
            println!("  live lyrics pane verified by expanded width: {:.1} DIP", notch.open_width_dip());
        } else {
            plain_width = notch.open_width_dip();
            println!("  plain media pane: {plain_width:.1} DIP");
        }
        park_pointer();
        sleep(Duration::from_secs(2));
        notch.observe(&format!("lyrics {state}, collapsed"));
        move_pointer(center_x, HOVER_Y);
        sleep(Duration::from_secs(2));
        notch.observe(&format!("lyrics {state}, held open by the pointer"));
        if !media_is_playing().expect("verify playback continued during the sample") {
            println!("Lyrics comparison INVALID: media playback stopped during the sample.");
            return false;
        }
    }
    true
}

impl Drop for RunningNotch {
    fn drop(&mut self) {
        park_pointer();
        let _ = self.process.kill();
    }
}

fn ticks(time: FILETIME) -> u64 {
    (u64::from(time.dwHighDateTime) << 32) | u64::from(time.dwLowDateTime)
}

fn move_pointer(x: i32, y: i32) {
    unsafe {
        let _ = SetCursorPos(x, y);
    }
}

fn park_pointer() {
    move_pointer(PARKED_POINTER.0, PARKED_POINTER.1);
}

/// Memory only this process uses: working-set pages shared with no other process.
fn private_working_set_mb(process: HANDLE) -> f64 {
    // The buffer starts with the entry count, followed by one word per page.
    let mut capacity = 8192usize;
    loop {
        let mut buffer = vec![0usize; capacity + 1];
        let size_bytes = (buffer.len() * size_of::<usize>()) as u32;
        let is_complete = unsafe { K32QueryWorkingSet(process, buffer.as_mut_ptr().cast(), size_bytes).as_bool() };
        let count = buffer[0];
        if is_complete {
            // Bit 8 of each word is set when the page can be shared with other processes.
            let private_pages = buffer[1..=count].iter().filter(|word| (*word >> 8) & 1 == 0).count();
            return (private_pages * PAGE_SIZE_BYTES) as f64 / BYTES_PER_MEGABYTE;
        }
        capacity = if count > capacity { count + 512 } else { capacity * 2 };
    }
}

struct Report {
    failures: Vec<String>,
}

impl Report {
    fn check(&mut self, name: &str, value: f64, limit: f64, unit: &str) {
        let is_ok = value <= limit;
        println!("  {name:<34} {value:>8} {unit:<4} (limit {limit} {unit})  {}", if is_ok { "ok" } else { "FAIL" });
        if !is_ok {
            self.failures.push(name.to_string());
        }
    }

    fn require(&mut self, name: &str, is_ok: bool, detail: &str) {
        println!("  {name:<34} {detail}  {}", if is_ok { "ok" } else { "FAIL" });
        if !is_ok {
            self.failures.push(name.to_string());
        }
    }
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    unsafe { RoInitialize(RO_INIT_MULTITHREADED) }.expect("initialize Windows Runtime for media verification");
    if arguments.iter().any(|argument| argument == "--media-status") {
        let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
            .and_then(|request| request.join())
            .expect("inspect Windows media sessions");
        println!(
            "Available Windows media sessions: {}",
            manager.GetSessions().and_then(|sessions| sessions.Size()).expect("media session count")
        );
        match media_is_playing() {
            Ok(is_playing) => println!("Current Windows media session playing: {is_playing}"),
            Err(error) => {
                eprintln!("Could not inspect the Windows media session: {error}");
                return ExitCode::FAILURE;
            }
        }
        return ExitCode::SUCCESS;
    }
    let value_of =
        |flag: &str| arguments.iter().position(|argument| argument == flag).and_then(|index| arguments.get(index + 1));
    let (Some(executable), Some(log)) = (value_of("--exe"), value_of("--log")) else {
        eprintln!(
            "usage: perf --exe <notch.exe> --log <log file> [--with-media] [--extended] [--lyrics-only] [--screen-mode]"
        );
        return ExitCode::from(2);
    };
    let (executable, log) = (PathBuf::from(executable), PathBuf::from(log));
    let with_media = arguments.iter().any(|argument| argument == "--with-media");
    let extended = arguments.iter().any(|argument| argument == "--extended");
    let with_screen_mode = arguments.iter().any(|argument| argument == "--screen-mode");

    unsafe {
        let _ = SetProcessDPIAware();
    }
    let center_x = unsafe { GetSystemMetrics(SM_CXSCREEN) } / 2;
    if arguments.iter().any(|argument| argument == "--lyrics-only") {
        return if lyrics_comparison(&executable, &log, center_x) { ExitCode::SUCCESS } else { ExitCode::from(2) };
    }
    let mut report = Report { failures: Vec::new() };

    println!("Notch performance check: shelf page only, {IDLE_SECONDS} s idle, {CYCLES} open/close cycles");
    {
        let notch = RunningNotch::start(&executable, &log, Some("shelf"));
        let started = notch.measure();
        println!(
            "  after start: {:.1} MB private working set, {} handles",
            started.private_working_set_mb, started.handles
        );

        println!("Idle, collapsed:");
        report.require("settles after start-up", notch.wait_until_quiet(), "");
        report.check("CPU while idle", notch.idle_cpu_percent(), MAX_IDLE_CPU_PERCENT, "%");
        report.check("memory (private working set)", round(started.private_working_set_mb), MAX_MEMORY_MB, "MB");

        println!("Opening and closing {CYCLES} times:");
        let previous_open_events = fs::read_to_string(&log)
            .expect("read open events before measured cycles")
            .lines()
            .filter(|line| line.contains("open="))
            .count();
        let before = notch.measure();
        for _ in 0..CYCLES {
            move_pointer(center_x, HOVER_Y);
            sleep(Duration::from_millis(1000));
            park_pointer();
            sleep(Duration::from_millis(1000));
        }
        let after = notch.measure();
        let cpu_per_cycle = ((after.cpu_ms - before.cpu_ms) / f64::from(CYCLES)).round();
        report.check("CPU per open+close cycle", cpu_per_cycle, MAX_CPU_MS_PER_CYCLE, "ms");
        let growth = round(after.private_working_set_mb - before.private_working_set_mb);
        report.check("memory growth over the cycles", growth, MAX_MEMORY_GROWTH_MB, "MB");
        let handle_growth = f64::from(after.handles) - f64::from(before.handles);
        report.check("handle growth over the cycles", handle_growth, MAX_HANDLE_GROWTH, "");

        let log_text = fs::read_to_string(&log).unwrap_or_default();
        let events = log_text.lines().filter(|line| line.contains("open=")).count() - previous_open_events;
        let expected = 2 * CYCLES as usize;
        let open_events_ok = events == expected;
        report.require("open events logged", open_events_ok, &format!("{events:>8}      (expected {expected})"));
        if !open_events_ok {
            println!("    log:\n{log_text}");
        }

        println!("Idle again after the animations (timers must have stopped):");
        report.require("settles after the animations", notch.wait_until_quiet(), "");
        report.check("CPU while idle after cycles", notch.idle_cpu_percent(), MAX_IDLE_CPU_PERCENT, "%");
    }

    scenarios::devices_page(&executable, &log, center_x, &mut report);
    scenarios::stats_page(&executable, &log, center_x, &mut report);
    scenarios::pinned_idle(&executable, &log, center_x, &mut report);
    if with_screen_mode {
        scenarios::screen_mode(&executable, &log, &mut report);
    }

    if with_media || extended {
        println!("All pages on (informational; play music first):");
        let notch = RunningNotch::start(&executable, &log, None);
        let measured = notch.measure();
        let idle = notch.idle_cpu_percent();
        println!(
            "  {:.1} MB private working set, {} handles; CPU {idle} % of one core over {IDLE_SECONDS} s",
            measured.private_working_set_mb, measured.handles
        );
        if extended {
            move_pointer(center_x, HOVER_Y);
            sleep(Duration::from_secs(2));
            notch.observe("all pages, held open by the pointer");
            notch.swipe_workload();
            sleep(Duration::from_secs(2));
            notch.observe("all pages, after touchpad snap");
            park_pointer();
            sleep(Duration::from_secs(2));
            notch.observe("all pages, collapsed after swipes");
        }
    }
    let _ = fs::remove_file(&log);

    if report.failures.is_empty() {
        println!("All performance checks passed.");
        ExitCode::SUCCESS
    } else {
        println!("FAILED: {}", report.failures.join(", "));
        ExitCode::FAILURE
    }
}

fn round(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_notification_database_write_during_an_idle_sample_is_an_outside_event() {
        let log = "1 ms  foreground class=Chrome_WidgetWin_1 process=msedge.exe mode=Maximized\n\
                   2 ms  notifications: read 49 listed in 2478 ms, 0 new\n";
        assert_eq!(outside_event(log), Some("notifications: read 49 listed in 2478 ms, 0 new"));
    }

    #[test]
    fn an_activity_appearing_in_the_pill_during_an_idle_sample_is_an_outside_event() {
        let log = "1 ms  pill shows transients=[bluetooth] badges=[volume] banner=none\n";
        assert_eq!(outside_event(log), Some("pill shows transients=[bluetooth] badges=[volume] banner=none"));
    }

    #[test]
    fn an_idle_sample_with_only_the_notchs_own_lines_is_not_taken_again() {
        let log = "1 ms  notifications: listening, 50 waiting, newest id 165394, first read took 2222 ms\n\
                   2 ms  frame timer started\n3 ms  frame timer stopped\n";
        assert_eq!(outside_event(log), None);
    }
}
