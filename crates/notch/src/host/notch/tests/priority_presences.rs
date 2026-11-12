//! Volume and brightness (priority activities) over windows that hide the notch.

use super::*;

/// A stand-in for the volume or brightness activity: a priority activity whose presence a test sets.
struct PriorityLevel(Rc<Cell<Option<Presence>>>);

impl Activity for PriorityLevel {
    fn id(&self) -> &'static str {
        "priority_level"
    }

    fn is_priority(&self) -> bool {
        true
    }

    fn presence(&self) -> Option<Presence> {
        self.0.get()
    }

    fn draw(&self, _canvas: &Canvas, _area: Rect, _opacity: f32) {}
}

fn priority_rig() -> (Notch, Rc<Cell<Option<Presence>>>) {
    let presence = Rc::new(Cell::new(None));
    let mut notch = configured_rig(
        SampleOptions::default(),
        Settings::at(None),
        vec![Box::new(PriorityLevel(presence.clone())), Box::new(Mark::showing(None).0)],
    );
    for slot in &mut notch.activities {
        slot.is_started = true;
    }
    settle(&mut notch);
    (notch, presence)
}

const LEVEL: Presence = Presence::Transient { width: 150.0 };
const MUTED_BADGE: Presence = Presence::Badge { width: 16.0 };

#[test]
fn a_volume_change_shows_over_a_hidden_full_screen_window_and_the_window_hides_again_after_it() {
    let (mut notch, level) = priority_rig();
    notch.hiding.fullscreen_showing = Showing::Hidden;
    notch.set_screen_mode(ScreenMode::Fullscreen);
    assert!(notch.is_suppressed());
    level.set(Some(LEVEL));
    notch.flush();
    assert!(!notch.is_suppressed(), "the level shows although the word is hidden");
    assert!(!notch.is_hidden());
    assert_eq!(notch.springs.resting_width.position, notch.springs.resting_width.target, "no morph from the old pill");
    assert!(!notch.opens_on_hover(), "the pointer still follows the word");
    level.set(None);
    notch.flush();
    assert!(notch.is_suppressed(), "the window hides again when the level ends");
}

#[test]
fn with_windows_own_flyout_back_a_volume_change_follows_the_word() {
    let (mut notch, level) = priority_rig();
    notch.hiding.replaces_windows_flyout = false;
    notch.hiding.fullscreen_showing = Showing::Hidden;
    notch.set_screen_mode(ScreenMode::Fullscreen);
    level.set(Some(LEVEL));
    notch.flush();
    assert!(notch.is_suppressed());
    level.set(Some(MUTED_BADGE));
    notch.hiding.hides_when_idle = true;
    notch.set_screen_mode(ScreenMode::Normal);
    notch.timer_fired(WINDOW_REVEAL_TIMER_ID);
    notch.flush();
    assert!(notch.is_hidden(), "the muted badge is lasting state when the flag does nothing");
}

#[test]
fn a_volume_change_never_shows_over_a_screenshot_overlay() {
    let (mut notch, level) = priority_rig();
    notch.set_screen_mode(ScreenMode::Capture);
    level.set(Some(LEVEL));
    notch.flush();
    assert!(notch.is_suppressed());
}

#[test]
fn the_muted_badge_keeps_the_pill_up_over_every_hiding_word_and_with_hide_when_idle() {
    let (mut notch, level) = priority_rig();
    level.set(Some(MUTED_BADGE));
    notch.flush();
    for (mode, showing) in super::hiding_rules::hiding_in_every_mode(&mut notch) {
        notch.hiding.maximized_showing = showing;
        notch.hiding.fullscreen_showing = showing;
        notch.hiding.screen_mode = mode;
        assert!(!notch.is_hidden(), "muted over {mode:?} with {showing:?}");
    }
    notch.hiding.fullscreen_showing = Showing::Hidden;
    assert!(!notch.is_suppressed(), "muted over a hidden full-screen window");
    level.set(None);
    notch.flush();
    assert!(notch.is_suppressed());
}

#[test]
fn the_windows_volume_popup_switch_decides_whether_priority_activities_override_the_word() {
    let file = SettingsFile::new("priority-flag");
    let mut settings = file.load();
    assert!(HidingState::read(&settings).replaces_windows_flyout, "on by default");
    let mut hiding = HidingState::read(&settings);
    settings.set_bool("notch.hide_windows_volume_popup", false);
    hiding.reload(&settings);
    assert!(!hiding.replaces_windows_flyout);
}

#[test]
fn an_ordinary_activity_with_a_lasting_badge_keeps_a_hidden_full_screen_window_suppressed() {
    let mut notch = marked_rig(SampleOptions::default(), vec![Mark::showing(Some(MUTED_BADGE)).0]);
    notch.hiding.fullscreen_showing = Showing::Hidden;
    notch.set_screen_mode(ScreenMode::Fullscreen);
    notch.flush();
    assert!(!notch.presences.has_priority);
    assert!(notch.is_suppressed());
}

#[test]
fn a_switched_off_priority_activity_does_not_override_the_word() {
    let (mut notch, level) = priority_rig();
    notch.activities[0].is_enabled = false;
    notch.hiding.fullscreen_showing = Showing::Hidden;
    notch.set_screen_mode(ScreenMode::Fullscreen);
    level.set(Some(LEVEL));
    notch.flush();
    assert!(notch.is_suppressed());
}

#[test]
fn a_hidden_full_screen_window_coming_to_the_front_while_muted_raises_the_notch_above_it() {
    let (mut notch, level) = priority_rig();
    notch.hiding.fullscreen_showing = Showing::Hidden;
    assert!(!notch.stays_on_screen_in(ScreenMode::Fullscreen), "with nothing up the notch steps away");
    level.set(Some(MUTED_BADGE));
    notch.flush();
    assert!(notch.stays_on_screen_in(ScreenMode::Fullscreen), "the muted badge needs the notch above the game");
    notch.set_screen_mode(ScreenMode::Fullscreen);
    assert!(!notch.is_suppressed());
    notch.timer_fired(RAISE_TIMER_ID);
    assert!(!notch.is_suppressed());
}
