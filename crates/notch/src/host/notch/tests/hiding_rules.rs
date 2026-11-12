//! What the notch shows over maximized and full-screen windows and while idle.

use super::*;
use crate::{gfx::Point, page::PointerEvent};

#[test]
fn a_notch_with_nothing_to_do_hides_over_a_maximized_window_and_shows_otherwise() {
    let mut notch = rig(SampleOptions::default());
    assert!(!notch.is_hidden());
    notch.hiding.screen_mode = ScreenMode::Maximized;
    assert!(notch.is_hidden());
    notch.hiding.maximized_showing = Showing::Music;
    assert!(!notch.is_hidden(), "the music choice keeps even the idle clock up");
}

#[test]
fn music_in_the_pill_does_not_keep_the_notch_up_over_a_maximized_window() {
    let mut notch = playing(false);
    notch.refresh_layout();
    assert!(notch.has_resting_content);
    notch.hiding.screen_mode = ScreenMode::Maximized;
    assert!(notch.is_hidden());
}

#[test]
fn lyrics_stay_over_a_maximized_window_only_when_the_user_asked() {
    let mut notch = playing(true);
    notch.hiding.maximized_showing = Showing::ActivitiesOnly;
    notch.refresh_layout();
    notch.hiding.screen_mode = ScreenMode::Maximized;
    assert!(notch.is_hidden());
    notch.hiding.maximized_showing = Showing::MusicWithLyrics;
    assert!(!notch.is_hidden());
}

#[test]
fn maximized_lyrics_hides_music_without_lyrics() {
    let mut without_lyrics = playing(false);
    without_lyrics.hiding.maximized_showing = Showing::MusicWithLyrics;
    without_lyrics.hiding.screen_mode = ScreenMode::Maximized;
    without_lyrics.refresh_layout();
    assert!(without_lyrics.is_hidden());
    assert_eq!(without_lyrics.springs.resting_height.target, without_lyrics.hidden_height());
    assert!(!without_lyrics.is_suppressed());
}

#[test]
fn maximized_music_shows_without_lyrics() {
    let mut notch = playing(false);
    notch.hiding.maximized_showing = Showing::Music;
    notch.hiding.hides_when_idle = true;
    notch.hiding.screen_mode = ScreenMode::Maximized;
    notch.refresh_layout();
    assert!(notch.has_resting_content);
    assert!(!notch.is_hidden());
    assert!(!notch.is_suppressed());
}

#[test]
fn fullscreen_activities_only_shows_a_banner_but_hides_the_idle_clock() {
    let mut notch = rig(SampleOptions::default());
    notch.hiding.fullscreen_showing = Showing::ActivitiesOnly;
    notch.set_screen_mode(ScreenMode::Fullscreen);
    assert!(!notch.is_suppressed());
    assert!(notch.is_hidden(), "the idle clock hides to the strip");
    notch.presences.banner = Some((Source::Activity(0), 330.0, 70.0));
    assert!(!notch.is_hidden());
    let mut playing_notch = playing(true);
    playing_notch.hiding.fullscreen_showing = Showing::ActivitiesOnly;
    playing_notch.hiding.screen_mode = ScreenMode::Fullscreen;
    playing_notch.refresh_layout();
    assert!(playing_notch.is_hidden(), "music and lyrics do not count as activities");
}

#[test]
fn leaving_full_screen_with_default_settings_keeps_the_pill_at_full_size_without_animating() {
    let mut notch = rig(SampleOptions::default());
    settle(&mut notch);
    assert_eq!(notch.springs.resting_height.target, RESTING_PILL_HEIGHT);
    // Settling snaps the springs but leaves the rig's start-up frame timer running.
    notch.stop_animation();
    notch.set_screen_mode(ScreenMode::Fullscreen);
    assert!(!notch.frame_timer.is_running, "entering full screen starts no strip animation");
    assert!(notch.is_suppressed());
    assert!(!notch.is_hidden(), "a suppressed notch keeps its full-size layout");
    assert_eq!(notch.springs.resting_height.target, RESTING_PILL_HEIGHT);
    notch.set_screen_mode(ScreenMode::Normal);
    notch.timer_fired(WINDOW_REVEAL_TIMER_ID);
    assert_eq!(notch.springs.resting_height.target, RESTING_PILL_HEIGHT);
    assert!(!notch.frame_timer.is_running);
}

#[test]
fn fullscreen_music_keeps_the_idle_clock_up_even_with_hide_when_idle_on() {
    let mut notch = rig(SampleOptions::default());
    notch.hiding.hides_when_idle = true;
    notch.hiding.fullscreen_showing = Showing::Music;
    notch.hiding.screen_mode = ScreenMode::Fullscreen;
    notch.refresh_layout();
    assert!(!notch.has_resting_content);
    assert!(!notch.is_hidden());
}

#[test]
fn fullscreen_hidden_suppresses_the_window() {
    let mut notch = rig(SampleOptions::default());
    notch.hiding.fullscreen_showing = Showing::Hidden;
    notch.set_screen_mode(ScreenMode::Fullscreen);
    assert!(notch.is_suppressed());
    notch.presences.banner = Some((Source::Activity(0), 330.0, 70.0));
    assert!(notch.is_suppressed(), "hidden hides even an activity");
    notch.set_screen_mode(ScreenMode::Normal);
    notch.timer_fired(WINDOW_REVEAL_TIMER_ID);
    assert!(!notch.is_suppressed());
    notch.hiding.maximized_showing = Showing::Hidden;
    notch.set_screen_mode(ScreenMode::Maximized);
    assert!(notch.is_suppressed(), "hidden over a maximized window suppresses too");
}

#[test]
fn a_screenshot_overlay_hides_everything_and_hover_never_opens_the_notch() {
    let mut notch = rig(SampleOptions::default());
    notch.hiding.maximized_showing = Showing::Music;
    notch.hiding.fullscreen_showing = Showing::Music;
    notch.hiding.opens_on_hover_over_maximized = true;
    notch.hiding.opens_on_hover_over_fullscreen = true;
    notch.set_screen_mode(ScreenMode::Capture);
    assert!(notch.is_suppressed());
    notch.presences.banner = Some((Source::Activity(0), 330.0, 70.0));
    assert!(notch.is_suppressed(), "a banner stays hidden over the overlay");
    assert!(!notch.opens_on_hover());
    notch.set_hovering(true);
    notch.timer_fired(REVEAL_TIMER_ID);
    notch.timer_fired(EXPAND_TIMER_ID);
    assert_eq!(notch.springs.expansion.target, 0.0, "pointing at the top edge does not open the notch");
}

#[test]
fn maximized_window_hiding_takes_precedence_over_hide_when_idle() {
    for lyrics_enabled in [false, true] {
        for hides_when_idle in [false, true] {
            for keeps_lyrics_over_windows in [false, true] {
                let mut notch = playing(lyrics_enabled);
                notch.hiding.hides_when_idle = hides_when_idle;
                notch.hiding.maximized_showing =
                    if keeps_lyrics_over_windows { Showing::MusicWithLyrics } else { Showing::ActivitiesOnly };
                notch.hiding.screen_mode = ScreenMode::Maximized;
                notch.refresh_layout();
                let should_hide = !(lyrics_enabled && keeps_lyrics_over_windows);
                assert_eq!(
                    notch.is_hidden(),
                    should_hide,
                    "lyrics={lyrics_enabled}, hide_idle={hides_when_idle}, keep_lyrics={keeps_lyrics_over_windows}"
                );
                if should_hide {
                    assert_eq!(notch.springs.resting_width.target, HIDDEN_WIDTH);
                    assert_eq!(notch.springs.resting_height.target, notch.hidden_height());
                }
                notch.hiding.screen_mode = ScreenMode::Normal;
                notch.refresh_layout();
                assert!(!notch.is_hidden(), "music remains visible outside maximized windows");
            }
        }
    }
}

#[test]
fn maximized_music_keeps_music_visible_regardless_of_lyrics_and_idle_settings() {
    for lyrics_enabled in [false, true] {
        for hides_when_idle in [false, true] {
            let mut notch = playing(lyrics_enabled);
            notch.hiding.hides_when_idle = hides_when_idle;
            notch.hiding.maximized_showing = Showing::Music;
            notch.hiding.screen_mode = ScreenMode::Maximized;
            notch.refresh_layout();
            assert!(!notch.is_hidden());
            notch.hiding.screen_mode = ScreenMode::Fullscreen;
            assert!(notch.is_suppressed());
        }
    }
}

#[test]
fn pointing_at_a_collapsed_banner_reaches_its_activity_and_makes_room_for_the_buttons() {
    let fonts = Fonts::new(SCALE).unwrap();
    let body = "Deploy is green. Can you check the hover state on the 1440p monitor before we tag it? ".repeat(3);
    let activity = crate::activities::notifications::NotificationsActivity::sample_with_body(&fonts, &body);
    let mut notch = configured_rig(SampleOptions::default(), Settings::at(None), vec![Box::new(activity)]);
    notch.activities[0].is_started = true;
    settle(&mut notch);
    let (calm_width, calm_height) = (notch.springs.resting_width.target, notch.springs.resting_height.target);
    let (_, area) = notch.banner_area(&notch.frame()).expect("the banner shows");
    notch.set_hovering(true);
    notch.route_pointer(PointerEvent::Moved(Point::new(area.center_x(), area.center_y())));
    notch.flush();
    assert_eq!(notch.springs.resting_width.target, calm_width);
    assert!(
        notch.springs.resting_height.target > calm_height,
        "{} vs {}",
        notch.springs.resting_height.target,
        calm_height
    );
}

#[test]
fn a_banner_or_a_short_message_still_shows_over_a_maximized_window() {
    let mut notch = rig(SampleOptions::default());
    notch.hiding.screen_mode = ScreenMode::Maximized;
    notch.presences.banner = Some((Source::Activity(0), 330.0, 70.0));
    assert!(!notch.is_hidden());
    notch.presences.banner = None;
    notch.presences.transients = vec![(Source::Activity(0), 100.0)];
    assert!(!notch.is_hidden());
}

/// A hiding notch over each kind of window: a regular one with Hide when idle, then maximized and full screen.
pub(super) fn hiding_in_every_mode(notch: &mut Notch) -> Vec<(ScreenMode, Showing)> {
    notch.hiding.hides_when_idle = true;
    let mut modes = vec![(ScreenMode::Normal, Showing::ActivitiesOnly)];
    for showing in [Showing::ActivitiesOnly, Showing::MusicWithLyrics] {
        modes.push((ScreenMode::Maximized, showing));
        modes.push((ScreenMode::Fullscreen, showing));
    }
    modes
}

#[test]
fn an_ordinary_badge_or_a_privacy_dot_does_not_keep_the_notch_up_over_any_window() {
    let mut notch = rig(SampleOptions::default());
    notch.presences.badges = vec![(Source::Activity(0), 16.0)];
    notch.resting_row.dots = vec![(Source::Activity(1), 6.0)];
    for (mode, showing) in hiding_in_every_mode(&mut notch) {
        notch.hiding.maximized_showing = showing;
        notch.hiding.fullscreen_showing = showing;
        notch.hiding.screen_mode = mode;
        assert!(notch.is_hidden(), "{showing:?} over {mode:?}");
    }
}

#[test]
fn a_short_message_or_a_banner_shows_a_hiding_notch_over_any_window() {
    let mut notch = rig(SampleOptions::default());
    notch.presences.badges = vec![(Source::Activity(0), 16.0)];
    for (mode, showing) in hiding_in_every_mode(&mut notch) {
        notch.hiding.maximized_showing = showing;
        notch.hiding.fullscreen_showing = showing;
        notch.hiding.screen_mode = mode;
        notch.presences.transients = vec![(Source::Activity(0), 60.0)];
        assert!(!notch.is_hidden(), "a message over {mode:?} with {showing:?}");
        notch.presences.transients.clear();
        notch.presences.banner = Some((Source::Activity(0), 330.0, 70.0));
        assert!(!notch.is_hidden(), "a banner over {mode:?} with {showing:?}");
        notch.presences.banner = None;
        assert!(notch.is_hidden(), "the notch hides again over {mode:?} once the message ends");
    }
}

#[test]
fn a_full_screen_window_takes_the_notch_off_screen_and_nothing_else_does() {
    let mut notch = rig(SampleOptions::default());
    notch.hiding.screen_mode = ScreenMode::Fullscreen;
    assert!(notch.is_suppressed());
    for showing in [Showing::ActivitiesOnly, Showing::MusicWithLyrics, Showing::Music] {
        notch.hiding.fullscreen_showing = showing;
        assert!(!notch.is_suppressed(), "{showing:?}");
    }
    notch.hiding.fullscreen_showing = Showing::Hidden;
    notch.hiding.screen_mode = ScreenMode::Maximized;
    assert!(!notch.is_suppressed());
    notch.hiding.screen_mode = ScreenMode::Normal;
    notch.hiding.maximized_showing = Showing::Hidden;
    assert!(!notch.is_suppressed(), "a regular window never suppresses the notch");
}

#[test]
fn over_a_maximized_window_the_strip_is_thinner_and_slower_to_wake_than_with_hide_when_idle() {
    let mut notch = rig(SampleOptions::default());
    notch.hiding.hides_when_idle = true;
    let (strip, delay) = (notch.hidden_height(), notch.reveal_delay());
    notch.hiding.hides_when_idle = false;
    assert!(notch.hidden_height() < strip);
    assert!(notch.reveal_delay() > delay);
}
