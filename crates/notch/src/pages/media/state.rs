//! What the media page is showing, derived from its fields.

use super::lyrics::LyricLine;
use super::session::LyricsStatus;
use super::{MediaPage, NOTHING_PLAYING_DETAIL, NOTHING_PLAYING_TITLE, UNKNOWN_TITLE, resting};

impl MediaPage {
    pub(super) fn is_wave_shown(&self) -> bool {
        self.is_track_playing()
    }

    pub(super) fn is_wave_visible(&self) -> bool {
        self.is_wave_shown() && (self.is_hovered || self.resting_view() == resting::View::Wave)
    }

    pub(super) fn is_track_playing(&self) -> bool {
        self.info.is_playing && !self.info.title.is_empty()
    }

    pub(super) fn has_lyrics(&self) -> bool {
        matches!(self.info.lyrics, LyricsStatus::Found(_))
    }

    pub(super) fn is_lyric_panel_visible(&self) -> bool {
        self.lyrics_enabled && self.has_lyrics()
    }

    pub(super) fn is_lyric_mode(&self) -> bool {
        self.info.is_playing && self.is_lyric_panel_visible()
    }

    pub(super) fn resting_view(&self) -> resting::View {
        if self.is_title_shown {
            resting::View::TrackChange
        } else if self.is_lyric_mode() {
            // Stays in lyric mode through a part of the song with no words; the pill then shows a note.
            resting::View::Lyrics
        } else {
            resting::View::Wave
        }
    }

    pub(super) fn visible_lyric_lines(&self) -> Option<&[LyricLine]> {
        match &self.info.lyrics {
            LyricsStatus::Found(lyrics) if self.lyrics_enabled && self.info.available => Some(&lyrics.lines),
            _ => None,
        }
    }

    pub(super) fn displayed_title(&self) -> &str {
        if !self.info.available {
            NOTHING_PLAYING_TITLE
        } else if self.info.title.is_empty() {
            UNKNOWN_TITLE
        } else {
            &self.info.title
        }
    }

    pub(super) fn displayed_detail(&self) -> &str {
        if self.info.available { &self.info.artist } else { NOTHING_PLAYING_DETAIL }
    }
}
