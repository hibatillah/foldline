//! Pausing: the notch hides completely, the stored key decides, and every popup test keeps Windows' popups untouched.

use super::*;
use store::pause::{PAUSED_UNTIL_KEY, Pause};
use windows::Win32::UI::WindowsAndMessaging::{IsWindowVisible, KillTimer, SW_SHOWNOACTIVATE, ShowWindow};

/// Settings with both Windows-popup keys off, so no test switches a popup on or off.
fn popups_off(settings_file: &SettingsFile) -> Settings {
    let mut settings = settings_file.load();
    settings.set_bool("notch.hide_windows_volume_popup", false);
    settings.set_bool("notch.hide_windows_notifications", false);
    settings
}

fn visible_rig(settings_file: &SettingsFile) -> Notch {
    let notch = configured_rig(SampleOptions::default(), popups_off(settings_file), Vec::new());
    let _ = unsafe { ShowWindow(notch.window, SW_SHOWNOACTIVATE) };
    notch
}

fn is_visible(notch: &Notch) -> bool {
    unsafe { IsWindowVisible(notch.window) }.as_bool()
}

fn stored(notch: &Notch) -> &str {
    notch.settings.get_string(PAUSED_UNTIL_KEY).unwrap_or("")
}

#[test]
fn pausing_hides_the_window_and_writes_the_end_time() {
    let settings_file = SettingsFile::new("pause-hides");
    let mut notch = visible_rig(&settings_file);
    assert!(is_visible(&notch));
    notch.set_pause(Pause::Until(1_600), 1_000);
    assert!(!is_visible(&notch));
    assert_eq!(stored(&notch), "1600");
    assert!(notch.is_paused());
}

#[test]
fn resuming_shows_the_window_and_clears_the_key() {
    let settings_file = SettingsFile::new("pause-resumes");
    let mut notch = visible_rig(&settings_file);
    notch.set_pause(Pause::Until(1_600), 1_000);
    notch.set_pause(Pause::Off, 1_100);
    assert!(is_visible(&notch));
    assert!(!notch.is_paused());
    assert_eq!(stored(&notch), "");
}

#[test]
fn pausing_an_open_pinned_notch_closes_it_and_resume_keeps_the_pin() {
    let settings_file = SettingsFile::new("pause-pinned");
    let mut notch = visible_rig(&settings_file);
    notch.screen_position = POINT { x: -100_000, y: -100_000 };
    open(&mut notch, "media");
    notch.set_pinned(true);
    notch.set_pause(Pause::Manual, 1_000);
    assert!(!notch.open.is_open);
    assert!(notch.open.is_pinned);
    notch.set_pause(Pause::Off, 1_100);
    assert!(notch.open.is_pinned);
}

#[test]
fn a_settings_change_while_paused_keeps_the_notch_hidden() {
    let settings_file = SettingsFile::new("pause-reload");
    let mut notch = visible_rig(&settings_file);
    notch.set_pause(Pause::Manual, 1_000);
    notch.settings.set_bool("notch.hide_when_idle", true);
    notch.reload_settings();
    assert!(!is_visible(&notch));
    assert!(notch.is_paused());
}

#[test]
fn the_end_timer_resumes_a_timed_pause() {
    let settings_file = SettingsFile::new("pause-timer-ends");
    let mut notch = visible_rig(&settings_file);
    notch.set_pause(Pause::Until(1_600), 1_000);
    notch.pause_timer_fired(1_600);
    assert!(!notch.is_paused());
    assert!(is_visible(&notch));
    assert_eq!(stored(&notch), "");
}

#[test]
fn an_early_timer_keeps_the_pause() {
    let settings_file = SettingsFile::new("pause-timer-early");
    let mut notch = visible_rig(&settings_file);
    notch.set_pause(Pause::Until(1_600), 1_000);
    notch.pause_timer_fired(1_500);
    assert!(notch.is_paused());
}

#[test]
fn a_stale_stored_pause_is_cleared_and_ignored() {
    let settings_file = SettingsFile::new("pause-stale");
    let mut notch = visible_rig(&settings_file);
    notch.settings.set_string(PAUSED_UNTIL_KEY, "1000");
    notch.apply_stored_pause(2_000);
    assert!(!notch.is_paused());
    assert_eq!(stored(&notch), "");
}

#[test]
fn a_start_clears_a_pause_left_by_a_quit() {
    let settings_file = SettingsFile::new("pause-start");
    let mut settings = popups_off(&settings_file);
    settings.set_string(PAUSED_UNTIL_KEY, "manual");
    let mut notch = configured_rig(SampleOptions::default(), settings, Vec::new());
    notch.start();
    assert!(!notch.is_paused());
    assert_eq!(stored(&notch), "");
}

#[test]
fn a_manual_pause_arms_no_timer() {
    let settings_file = SettingsFile::new("pause-manual");
    let mut notch = visible_rig(&settings_file);
    notch.set_pause(Pause::Manual, 1_000);
    assert!(unsafe { KillTimer(Some(notch.window), PAUSE_TIMER_ID) }.is_err());
}

#[test]
fn waking_after_the_end_resumes_at_once() {
    let settings_file = SettingsFile::new("pause-wake");
    let mut notch = visible_rig(&settings_file);
    notch.set_pause(Pause::Until(1_600), 1_000);
    notch.apply_stored_pause(5_000);
    assert!(!notch.is_paused());
}

/// Whether `timer_key` was armed on the rig's window; disarms it as a side effect.
fn is_armed(notch: &Notch, timer_key: usize) -> bool {
    unsafe { KillTimer(Some(notch.window), timer_key) }.is_ok()
}

#[test]
fn the_clock_timer_stops_while_paused_and_returns_after_the_resume() {
    let settings_file = SettingsFile::new("pause-clock");
    let mut notch = visible_rig(&settings_file);
    notch.arm_clock_timer();
    notch.set_pause(Pause::Manual, 1_000);
    assert!(!is_armed(&notch, timers::CLOCK_TIMER_ID), "no clock timer while paused");
    notch.timer_fired(timers::CLOCK_TIMER_ID);
    assert!(!is_armed(&notch, timers::CLOCK_TIMER_ID), "a late tick does not re-arm it");
    notch.set_pause(Pause::Off, 1_100);
    assert!(is_armed(&notch, timers::CLOCK_TIMER_ID), "the resume arms it again");
}

#[test]
fn a_request_to_animate_while_paused_starts_no_frame_timer() {
    let settings_file = SettingsFile::new("pause-animation");
    let mut notch = visible_rig(&settings_file);
    notch.set_pause(Pause::Manual, 1_000);
    notch.springs.expansion.target = 1.0;
    notch.start_animation();
    assert!(!notch.frame_timer.is_running);
    assert!(!is_armed(&notch, ANIMATION_TIMER_ID));
}

#[test]
fn a_screen_mode_while_paused_is_kept_and_arms_no_timer() {
    use crate::host::screen_mode::ScreenMode;
    let settings_file = SettingsFile::new("pause-screen-mode");
    let mut notch = visible_rig(&settings_file);
    notch.set_pause(Pause::Manual, 1_000);
    notch.set_screen_mode(ScreenMode::Maximized);
    assert_eq!(notch.hiding.screen_mode, ScreenMode::Maximized);
    assert!(!is_armed(&notch, RAISE_TIMER_ID));
    assert!(!is_armed(&notch, WINDOW_REVEAL_TIMER_ID));
}

#[test]
fn a_screen_mode_set_while_paused_is_applied_on_resume() {
    use crate::host::screen_mode::ScreenMode;
    let settings_file = SettingsFile::new("pause-mode-resume");
    let mut settings = popups_off(&settings_file);
    settings.set_string("notch.fullscreen", "hidden");
    let mut notch = configured_rig(SampleOptions::default(), settings, Vec::new());
    let _ = unsafe { ShowWindow(notch.window, SW_SHOWNOACTIVATE) };
    notch.set_pause(Pause::Manual, 1_000);
    notch.set_screen_mode(ScreenMode::Fullscreen);
    notch.set_pause(Pause::Off, 1_100);
    assert!(notch.is_suppressed(), "the full-screen window that came during the pause hides the notch");
    assert!(!is_visible(&notch));

    notch.set_pause(Pause::Manual, 1_200);
    notch.set_screen_mode(ScreenMode::Normal);
    notch.set_pause(Pause::Off, 1_300);
    assert!(!notch.is_suppressed());
    assert!(is_visible(&notch), "a regular window after the pause shows the notch");
}
