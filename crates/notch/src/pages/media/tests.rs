use crate::{
    activity::Presence,
    page::{Mode, Page, Priority},
};
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
    assert_eq!(page.displayed_title(), "Play a song");
    assert_eq!(page.displayed_detail(), "Nothing is playing");
    page.info.available = true;
    page.info.title = "Song".to_owned();
    page.info.artist = "Artist".to_owned();
    assert_eq!((page.displayed_title().as_ref(), page.displayed_detail().as_ref()), ("Song", "Artist"));
    page.info.title.clear();
    assert_eq!(page.displayed_title(), "Unknown title");
}

#[test]
fn playing_music_without_a_lyric_line_leaves_the_collapsed_pill_alone_but_is_open_content() {
    let page = playing_page();
    assert_eq!(page.priority(Mode::Resting), None);
    assert_eq!(page.priority(Mode::Expanded), Some(Priority::NORMAL));
}

#[test]
fn playing_music_with_a_lyric_line_claims_the_collapsed_pill() {
    let page = with_lyrics(playing_page());
    assert_eq!(page.priority(Mode::Resting), Some(Priority::NORMAL));
    assert_eq!(page.resting_lead_width(), resting::ARTWORK_SIZE);
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
fn music_without_a_lyric_line_is_a_wave_badge_in_the_pill() {
    let mut page = playing_page();
    assert_eq!(page.presence(), Some(Presence::Badge { width: wave::WIDTH }));
    assert!(page.is_wave_visible());
    page.info.is_playing = false;
    assert_eq!(page.presence(), None, "paused music has no wave");
    assert!(!page.is_wave_visible());
}

#[test]
fn lyric_mode_has_no_wave_in_the_pill_until_the_notch_opens() {
    let mut page = with_lyrics(playing_page());
    assert_eq!(page.presence(), None);
    assert!(!page.is_wave_visible());
    page.is_hovered = true;
    assert!(page.is_wave_visible(), "the open notch has the wave in its header");
}

#[test]
fn turning_lyrics_off_hands_the_pill_back_to_the_wave_badge() {
    let mut page = with_lyrics(playing_page());
    page.lyrics_enabled = false;
    assert_eq!(page.priority(Mode::Resting), None);
    assert_eq!(page.presence(), Some(Presence::Badge { width: wave::WIDTH }));
}

#[test]
fn the_wave_ranks_nearest_the_clock() {
    assert_eq!(playing_page().header_status_rank(), 0);
    assert_eq!(playing_page().header_status_page(), Some("media"));
}

#[test]
fn a_rest_in_the_lyrics_keeps_the_collapsed_pill_in_lyric_mode() {
    let page = with_lyrics(playing_page());
    assert!(page.follower.current().is_empty());
    assert!(page.claims_pill());
}

#[test]
fn the_media_page_sits_at_the_left_padding_when_the_notch_is_wider() {
    assert!(playing_page().is_left_aligned());
}

#[test]
fn the_idle_player_texts_read_in_indonesian() {
    store::language::with_override(store::language::Language::Indonesian, || {
        let mut page = MediaPage::new();
        assert_eq!(
            (page.displayed_title().as_ref(), page.displayed_detail().as_ref()),
            ("Putar sebuah lagu", "Tidak ada yang diputar")
        );
        page.info.available = true;
        assert_eq!(page.displayed_title(), "Judul tidak diketahui");
    });
}
