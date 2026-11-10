//! What the media page is showing, derived from its fields.

use super::MediaPage;
use super::lyrics::LyricLine;
use super::session::LyricsStatus;
use crate::i18n::tr;

impl MediaPage {
    pub(super) fn is_wave_shown(&self) -> bool {
        self.is_track_playing()
    }

    /// Playing with a lyric line: the page takes over the collapsed pill.
    pub(super) fn claims_pill(&self) -> bool {
        self.is_track_playing() && self.is_lyric_mode()
    }

    /// Playing without a lyric line: only the wave shows, as a badge.
    pub(super) fn is_wave_in_pill(&self) -> bool {
        self.is_track_playing() && !self.is_lyric_mode()
    }

    pub(super) fn is_wave_visible(&self) -> bool {
        self.is_wave_shown() && (self.is_hovered || !self.is_lyric_mode())
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

    pub(super) fn visible_lyric_lines(&self) -> Option<&[LyricLine]> {
        match &self.info.lyrics {
            LyricsStatus::Found(lyrics) if self.lyrics_enabled && self.info.available => Some(&lyrics.lines),
            _ => None,
        }
    }

    pub(super) fn displayed_title(&self) -> std::borrow::Cow<'_, str> {
        if !self.info.available {
            tr!("media.nothing_title")
        } else if self.info.title.is_empty() {
            tr!("media.unknown_title")
        } else {
            std::borrow::Cow::Borrowed(self.info.title.as_str())
        }
    }

    pub(super) fn displayed_detail(&self) -> std::borrow::Cow<'_, str> {
        if self.info.available {
            std::borrow::Cow::Borrowed(self.info.artist.as_str())
        } else {
            tr!("media.nothing_detail")
        }
    }
}
