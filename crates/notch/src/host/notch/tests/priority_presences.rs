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
    assert!(!notch.opens_on_pointer(), "the pointer still follows the word");
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

/// A stand-in for a notification: a banner that goes away for good when the host dismisses it.
struct DismissibleBanner(Rc<Cell<bool>>);

impl Activity for DismissibleBanner {
    fn id(&self) -> &'static str {
        "dismissible_banner"
    }

    fn presence(&self) -> Option<Presence> {
        self.0.get().then_some(Presence::Banner { width: 340.0, height: 60.0 })
    }

    fn dismiss_banner(&mut self, _context: &mut Context) {
        self.0.set(false);
    }

    fn draw(&self, _canvas: &Canvas, _area: Rect, _opacity: f32) {}
}

type Shared = Rc<Cell<Option<Presence>>>;

/// The notch, the level, whether the banner is up and an ordinary activity's presence.
fn banner_rig() -> (Notch, Shared, Rc<Cell<bool>>, Shared) {
    let level = Rc::new(Cell::new(None));
    let banner = Rc::new(Cell::new(true));
    let (ordinary, ordinary_presence) = Mark::showing(None);
    let mut notch = configured_rig(
        SampleOptions::default(),
        Settings::at(None),
        vec![Box::new(PriorityLevel(level.clone())), Box::new(DismissibleBanner(banner.clone())), Box::new(ordinary)],
    );
    for slot in &mut notch.activities {
        slot.is_started = true;
    }
    settle(&mut notch);
    notch.flush();
    (notch, level, banner, ordinary_presence)
}

#[test]
fn a_priority_presence_brings_down_a_showing_banner_and_the_banner_does_not_return() {
    let (mut notch, level, banner, _) = banner_rig();
    assert!(notch.presences.banner.is_some(), "the banner shows first");
    level.set(Some(LEVEL));
    notch.flush();
    assert!(notch.presences.banner.is_none(), "the banner is brought down at once");
    assert_eq!(notch.presences.transients.len(), 1, "the level shows in its place");
    assert!(!banner.get(), "the activity dropped its banner");
    level.set(None);
    notch.flush();
    assert!(notch.presences.banner.is_none(), "the banner does not return after the level ends");
}

#[test]
fn a_normal_transient_still_waits_behind_a_banner() {
    let (mut notch, _, banner, ordinary) = banner_rig();
    ordinary.set(Some(Presence::Transient { width: 100.0 }));
    notch.flush();
    assert!(notch.presences.banner.is_some());
    assert!(notch.presences.transients.is_empty(), "the transient waits");
    assert!(banner.get(), "the banner stays up");
}

#[test]
fn a_muted_badge_does_not_bring_down_a_banner() {
    let (mut notch, level, banner, _) = banner_rig();
    level.set(Some(MUTED_BADGE));
    notch.flush();
    assert!(notch.presences.banner.is_some());
    assert!(banner.get());
}

#[test]
fn a_priority_level_brings_down_a_download_banner_and_the_running_badge_stays() {
    let level = Rc::new(Cell::new(None));
    let fonts = Fonts::new(SCALE).unwrap();
    let mut notch = configured_rig(
        SampleOptions::default(),
        Settings::at(None),
        vec![Box::new(PriorityLevel(level.clone())), crate::activities::sample_download_started(&fonts)],
    );
    for slot in &mut notch.activities {
        slot.is_started = true;
    }
    settle(&mut notch);
    notch.flush();
    assert!(notch.presences.banner.is_some(), "the start banner shows first");
    level.set(Some(LEVEL));
    notch.flush();
    assert!(notch.presences.banner.is_none(), "the volume level brings the download banner down");
    level.set(None);
    notch.flush();
    assert!(notch.presences.banner.is_none(), "the banner does not return after the level ends");
    assert_eq!(notch.activities[1].activity.presence(), Some(Presence::Badge { width: Icon::Download.size() }));
}
