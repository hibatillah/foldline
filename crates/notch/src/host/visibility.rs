use super::{screen_mode::ScreenMode, settings::Settings};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Showing {
    Hidden,
    ActivitiesOnly,
    MusicWithLyrics,
    Music,
}

/// Matches the old defaults: lyrics stay visible over maximized windows, full-screen windows hide the notch.
pub(super) const DEFAULT_MAXIMIZED: Showing = Showing::MusicWithLyrics;
pub(super) const DEFAULT_FULLSCREEN: Showing = Showing::Hidden;

pub(super) const MAXIMIZED_KEY: &str = "notch.maximized";
pub(super) const FULLSCREEN_KEY: &str = "notch.fullscreen";
pub(super) const HOVER_MAXIMIZED_KEY: &str = "notch.hover_maximized";
pub(super) const HOVER_FULLSCREEN_KEY: &str = "notch.hover_fullscreen";

const OLD_HIDE_FOR_FULLSCREEN_KEY: &str = "notch.hide_for_fullscreen";
const OLD_SHOW_OVER_MAXIMIZED_KEY: &str = "notch.show_over_maximized";
const OLD_LYRICS_OVER_MAXIMIZED_KEY: &str = "notch.lyrics_over_maximized";

impl Showing {
    pub fn parse(text: &str) -> Option<Showing> {
        match text {
            "hidden" => Some(Showing::Hidden),
            "activities" => Some(Showing::ActivitiesOnly),
            "lyrics" => Some(Showing::MusicWithLyrics),
            "music" => Some(Showing::Music),
            _ => None,
        }
    }

    pub fn as_text(self) -> &'static str {
        match self {
            Showing::Hidden => "hidden",
            Showing::ActivitiesOnly => "activities",
            Showing::MusicWithLyrics => "lyrics",
            Showing::Music => "music",
        }
    }
}

pub(super) fn showing_for(mode: ScreenMode, maximized: Showing, fullscreen: Showing) -> Option<Showing> {
    match mode {
        ScreenMode::Normal => None,
        ScreenMode::Maximized => Some(maximized),
        ScreenMode::Fullscreen => Some(fullscreen),
        // A screenshot must never catch the notch, so no setting can show it over the overlay.
        ScreenMode::Capture => Some(Showing::Hidden),
    }
}

/// How much a showing lets through; Music and a regular window (`None`) both keep the pill up.
fn openness(showing: Option<Showing>) -> u8 {
    match showing {
        Some(Showing::Hidden) => 0,
        Some(Showing::ActivitiesOnly) => 1,
        Some(Showing::MusicWithLyrics) => 2,
        Some(Showing::Music) | None => 3,
    }
}

/// Whether going from `from` to `to` may show what was hidden; such a change waits, a hiding one applies at once.
pub(super) fn reveals_more(from: Option<Showing>, to: Option<Showing>) -> bool {
    openness(to) > openness(from)
}

/// Whether the pointer may open the notch; a regular window always allows it.
pub(super) fn opens_on_pointer(mode: ScreenMode, over_maximized: bool, over_fullscreen: bool) -> bool {
    match mode {
        ScreenMode::Normal => true,
        ScreenMode::Maximized => over_maximized,
        ScreenMode::Fullscreen => over_fullscreen,
        ScreenMode::Capture => false,
    }
}

/// Writes a new key only when it is absent and an old key exists; with no old key the default already matches.
pub(super) fn migrate(settings: &mut Settings) {
    if settings.get_string(FULLSCREEN_KEY).is_none() && settings.get_string(OLD_HIDE_FOR_FULLSCREEN_KEY).is_some() {
        let hides = settings.get_bool(OLD_HIDE_FOR_FULLSCREEN_KEY, true);
        settings.set_string(FULLSCREEN_KEY, if hides { Showing::Hidden } else { Showing::Music }.as_text());
    }
    let has_old_maximized_key = settings.get_string(OLD_SHOW_OVER_MAXIMIZED_KEY).is_some()
        || settings.get_string(OLD_LYRICS_OVER_MAXIMIZED_KEY).is_some();
    if settings.get_string(MAXIMIZED_KEY).is_none() && has_old_maximized_key {
        let showing = if settings.get_bool(OLD_SHOW_OVER_MAXIMIZED_KEY, false) {
            Showing::Music
        } else if settings.get_bool(OLD_LYRICS_OVER_MAXIMIZED_KEY, true) {
            Showing::MusicWithLyrics
        } else {
            Showing::ActivitiesOnly
        };
        settings.set_string(MAXIMIZED_KEY, showing.as_text());
    }
}

/// An activity (a notification banner, a volume change) shows in every mode except Hidden; Music keeps the pill up
/// even with nothing playing, so the idle clock stays visible.
fn allows(showing: Showing, has_activity: bool, is_playing: bool, has_lyrics: bool) -> bool {
    match showing {
        Showing::Hidden => false,
        Showing::ActivitiesOnly => has_activity,
        Showing::MusicWithLyrics => has_activity || (is_playing && has_lyrics),
        Showing::Music => true,
    }
}

/// What the collapsed notch has to show besides a priority activity.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Collapsed {
    /// A short message or a banner; a lasting mark (muted, Caps Lock, a privacy dot) is state and does not count.
    pub(super) has_message: bool,
    pub(super) has_resting_content: bool,
    pub(super) has_lyrics: bool,
}

/// Whether a priority activity (volume, brightness) shows where the word or hide-when-idle would hide it: only while
/// the notch replaces Windows' own flyout, and never over a screenshot overlay.
pub(super) fn priority_shows(mode: ScreenMode, replaces_windows_flyout: bool, has_priority_presence: bool) -> bool {
    has_priority_presence && replaces_windows_flyout && mode != ScreenMode::Capture
}

/// Whether the user's pin shows the notch where the word or hide-when-idle would hide it; never over a screenshot
/// overlay, so a screenshot never catches a pinned notch.
pub(super) fn pin_shows(mode: ScreenMode, is_pinned: bool) -> bool {
    is_pinned && mode != ScreenMode::Capture
}

/// Whether the window goes off screen: the word `hidden` or a screenshot overlay, unless a priority activity or the
/// pin shows it.
pub(super) fn suppresses(showing: Option<Showing>, priority_shows: bool, pin_shows: bool) -> bool {
    showing == Some(Showing::Hidden) && !priority_shows && !pin_shows
}

/// Whether the notch stays on screen (is not suppressed) in front of a window in `mode`.
pub(super) fn stays_on_screen(
    mode: ScreenMode,
    maximized: Showing,
    fullscreen: Showing,
    replaces_windows_flyout: bool,
    has_priority_presence: bool,
    is_pinned: bool,
) -> bool {
    let priority = priority_shows(mode, replaces_windows_flyout, has_priority_presence);
    !suppresses(showing_for(mode, maximized, fullscreen), priority, pin_shows(mode, is_pinned))
}

/// Whether the collapsed notch shrinks to the strip; `None` is a regular window, where hide-when-idle decides.
pub(super) fn hides_to_strip(
    showing: Option<Showing>,
    hides_when_idle: bool,
    collapsed: Collapsed,
    priority_shows: bool,
    pin_shows: bool,
) -> bool {
    if priority_shows || pin_shows {
        return false;
    }
    match showing {
        None => hides_when_idle && !collapsed.has_message && !collapsed.has_resting_content,
        // The suppressed window draws nothing; a full-size layout avoids a strip growing back when it returns.
        Some(Showing::Hidden) => false,
        Some(showing) => !allows(showing, collapsed.has_message, collapsed.has_resting_content, collapsed.has_lyrics),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings_with(contents: &str) -> (Settings, std::path::PathBuf) {
        static COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("notch-visibility-{}-{id}.txt", std::process::id()));
        std::fs::write(&path, contents).unwrap();
        (Settings::at(Some(path.clone())), path)
    }

    #[test]
    fn parse_rejects_unknown_words_and_the_default_applies() {
        assert_eq!(Showing::parse("sometimes"), None);
        assert_eq!(Showing::parse(""), None);
        assert_eq!(Showing::parse("Music"), None);
        assert_eq!(Showing::parse(" music"), None);
        assert_eq!(Showing::parse("bogus").unwrap_or(DEFAULT_MAXIMIZED), Showing::MusicWithLyrics);
        assert_eq!(Showing::parse("bogus").unwrap_or(DEFAULT_FULLSCREEN), Showing::Hidden);
    }

    #[test]
    fn every_showing_round_trips_through_its_text() {
        for showing in [Showing::Hidden, Showing::ActivitiesOnly, Showing::MusicWithLyrics, Showing::Music] {
            assert_eq!(Showing::parse(showing.as_text()), Some(showing));
        }
        assert_eq!(Showing::parse("hidden"), Some(Showing::Hidden));
        assert_eq!(Showing::parse("activities"), Some(Showing::ActivitiesOnly));
        assert_eq!(Showing::parse("lyrics"), Some(Showing::MusicWithLyrics));
        assert_eq!(Showing::parse("music"), Some(Showing::Music));
    }

    #[test]
    fn a_normal_window_has_no_showing_override() {
        assert_eq!(showing_for(ScreenMode::Normal, Showing::Music, Showing::Music), None);
        assert_eq!(showing_for(ScreenMode::Maximized, Showing::Music, Showing::Hidden), Some(Showing::Music));
        assert_eq!(showing_for(ScreenMode::Fullscreen, Showing::Music, Showing::Hidden), Some(Showing::Hidden));
    }

    #[test]
    fn pointer_opening_follows_the_switch_for_the_window_in_front_and_always_opens_over_a_regular_window() {
        assert!(opens_on_pointer(ScreenMode::Normal, false, false));
        assert!(!opens_on_pointer(ScreenMode::Maximized, false, true));
        assert!(opens_on_pointer(ScreenMode::Maximized, true, false));
        assert!(!opens_on_pointer(ScreenMode::Fullscreen, true, false));
        assert!(opens_on_pointer(ScreenMode::Fullscreen, false, true));
    }

    #[test]
    fn a_screenshot_overlay_hides_the_notch_and_ignores_the_pointer_whatever_the_settings_say() {
        assert_eq!(showing_for(ScreenMode::Capture, Showing::Music, Showing::Music), Some(Showing::Hidden));
        assert!(!opens_on_pointer(ScreenMode::Capture, true, true));
        assert!(reveals_more(showing_for(ScreenMode::Capture, Showing::Music, Showing::Music), None));
    }

    #[test]
    fn fullscreen_hide_on_migrates_to_hidden() {
        let (mut settings, path) = settings_with("notch.hide_for_fullscreen=on\n");
        migrate(&mut settings);
        assert_eq!(settings.get_string(FULLSCREEN_KEY), Some("hidden"));
        let (mut settings, path_off) = settings_with("notch.hide_for_fullscreen=off\n");
        migrate(&mut settings);
        assert_eq!(settings.get_string(FULLSCREEN_KEY), Some("music"));
        let _ = (std::fs::remove_file(path), std::fs::remove_file(path_off));
    }

    #[test]
    fn maximized_show_over_maximized_migrates_to_music() {
        let (mut settings, path) = settings_with("notch.show_over_maximized=on\nnotch.lyrics_over_maximized=off\n");
        migrate(&mut settings);
        assert_eq!(settings.get_string(MAXIMIZED_KEY), Some("music"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn maximized_defaults_migrate_to_lyrics() {
        let (mut settings, path) = settings_with("notch.show_over_maximized=off\n");
        migrate(&mut settings);
        assert_eq!(settings.get_string(MAXIMIZED_KEY), Some("lyrics"));
        let (mut settings, path_off) = settings_with("notch.lyrics_over_maximized=off\n");
        migrate(&mut settings);
        assert_eq!(settings.get_string(MAXIMIZED_KEY), Some("activities"));
        let _ = (std::fs::remove_file(path), std::fs::remove_file(path_off));
    }

    #[test]
    fn migration_never_overwrites_a_new_key() {
        let (mut settings, path) = settings_with(
            "notch.maximized=hidden\nnotch.fullscreen=music\nnotch.show_over_maximized=on\nnotch.hide_for_fullscreen=on\n",
        );
        migrate(&mut settings);
        migrate(&mut settings);
        assert_eq!(settings.get_string(MAXIMIZED_KEY), Some("hidden"));
        assert_eq!(settings.get_string(FULLSCREEN_KEY), Some("music"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn migration_writes_nothing_when_no_old_key_exists_and_is_idempotent() {
        let (mut settings, path) = settings_with("");
        migrate(&mut settings);
        assert_eq!(settings.get_string(MAXIMIZED_KEY), None);
        assert_eq!(settings.get_string(FULLSCREEN_KEY), None);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "");
        let _ = std::fs::remove_file(path);

        let (mut settings, path) = settings_with("notch.show_over_maximized=on\n");
        migrate(&mut settings);
        let once = std::fs::read_to_string(&path).unwrap();
        migrate(&mut settings);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), once);
        assert_eq!(settings.get_string(FULLSCREEN_KEY), None);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn music_with_lyrics_needs_lyrics_but_music_does_not() {
        assert!(allows(Showing::MusicWithLyrics, false, true, true));
        assert!(!allows(Showing::MusicWithLyrics, false, true, false));
        assert!(!allows(Showing::MusicWithLyrics, false, false, true));
        assert!(allows(Showing::MusicWithLyrics, true, false, false));
        assert!(allows(Showing::Music, false, true, false));
    }

    #[test]
    fn leaving_a_hiding_window_for_a_regular_one_reveals_more_but_coming_back_does_not() {
        for hiding in [Showing::Hidden, Showing::ActivitiesOnly, Showing::MusicWithLyrics] {
            assert!(reveals_more(Some(hiding), None), "{hiding:?} to a regular window");
            assert!(!reveals_more(None, Some(hiding)), "a regular window to {hiding:?}");
        }
        assert!(reveals_more(Some(Showing::Hidden), Some(Showing::ActivitiesOnly)));
        assert!(!reveals_more(Some(Showing::Music), None), "music already keeps the pill up");
        assert!(!reveals_more(None, None));
    }

    #[test]
    fn music_keeps_the_pill_up_even_with_nothing_playing() {
        assert!(allows(Showing::Music, false, false, false));
    }

    const EVERY_MODE: [ScreenMode; 4] =
        [ScreenMode::Normal, ScreenMode::Maximized, ScreenMode::Fullscreen, ScreenMode::Capture];
    const EVERY_WORD: [Showing; 4] =
        [Showing::Hidden, Showing::ActivitiesOnly, Showing::MusicWithLyrics, Showing::Music];

    /// Whether an idle, unpinned notch with hide-when-idle on is on screen at full size.
    fn is_shown(mode: ScreenMode, word: Showing, replaces_windows_flyout: bool, has_priority_presence: bool) -> bool {
        is_shown_with(mode, word, true, replaces_windows_flyout && has_priority_presence, false)
    }

    /// Whether an idle notch is on screen at full size, with the flyout replaced when `has_priority_presence`.
    fn is_shown_with(
        mode: ScreenMode,
        word: Showing,
        hides_when_idle: bool,
        has_priority_presence: bool,
        is_pinned: bool,
    ) -> bool {
        let showing = showing_for(mode, word, word);
        let priority = priority_shows(mode, true, has_priority_presence);
        let pinned = pin_shows(mode, is_pinned);
        let is_on_screen = stays_on_screen(mode, word, word, true, has_priority_presence, is_pinned);
        assert_eq!(is_on_screen, !suppresses(showing, priority, pinned), "the two suppression paths agree");
        is_on_screen && !hides_to_strip(showing, hides_when_idle, Collapsed::default(), priority, pinned)
    }

    #[test]
    fn a_pinned_notch_is_never_suppressed_over_any_word_in_any_mode_except_capture() {
        for mode in [ScreenMode::Normal, ScreenMode::Maximized, ScreenMode::Fullscreen] {
            for word in EVERY_WORD {
                for hides_when_idle in [false, true] {
                    assert!(
                        is_shown_with(mode, word, hides_when_idle, false, true),
                        "{word:?} over {mode:?}, hide when idle {hides_when_idle}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_pinned_notch_is_hidden_during_a_screenshot_overlay() {
        for word in EVERY_WORD {
            for has_priority_presence in [false, true] {
                assert!(!stays_on_screen(ScreenMode::Capture, word, word, true, has_priority_presence, true));
                assert!(!is_shown_with(ScreenMode::Capture, word, false, has_priority_presence, true));
            }
        }
        assert!(!pin_shows(ScreenMode::Capture, true));
    }

    #[test]
    fn an_unpinned_notch_keeps_its_old_rule() {
        for mode in EVERY_MODE {
            assert!(!pin_shows(mode, false), "{mode:?}");
            for word in EVERY_WORD {
                for hides_when_idle in [false, true] {
                    let showing = showing_for(mode, word, word);
                    let expected = !suppresses(showing, false, false)
                        && !(hides_when_idle && showing.is_none())
                        && (showing.is_none() || word == Showing::Music);
                    assert_eq!(
                        is_shown_with(mode, word, hides_when_idle, false, false),
                        expected,
                        "{word:?} over {mode:?}, hide when idle {hides_when_idle}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_priority_activity_shows_over_every_word_and_mode_except_a_screenshot_only_while_the_flyout_is_replaced() {
        for mode in EVERY_MODE {
            for word in EVERY_WORD {
                let shown_without_priority = is_shown(mode, word, true, false);
                assert_eq!(
                    is_shown(mode, word, true, true),
                    mode != ScreenMode::Capture,
                    "{word:?} over {mode:?} with the flyout replaced"
                );
                assert_eq!(
                    is_shown(mode, word, false, true),
                    shown_without_priority,
                    "{word:?} over {mode:?} with Windows' flyout: the word applies"
                );
                let expected_without_priority =
                    mode != ScreenMode::Capture && word == Showing::Music && mode != ScreenMode::Normal;
                assert_eq!(shown_without_priority, expected_without_priority, "{word:?} over {mode:?} idle");
            }
        }
    }

    #[test]
    fn hidden_suppresses_the_window_until_a_priority_activity_shows_but_the_screenshot_overlay_always_does() {
        for mode in [ScreenMode::Maximized, ScreenMode::Fullscreen] {
            let showing = showing_for(mode, Showing::Hidden, Showing::Hidden);
            assert!(suppresses(showing, priority_shows(mode, true, false), false));
            assert!(!suppresses(showing, priority_shows(mode, true, true), false));
            assert!(suppresses(showing, priority_shows(mode, false, true), false));
        }
        let capture = showing_for(ScreenMode::Capture, Showing::Music, Showing::Music);
        assert!(suppresses(capture, priority_shows(ScreenMode::Capture, true, true), false));
        assert!(!suppresses(None, false, false), "a regular window never suppresses");
    }

    #[test]
    fn the_notch_stays_on_screen_over_a_hidden_window_only_with_a_priority_presence_and_never_over_a_screenshot() {
        let hidden = Showing::Hidden;
        assert!(!stays_on_screen(ScreenMode::Fullscreen, Showing::Music, hidden, true, false, false));
        assert!(stays_on_screen(ScreenMode::Fullscreen, Showing::Music, hidden, true, true, false));
        assert!(!stays_on_screen(ScreenMode::Fullscreen, Showing::Music, hidden, false, true, false));
        assert!(stays_on_screen(ScreenMode::Maximized, hidden, hidden, true, true, false));
        assert!(stays_on_screen(ScreenMode::Fullscreen, Showing::Music, Showing::ActivitiesOnly, true, false, false));
        assert!(stays_on_screen(ScreenMode::Normal, hidden, hidden, false, false, false));
        assert!(!stays_on_screen(ScreenMode::Capture, Showing::Music, Showing::Music, true, true, false));
    }

    #[test]
    fn a_message_keeps_a_hiding_word_up_but_lasting_state_does_not() {
        let message = Collapsed { has_message: true, ..Collapsed::default() };
        let playing = Collapsed { has_resting_content: true, ..Collapsed::default() };
        assert!(!hides_to_strip(None, true, message, false, false));
        assert!(!hides_to_strip(None, true, playing, false, false));
        assert!(hides_to_strip(None, true, Collapsed::default(), false, false));
        assert!(!hides_to_strip(None, false, Collapsed::default(), false, false));
        assert!(!hides_to_strip(Some(Showing::ActivitiesOnly), false, message, false, false));
        assert!(hides_to_strip(Some(Showing::ActivitiesOnly), false, playing, false, false));
        assert!(!hides_to_strip(Some(Showing::Hidden), true, Collapsed::default(), false, false));
    }

    #[test]
    fn activities_only_shows_only_with_an_activity() {
        assert!(allows(Showing::ActivitiesOnly, true, false, false));
        assert!(!allows(Showing::ActivitiesOnly, false, true, true));
        assert!(!allows(Showing::Hidden, true, true, true));
    }
}
