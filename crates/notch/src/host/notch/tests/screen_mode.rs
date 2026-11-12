//! Topmost restoring, session recovery and the visibility keys.

use super::*;
use crate::host::notch::hiding::WINDOW_REVEAL_DELAY_MILLISECONDS;

#[test]
fn an_unchanged_screen_mode_restores_a_demoted_notch_without_activating_it() {
    use windows::Win32::UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GetForegroundWindow, GetWindowLongW, HWND_NOTOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
        SetWindowPos, WS_EX_TOPMOST,
    };
    let mut notch = rig(SampleOptions::default());
    notch.hiding.screen_mode = ScreenMode::Maximized;
    let foreground = unsafe { GetForegroundWindow() };
    unsafe {
        SetWindowPos(notch.window, Some(HWND_NOTOPMOST), 0, 0, 0, 0, SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE).unwrap();
    }
    notch.set_screen_mode(ScreenMode::Maximized);
    assert_ne!(unsafe { GetWindowLongW(notch.window, GWL_EXSTYLE) } as u32 & WS_EX_TOPMOST.0, 0);
    assert_eq!(unsafe { GetForegroundWindow() }, foreground);
}

#[test]
fn hover_restores_topmost_before_the_drop_target_hands_off_input() {
    use windows::Win32::UI::WindowsAndMessaging::{GWL_EXSTYLE, GetWindowLongW, WS_EX_TOPMOST};
    let mut notch = rig(SampleOptions::default());
    notch.hiding.hides_when_idle = true;
    assert_eq!(unsafe { GetWindowLongW(notch.window, GWL_EXSTYLE) } as u32 & WS_EX_TOPMOST.0, 0);
    notch.set_open(true);
    assert_ne!(unsafe { GetWindowLongW(notch.window, GWL_EXSTYLE) } as u32 & WS_EX_TOPMOST.0, 0);
}

#[test]
fn session_recovery_clears_stale_hover_and_allows_a_hidden_notch_to_open_again() {
    let mut notch = rig(SampleOptions::default());
    notch.hiding.screen_mode = ScreenMode::Maximized;
    notch.hiding.hides_when_idle = true;
    notch.open.is_open = true;
    notch.hiding.is_revealed = true;
    notch.open.is_dragging_files = true;
    notch.open.is_close_pending = true;
    notch.springs.expansion.jump_to(1.0);
    notch.reset_session_interaction();
    assert!(!notch.open.is_open);
    assert!(!notch.hiding.is_revealed);
    assert!(!notch.open.is_dragging_files);
    assert!(!notch.open.is_close_pending);
    assert_eq!(notch.springs.expansion.target, 0.0);
    assert!(notch.is_hidden());
    notch.set_open(true);
    notch.timer_fired(REVEAL_TIMER_ID);
    assert!(notch.hiding.is_revealed);
    notch.timer_fired(EXPAND_TIMER_ID);
    assert_eq!(notch.springs.expansion.target, 1.0);
}

/// Whether the window reveal timer was armed; disarms it as a side effect.
fn window_reveal_is_armed(notch: &Notch) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::KillTimer;
    unsafe { KillTimer(Some(notch.window), WINDOW_REVEAL_TIMER_ID) }.is_ok()
}

fn hidden_over_maximized() -> Notch {
    let mut notch = rig(SampleOptions::default());
    notch.hiding.maximized_showing = Showing::Hidden;
    notch.set_screen_mode(ScreenMode::Maximized);
    notch
}

#[test]
fn switching_between_maximized_windows_through_a_brief_regular_window_never_shows_the_notch() {
    let mut notch = hidden_over_maximized();
    notch.set_screen_mode(ScreenMode::Normal);
    assert!(notch.is_suppressed(), "the notch waits before it comes back");
    notch.set_screen_mode(ScreenMode::Maximized);
    assert_eq!(notch.hiding.pending_screen_mode, None);
    assert!(!window_reveal_is_armed(&notch), "a covering window inside the second disarms the reveal");
    notch.timer_fired(WINDOW_REVEAL_TIMER_ID);
    assert!(notch.is_suppressed());
}

#[test]
fn a_regular_window_that_stays_in_front_for_half_a_second_shows_the_notch_again() {
    assert_eq!(WINDOW_REVEAL_DELAY_MILLISECONDS, 500);
    let mut notch = hidden_over_maximized();
    notch.set_screen_mode(ScreenMode::Normal);
    assert!(notch.is_suppressed());
    notch.timer_fired(WINDOW_REVEAL_TIMER_ID);
    assert!(!notch.is_suppressed());
    assert_eq!(notch.hiding.screen_mode, ScreenMode::Normal);
    assert!(!window_reveal_is_armed(&notch), "the one-shot timer is gone once the notch is back");
}

#[test]
fn closing_a_screenshot_overlay_brings_the_notch_back_only_after_the_reveal_delay() {
    let mut notch = rig(SampleOptions::default());
    notch.set_screen_mode(ScreenMode::Capture);
    assert!(notch.is_suppressed(), "the overlay hides the notch at once");
    assert!(!window_reveal_is_armed(&notch));
    notch.set_screen_mode(ScreenMode::Normal);
    assert!(notch.is_suppressed(), "the notch waits before it comes back");
    assert_eq!(notch.hiding.pending_screen_mode, Some(ScreenMode::Normal));
    notch.timer_fired(WINDOW_REVEAL_TIMER_ID);
    assert!(!notch.is_suppressed());
    assert_eq!(notch.hiding.screen_mode, ScreenMode::Normal);
}

#[test]
fn a_screenshot_overlay_over_a_music_showing_full_screen_window_hides_the_notch_at_once() {
    let mut notch = rig(SampleOptions::default());
    notch.hiding.fullscreen_showing = Showing::Music;
    notch.set_screen_mode(ScreenMode::Fullscreen);
    assert!(!notch.is_suppressed());
    notch.set_screen_mode(ScreenMode::Capture);
    assert!(notch.is_suppressed());
    assert!(!window_reveal_is_armed(&notch), "hiding never waits");
}

#[test]
fn a_window_that_covers_the_screen_hides_the_notch_at_once() {
    let mut notch = rig(SampleOptions::default());
    notch.hiding.maximized_showing = Showing::Hidden;
    notch.hiding.fullscreen_showing = Showing::ActivitiesOnly;
    notch.set_screen_mode(ScreenMode::Fullscreen);
    assert!(notch.is_hidden(), "the idle pill hides as soon as the full-screen window is in front");
    assert!(!window_reveal_is_armed(&notch));
    notch.set_screen_mode(ScreenMode::Maximized);
    assert!(notch.is_suppressed(), "going from activities only to hidden waits for nothing");
    assert!(!window_reveal_is_armed(&notch));
}

#[test]
fn a_banner_still_shows_over_an_activities_only_window_while_the_reveal_waits() {
    let mut notch = rig(SampleOptions::default());
    notch.hiding.maximized_showing = Showing::ActivitiesOnly;
    notch.set_screen_mode(ScreenMode::Maximized);
    assert!(notch.is_hidden());
    notch.set_screen_mode(ScreenMode::Normal);
    assert!(notch.is_hidden(), "the idle pill stays hidden during the wait");
    notch.presences.banner = Some((Source::Activity(0), 330.0, 70.0));
    assert!(!notch.is_hidden(), "an activity shows as it did over the maximized window");
    notch.presences.banner = None;
    notch.timer_fired(WINDOW_REVEAL_TIMER_ID);
    assert!(!notch.is_hidden());
}

#[test]
fn fresh_settings_use_the_requested_visibility_defaults() {
    let settings_file = SettingsFile::new("defaults");
    let notch = configured_rig(SampleOptions::default(), settings_file.load(), Vec::new());
    assert!(!notch.hiding.hides_when_idle);
    assert_eq!(notch.hiding.maximized_showing, Showing::MusicWithLyrics);
    assert_eq!(notch.hiding.fullscreen_showing, Showing::Hidden);
}

#[test]
fn the_old_maximized_override_shows_even_an_empty_notch_with_hide_when_idle_enabled() {
    let settings_file = SettingsFile::new("maximized");
    let mut settings = settings_file.load();
    settings.set_bool("notch.show_over_maximized", true);
    settings.set_bool(HIDE_WHEN_IDLE_SETTING, true);
    let mut notch = configured_rig(SampleOptions::default(), settings, Vec::new());
    notch.hiding.screen_mode = ScreenMode::Maximized;
    notch.refresh_layout();
    assert!(!notch.is_hidden());
    notch.hiding.screen_mode = ScreenMode::Fullscreen;
    assert!(notch.is_suppressed(), "the maximized override must not override fullscreen hiding");
}

#[test]
fn disabling_fullscreen_hiding_does_not_show_the_notch_over_maximized_windows() {
    let mut notch = rig(SampleOptions::default());
    notch.hiding.fullscreen_showing = Showing::Music;
    notch.hiding.screen_mode = ScreenMode::Maximized;
    assert!(notch.is_hidden());
}

#[test]
fn saved_old_visibility_choices_migrate_at_start() {
    let settings_file = SettingsFile::new("saved-visibility");
    let mut settings = settings_file.load();
    settings.set_bool(HIDE_WHEN_IDLE_SETTING, true);
    settings.set_bool("notch.hide_for_fullscreen", false);
    settings.set_bool("notch.lyrics_over_maximized", false);
    let notch = configured_rig(SampleOptions::default(), settings_file.load(), Vec::new());
    assert!(notch.hiding.hides_when_idle);
    assert_eq!(notch.hiding.fullscreen_showing, Showing::Music);
    assert_eq!(notch.hiding.maximized_showing, Showing::ActivitiesOnly);
    assert_eq!(settings_file.load().get_string("notch.fullscreen"), Some("music"));
    assert_eq!(settings_file.load().get_string("notch.maximized"), Some("activities"));
}

#[test]
fn saved_showings_are_read_and_an_unknown_word_falls_back_to_the_default() {
    let settings_file = SettingsFile::new("saved-showings");
    fs::write(&settings_file.0, "notch.maximized=music\nnotch.fullscreen=sometimes\n").unwrap();
    let notch = configured_rig(SampleOptions::default(), settings_file.load(), Vec::new());
    assert_eq!(notch.hiding.maximized_showing, Showing::Music);
    assert_eq!(notch.hiding.fullscreen_showing, Showing::Hidden);
}

#[test]
fn reloading_a_showing_over_a_maximized_window_applies_at_once_and_releases_the_window() {
    let settings_file = SettingsFile::new("reload-showing");
    fs::write(&settings_file.0, "notch.maximized=hidden\n").unwrap();
    let mut notch = configured_rig(SampleOptions::default(), settings_file.load(), Vec::new());
    notch.set_screen_mode(ScreenMode::Maximized);
    assert!(notch.is_suppressed());
    fs::write(&settings_file.0, "notch.maximized=music\n").unwrap();
    notch.reload_settings();
    assert_eq!(notch.hiding.maximized_showing, Showing::Music);
    assert!(!notch.is_suppressed(), "a choice other than hidden must not leave the window hidden");
    assert!(!notch.is_hidden());
    assert_eq!(notch.springs.resting_height.target, RESTING_PILL_HEIGHT);
    fs::write(&settings_file.0, "notch.maximized=bogus\n").unwrap();
    notch.reload_settings();
    assert_eq!(notch.hiding.maximized_showing, Showing::Music, "an unknown word keeps the current choice");
    fs::write(&settings_file.0, "notch.maximized=activities\n").unwrap();
    notch.reload_settings();
    assert!(notch.is_hidden());
    assert_eq!(notch.springs.resting_height.target, notch.hidden_height());
}

#[test]
fn a_hand_edited_old_key_migrates_on_reload() {
    let settings_file = SettingsFile::new("reload-old-key");
    let mut notch = configured_rig(SampleOptions::default(), settings_file.load(), Vec::new());
    fs::write(&settings_file.0, "notch.show_over_maximized=on\n").unwrap();
    notch.reload_settings();
    assert_eq!(notch.hiding.maximized_showing, Showing::Music);
    assert_eq!(settings_file.load().get_string("notch.maximized"), Some("music"));
}
