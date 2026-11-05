//! Topmost restoring, session recovery and the visibility keys.

use super::*;

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
    notch.set_hovering(true);
    assert_ne!(unsafe { GetWindowLongW(notch.window, GWL_EXSTYLE) } as u32 & WS_EX_TOPMOST.0, 0);
}

#[test]
fn session_recovery_clears_stale_hover_and_allows_a_hidden_notch_to_open_again() {
    let mut notch = rig(SampleOptions::default());
    notch.hiding.screen_mode = ScreenMode::Maximized;
    notch.hiding.hides_when_idle = true;
    notch.hover.is_hovering = true;
    notch.hiding.is_revealed = true;
    notch.hover.is_dragging_files = true;
    notch.hover.is_close_pending = true;
    notch.springs.expansion.jump_to(1.0);
    notch.reset_session_interaction();
    assert!(!notch.hover.is_hovering);
    assert!(!notch.hiding.is_revealed);
    assert!(!notch.hover.is_dragging_files);
    assert!(!notch.hover.is_close_pending);
    assert_eq!(notch.springs.expansion.target, 0.0);
    assert!(notch.is_hidden());
    notch.set_hovering(true);
    notch.timer_fired(REVEAL_TIMER_ID);
    assert!(notch.hiding.is_revealed);
    notch.timer_fired(EXPAND_TIMER_ID);
    notch.timer_fired(STAGE_TIMER_ID);
    assert_eq!(notch.springs.expansion.target, 1.0);
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
