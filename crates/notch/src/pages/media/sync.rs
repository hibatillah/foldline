//! Keeping the page's text, lyrics, backdrop and timers in step with the playing track.

use super::controls::Control;
use super::session::LyricsStatus;
use super::{LYRIC_TIMER, MediaPage, SETTING_LYRICS, WAVE_TIMER, layout, lyrics, styles, theme};
use crate::{
    gfx::{Fonts, balance_lines, clamp_lines},
    log,
    page::Context,
};

pub(super) const WAVE_FRAME_MILLISECONDS: u32 = 100;
/// Title lines are fitted a little narrower than their rectangle, so rounding never trims a line that fits.
pub(super) const TITLE_FIT_MARGIN: f32 = 1.0;
pub(super) const LYRIC_FIT_MARGIN: f32 = 2.0;

impl MediaPage {
    pub(super) fn sync_backdrop(&mut self, context: &mut Context) {
        let wanted = if self.is_lyric_panel_visible() && self.theme_color.is_some() { 1.0 } else { 0.0 };
        if self.backdrop.target != wanted {
            self.backdrop.target = wanted;
            context.request_animation();
        }
    }

    pub(super) fn sync_wave_timer(&self, context: &mut Context) {
        if self.is_wave_visible() {
            context.set_timer(WAVE_TIMER, WAVE_FRAME_MILLISECONDS);
        } else {
            context.kill_timer(WAVE_TIMER);
        }
    }

    pub(super) fn apply_snapshot(&mut self, context: &mut Context) {
        if let Some(watcher) = &self.watcher {
            self.info = watcher.snapshot();
        }
        self.artwork.sync(self.info.artwork.as_deref(), |image| context.upload_image(image));
        self.app_icon.sync(self.info.app_icon.as_deref(), |image| context.upload_image(image));
        let cover_version = self.info.artwork.as_deref().map_or(0, |image| image.version);
        if cover_version != self.theme_version {
            self.theme_version = cover_version;
            self.theme_color = self.info.artwork.as_deref().and_then(theme::color_of);
        }
        self.refresh_title_lines(context.fonts());
        self.sync_wave_timer(context);
        self.sync_backdrop(context);
        self.update_lyrics(context);
        context.request_repaint();
    }

    pub(super) fn refresh_title_lines(&mut self, fonts: &Fonts) {
        let source =
            (self.displayed_title().to_owned(), self.displayed_detail().to_owned(), self.is_lyric_panel_visible());
        if source == self.title_lines_source && !self.title_lines.is_empty() {
            return;
        }
        let title = &source.0;
        let width = layout::title_width(source.2) - TITLE_FIT_MARGIN;
        let measure_title = |text: &str| fonts.measure(&styles::TITLE, text);
        self.title_lines = clamp_lines(title, layout::TITLE_MAX_LINES, width, measure_title);
        let widest = self
            .title_lines
            .iter()
            .map(|line| measure_title(line))
            .fold(fonts.measure(&styles::DETAIL, &source.1), f32::max);
        self.text_width = (widest + TITLE_FIT_MARGIN).clamp(layout::TEXT_MIN_WIDTH, layout::TEXT_MAX_WIDTH);
        self.title_lines_source = source;
    }

    pub(super) fn refresh_lyric_wraps(&mut self, lyrics: Option<&std::sync::Arc<lyrics::Lyrics>>, fonts: &Fonts) {
        let Some(lyrics) = lyrics else {
            self.lyric_wraps.clear();
            self.lyric_wraps_source = 0;
            return;
        };
        let source = std::sync::Arc::as_ptr(lyrics) as usize;
        if source == self.lyric_wraps_source {
            return;
        }
        let width = layout::lyric_panel_width() - LYRIC_FIT_MARGIN;
        self.lyric_wraps = lyrics
            .lines
            .iter()
            .map(|line| {
                balance_lines(&line.text, layout::LYRIC_MAX_LINES, width, |text| {
                    fonts.measure(&styles::LYRIC_LINE, text)
                })
            })
            .collect();
        self.lyric_wraps_source = source;
    }

    pub(super) fn update_lyrics(&mut self, context: &mut Context) {
        let lyrics = match &self.info.lyrics {
            LyricsStatus::Found(lyrics) if self.lyrics_enabled => Some(lyrics.clone()),
            _ => None,
        };
        self.refresh_lyric_wraps(lyrics.as_ref(), context.fonts());
        let update = self.follower.update(
            lyrics.as_ref(),
            self.info.position_ticks_now(),
            self.info.is_playing,
            context.fonts(),
        );
        if update.line_changed {
            log::debug(&format!(
                "lyric line index={:?} chars={}",
                self.follower.index(),
                self.follower.current().chars().count()
            ));
            context.request_animation();
        }
        match update.next_line_in_milliseconds {
            Some(milliseconds) => context.set_timer(LYRIC_TIMER, milliseconds),
            None => context.kill_timer(LYRIC_TIMER),
        }
    }

    pub(super) fn toggle_lyrics(&mut self, context: &mut Context) {
        context.settings.set_bool(SETTING_LYRICS, !self.lyrics_enabled);
        self.set_lyrics_enabled(!self.lyrics_enabled, context);
    }

    pub(super) fn set_lyrics_enabled(&mut self, is_enabled: bool, context: &mut Context) {
        self.lyrics_enabled = is_enabled;
        self.refresh_title_lines(context.fonts());
        self.sync_wave_timer(context);
        self.sync_backdrop(context);
        self.update_lyrics(context);
        context.request_repaint();
    }

    pub(super) fn activate(&mut self, control: Control, context: &mut Context) {
        if control == Control::Lyrics {
            self.toggle_lyrics(context);
            return;
        }
        let Some(watcher) = &self.watcher else { return };
        match control {
            Control::Previous => watcher.skip_previous(),
            Control::PlayPause => watcher.toggle_play_pause(),
            Control::Next => watcher.skip_next(),
            Control::Lyrics => {}
        }
    }
}
