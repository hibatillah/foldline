//! What the media page shows collapsed and open.

use crate::page::{Mode, Page, Priority};
use lyrics::LyricLine;
use session::LyricsStatus;

use super::*;

use std::sync::Arc;

fn playing_page() -> MediaPage {
    let mut page = MediaPage::new();
    page.info.available = true;
    page.info.is_playing = true;
    page.info.title = "Song".to_owned();
    page
}

fn with_lyrics(mut page: MediaPage) -> MediaPage {
    let lines = vec![LyricLine { start_ticks: 0, text: "la".to_owned() }];
    page.info.lyrics = LyricsStatus::Found(Arc::new(lyrics::Lyrics { lines }));
    page
}

#[test]
fn with_no_music_the_page_stays_out_of_the_way() {
    let page = MediaPage::new();
    assert_eq!(page.priority(Mode::Resting), None);
    assert_eq!(page.priority(Mode::Expanded), Some(Priority::BACKGROUND));
}

#[test]
fn with_nothing_playing_the_open_player_asks_for_a_song() {
    let mut page = MediaPage::new();
    assert_eq!(page.displayed_title(), NOTHING_PLAYING_TITLE);
    assert_eq!(page.displayed_detail(), NOTHING_PLAYING_DETAIL);
    page.info.available = true;
    page.info.title = "Song".to_owned();
    page.info.artist = "Artist".to_owned();
    assert_eq!((page.displayed_title(), page.displayed_detail()), ("Song", "Artist"));
    page.info.title.clear();
    assert_eq!(page.displayed_title(), UNKNOWN_TITLE);
}

#[test]
fn playing_music_is_shown_collapsed_and_open() {
    let page = playing_page();
    assert_eq!(page.priority(Mode::Resting), Some(Priority::NORMAL));
    assert_eq!(page.priority(Mode::Expanded), Some(Priority::NORMAL));
}

#[test]
fn paused_music_leaves_the_collapsed_pill_but_stays_up_when_open() {
    let mut page = playing_page();
    page.info.is_playing = false;
    assert_eq!(page.priority(Mode::Resting), None);
    assert_eq!(page.priority(Mode::Expanded), Some(Priority::NORMAL));
}

#[test]
fn the_open_player_widens_only_while_lyrics_are_shown() {
    let plain = playing_page().sizes().expanded;
    let mut page = with_lyrics(playing_page());
    assert_eq!(page.sizes().expanded.width, layout::LYRICS_WIDTH);
    assert!(page.sizes().expanded.width > plain.width);

    page.lyrics_enabled = false;
    assert_eq!(page.sizes().expanded.width, plain.width);
    assert_eq!(page.sizes().expanded.height, plain.height);
}

#[test]
fn lyrics_that_were_not_found_leave_the_layout_alone() {
    let mut page = playing_page();
    page.info.lyrics = LyricsStatus::NotFound;
    assert!(!page.is_lyric_panel_visible());
    assert!(!page.is_lyric_mode());
    assert_eq!(page.sizes().expanded.width, layout::plain_width(layout::TEXT_MIN_WIDTH));
}

#[test]
fn the_collapsed_pill_shows_lyrics_only_while_playing() {
    let mut page = with_lyrics(playing_page());
    assert!(page.is_lyric_mode());
    page.info.is_playing = false;
    assert!(!page.is_lyric_mode());
    assert!(page.is_lyric_panel_visible(), "the open view still shows them while paused");
}

#[test]
fn the_collapsed_pill_shows_still_bars_by_default_and_the_title_after_a_track_change() {
    let mut page = playing_page();
    assert_eq!(page.resting_view(), resting::View::Wave);
    page.is_title_shown = true;
    assert_eq!(page.resting_view(), resting::View::TrackChange);
    assert!(page.sizes().resting_width >= 150.0);
}

#[test]
fn the_header_has_the_wave_whenever_music_plays_whatever_the_lyrics_do() {
    let mut page = playing_page();
    assert_eq!(page.header_status_width(), Some(wave::WIDTH));
    page.lyrics_enabled = true;
    assert_eq!(page.header_status_width(), Some(wave::WIDTH));
    let page = with_lyrics(page);
    assert_eq!(page.header_status_width(), Some(wave::WIDTH));

    let mut paused = playing_page();
    paused.info.is_playing = false;
    assert_eq!(paused.header_status_width(), None, "paused music has nothing moving");
}

#[test]
fn the_collapsed_pill_has_the_wave_only_while_the_lyrics_are_off() {
    let mut page = playing_page();
    assert!(page.is_wave_visible());
    page = with_lyrics(page);
    assert!(!page.is_wave_visible(), "lyric mode shows the lyrics in the pill");
    page.is_hovered = true;
    assert!(page.is_wave_visible(), "the open notch has the wave in its header");
}

#[test]
fn a_rest_in_the_lyrics_keeps_the_collapsed_pill_in_lyric_mode() {
    let page = with_lyrics(playing_page());
    assert!(page.follower.current().is_empty());
    assert_eq!(page.resting_view(), resting::View::Lyrics);
}

#[test]
fn the_collapsed_pill_with_a_short_title_is_small() {
    assert_eq!(playing_page().sizes().resting_width, crate::page::RESTING_MIN_WIDTH);
}
