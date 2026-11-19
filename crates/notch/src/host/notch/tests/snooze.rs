//! Snoozing: notices are dropped in the collapsed and the open notch, indicators keep showing, and the key decides.

use super::*;
use store::snooze::{SNOOZED_UNTIL_KEY, Snooze};
use windows::Win32::UI::WindowsAndMessaging::KillTimer;

/// A stand-in activity whose presence and notice flag a test sets; a dismissed banner clears its presence.
struct Shown {
    presence: Rc<Cell<Option<Presence>>>,
    is_notice: Rc<Cell<bool>>,
    dismissals: Rc<Cell<u32>>,
    header_width: Option<f32>,
}

impl Activity for Shown {
    fn id(&self) -> &'static str {
        "shown"
    }

    fn presence(&self) -> Option<Presence> {
        self.presence.get()
    }

    fn is_notice(&self, _presence: Option<Presence>) -> bool {
        self.is_notice.get()
    }

    fn dismiss_banner(&mut self, _context: &mut Context) {
        self.dismissals.set(self.dismissals.get() + 1);
        self.presence.set(None);
        self.is_notice.set(false);
    }

    fn header_status_width(&self) -> Option<f32> {
        self.header_width
    }

    fn draw(&self, _canvas: &Canvas, _area: Rect, _opacity: f32) {}
}

struct Handles {
    presence: Rc<Cell<Option<Presence>>>,
    is_notice: Rc<Cell<bool>>,
    dismissals: Rc<Cell<u32>>,
}

impl Handles {
    fn show(&self, presence: Option<Presence>, is_notice: bool) {
        self.presence.set(presence);
        self.is_notice.set(is_notice);
    }
}

const MESSAGE: Presence = Presence::Transient { width: 120.0 };
const BANNER: Presence = Presence::Banner { width: 340.0, height: 58.0 };

/// A started rig with one stand-in activity that has a header mark, and a settings file of its own.
fn snooze_rig(settings_file: &SettingsFile) -> (Notch, Handles) {
    let (presence, is_notice, dismissals) =
        (Rc::new(Cell::new(None)), Rc::new(Cell::new(false)), Rc::new(Cell::new(0)));
    let activity = Shown {
        presence: presence.clone(),
        is_notice: is_notice.clone(),
        dismissals: dismissals.clone(),
        header_width: Some(16.0),
    };
    let mut notch = configured_rig(SampleOptions::default(), settings_file.load(), vec![Box::new(activity)]);
    for slot in &mut notch.activities {
        slot.is_started = true;
    }
    settle(&mut notch);
    (notch, Handles { presence, is_notice, dismissals })
}

fn shows_transient(notch: &Notch) -> bool {
    notch.presences.transients.iter().any(|&(source, _)| source == Source::Activity(0))
}

fn shows_header_mark(notch: &Notch) -> bool {
    notch.header_marks.icons.iter().any(|&(_, source)| source == Source::Activity(0))
}

/// Disarms `timer_key` and says whether it was armed; Windows cannot ask without killing, so call it once, last.
fn disarm_was_armed(notch: &Notch, timer_key: usize) -> bool {
    unsafe { KillTimer(Some(notch.window), timer_key) }.is_ok()
}

#[test]
fn a_message_that_comes_while_snoozed_is_dropped_in_the_collapsed_notch() {
    let settings_file = SettingsFile::new("snooze-collapsed-notice");
    let (mut notch, handles) = snooze_rig(&settings_file);
    notch.set_snooze(Snooze::Manual, 1_000);
    handles.show(Some(MESSAGE), true);
    settle(&mut notch);
    assert!(!shows_transient(&notch));
}

#[test]
fn an_icon_only_level_keeps_showing_in_the_collapsed_notch_while_snoozed() {
    let settings_file = SettingsFile::new("snooze-collapsed-indicator");
    let (mut notch, handles) = snooze_rig(&settings_file);
    notch.set_snooze(Snooze::Manual, 1_000);
    handles.show(Some(MESSAGE), false);
    settle(&mut notch);
    assert!(shows_transient(&notch));
}

#[test]
fn a_notice_loses_its_header_mark_in_the_open_notch_while_snoozed() {
    let settings_file = SettingsFile::new("snooze-open-notice");
    let (mut notch, handles) = snooze_rig(&settings_file);
    open(&mut notch, "media");
    notch.set_snooze(Snooze::Manual, 1_000);
    handles.show(Some(MESSAGE), true);
    settle(&mut notch);
    assert!(!shows_header_mark(&notch));
}

#[test]
fn an_indicator_keeps_its_header_mark_in_the_open_notch_while_snoozed() {
    let settings_file = SettingsFile::new("snooze-open-indicator");
    let (mut notch, handles) = snooze_rig(&settings_file);
    open(&mut notch, "media");
    notch.set_snooze(Snooze::Manual, 1_000);
    handles.show(Some(Presence::Badge { width: 16.0 }), false);
    settle(&mut notch);
    assert!(shows_header_mark(&notch));
}

#[test]
fn a_banner_that_comes_while_snoozed_is_dismissed_for_good() {
    let settings_file = SettingsFile::new("snooze-banner");
    let (mut notch, handles) = snooze_rig(&settings_file);
    notch.set_snooze(Snooze::Manual, 1_000);
    handles.show(Some(BANNER), true);
    settle(&mut notch);
    assert_eq!(notch.presences.banner, None);
    assert_eq!(handles.presence.get(), None, "the activity was told to drop it");
}

#[test]
fn a_dropped_banner_is_dismissed_once_even_if_the_activity_keeps_it() {
    let settings_file = SettingsFile::new("snooze-banner-once");
    let (mut notch, handles) = snooze_rig(&settings_file);
    notch.set_snooze(Snooze::Manual, 1_000);
    handles.show(Some(BANNER), true);
    notch.refresh_presences();
    // The activity ignores the dismissal and keeps showing it, so later refreshes see the same notice.
    handles.show(Some(BANNER), true);
    notch.refresh_presences();
    notch.refresh_presences();
    assert_eq!(handles.dismissals.get(), 1);
    assert_eq!(notch.presences.banner, None, "still held back");
}

#[test]
fn a_message_dropped_during_the_snooze_is_not_replayed_when_it_ends() {
    let settings_file = SettingsFile::new("snooze-no-replay");
    let (mut notch, handles) = snooze_rig(&settings_file);
    notch.set_snooze(Snooze::Manual, 1_000);
    handles.show(Some(MESSAGE), true);
    settle(&mut notch);
    notch.set_snooze(Snooze::Off, 1_100);
    settle(&mut notch);
    assert!(!shows_transient(&notch), "the message still running stays dropped");
    handles.show(None, false);
    settle(&mut notch);
    handles.show(Some(MESSAGE), true);
    settle(&mut notch);
    assert!(shows_transient(&notch), "the next message shows");
}

#[test]
fn a_message_that_was_showing_when_the_snooze_starts_goes_away() {
    let settings_file = SettingsFile::new("snooze-showing");
    let (mut notch, handles) = snooze_rig(&settings_file);
    handles.show(Some(MESSAGE), true);
    settle(&mut notch);
    assert!(shows_transient(&notch));
    notch.set_snooze(Snooze::Manual, 1_000);
    assert!(!shows_transient(&notch));
}

#[test]
fn the_end_timer_ends_a_timed_snooze_and_clears_the_key() {
    let settings_file = SettingsFile::new("snooze-timer-ends");
    let (mut notch, _handles) = snooze_rig(&settings_file);
    notch.set_snooze(Snooze::Until(1_600), 1_000);
    assert_eq!(notch.settings.get_string(SNOOZED_UNTIL_KEY), Some("1600"));
    notch.snooze_timer_fired(1_600);
    assert!(!notch.is_snoozed());
    assert_eq!(notch.settings.get_string(SNOOZED_UNTIL_KEY), Some(""));
    assert!(!disarm_was_armed(&notch, SNOOZE_TIMER_ID), "no timer outlives the snooze");
}

#[test]
fn an_early_end_timer_keeps_the_snooze_and_arms_for_the_rest() {
    let settings_file = SettingsFile::new("snooze-timer-early");
    let (mut notch, _handles) = snooze_rig(&settings_file);
    notch.set_snooze(Snooze::Until(1_600), 1_000);
    notch.snooze_timer_fired(1_500);
    assert!(notch.is_snoozed());
    assert!(disarm_was_armed(&notch, SNOOZE_TIMER_ID));
}

#[test]
fn a_snooze_until_turned_off_arms_no_timer() {
    let settings_file = SettingsFile::new("snooze-manual");
    let (mut notch, _handles) = snooze_rig(&settings_file);
    notch.set_snooze(Snooze::Manual, 1_000);
    assert!(!disarm_was_armed(&notch, SNOOZE_TIMER_ID));
}

#[test]
fn turning_the_snooze_off_stops_its_timer() {
    let settings_file = SettingsFile::new("snooze-off");
    let (mut notch, _handles) = snooze_rig(&settings_file);
    notch.set_snooze(Snooze::Until(1_600), 1_000);
    notch.set_snooze(Snooze::Off, 1_100);
    assert!(!notch.is_snoozed());
    assert!(!disarm_was_armed(&notch, SNOOZE_TIMER_ID));
}

#[test]
fn a_stale_stored_snooze_is_cleared_and_ignored() {
    let settings_file = SettingsFile::new("snooze-stale");
    let (mut notch, _handles) = snooze_rig(&settings_file);
    notch.settings.set_string(SNOOZED_UNTIL_KEY, "1000");
    notch.apply_stored_snooze(2_000);
    assert!(!notch.is_snoozed());
    assert_eq!(notch.settings.get_string(SNOOZED_UNTIL_KEY), Some(""));
}

#[test]
fn a_snooze_written_by_the_settings_window_applies_on_reload() {
    let settings_file = SettingsFile::new("snooze-reload");
    let (mut notch, _handles) = snooze_rig(&settings_file);
    let mut other_program = settings_file.load();
    other_program.set_string(SNOOZED_UNTIL_KEY, "manual");
    notch.reload_settings();
    assert_eq!(notch.snooze(), Snooze::Manual);
}

#[test]
fn a_snooze_until_turned_off_survives_a_restart() {
    let settings_file = SettingsFile::new("snooze-start");
    let mut settings = settings_file.load();
    settings.set_bool("notch.hide_windows_volume_popup", false);
    settings.set_bool("notch.hide_windows_notifications", false);
    settings.set_string(SNOOZED_UNTIL_KEY, "manual");
    let mut notch = configured_rig(SampleOptions::default(), settings, Vec::new());
    notch.start();
    assert_eq!(notch.snooze(), Snooze::Manual);
}
