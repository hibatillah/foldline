//! Media state comes from session; no progress bar means no per-second redraw while open.

use crate::{
    activity::Presence,
    gfx::{Canvas, Color, ImageSlot, Point, Rect, Size, Spring},
    log,
    page::{Context, Frame, Mode, Page, PageSizes, PointerEvent, Priority, TimerId},
    ui::HoverTarget,
};
use controls::Control;
use layout::Layout;
use lyric_follower::LyricFollower;
use session::{MediaInfo, MediaWatcher};

mod state;
mod sync;
#[cfg(test)]
mod tests;

mod artwork;

mod backdrop;

mod controls;

mod expanded;

mod layout;

mod lyric_follower;

mod lyric_panel;

mod lyrics;

mod resting;

mod session;

mod styles;

mod theme;

mod wave;

const SETTING_LYRICS: &str = "media.lyrics";

const LYRIC_TIMER: TimerId = TimerId(1);

const WAVE_TIMER: TimerId = TimerId(3);

pub struct MediaPage {
    watcher: Option<MediaWatcher>,
    info: MediaInfo,
    artwork: ImageSlot,
    app_icon: ImageSlot,
    follower: LyricFollower,
    lyrics_enabled: bool,
    hovered_control: HoverTarget<Control>,
    is_hovered: bool,
    theme_color: Option<Color>,
    theme_version: u64,
    backdrop: Spring,
    /// Rebuild title lines only when the title, artist, or text-column width changes.
    title_lines: Vec<String>,
    title_lines_source: (String, String, bool),
    lyric_wraps: Vec<Vec<String>>,
    lyric_wraps_source: usize,
    text_width: f32,
}

impl MediaPage {
    pub fn new() -> Self {
        Self {
            watcher: None,
            info: MediaInfo::default(),
            artwork: ImageSlot::default(),
            app_icon: ImageSlot::default(),
            follower: LyricFollower::new(),
            lyrics_enabled: true,
            hovered_control: HoverTarget::default(),
            is_hovered: false,
            theme_color: None,
            theme_version: 0,
            backdrop: Spring::new(0.0, 180.0, 26.0),
            title_lines: Vec::new(),
            title_lines_source: (String::new(), String::new(), false),
            lyric_wraps: Vec::new(),
            lyric_wraps_source: 0,
            text_width: layout::TEXT_MIN_WIDTH,
        }
    }

    #[cfg(test)]
    pub fn sample_idle(fonts: &crate::gfx::Fonts) -> Self {
        let mut page = Self::new();
        page.refresh_title_lines(fonts);
        page
    }

    #[cfg(test)]
    pub fn sample(fonts: &crate::gfx::Fonts, lyrics_on: bool) -> Self {
        use {lyrics::LyricLine, session::LyricsStatus};
        let mut page = Self::new();
        page.info.available = true;
        page.info.is_playing = true;
        page.info.title = "Location Unknown".to_owned();
        page.info.artist = "HONNE, NIKI".to_owned();
        page.info.album = "10 Years".to_owned();
        let texts = [
            "Hold on to what you had",
            "I will find you in the dark",
            "Wherever you are, wherever you go",
            "Take me to the place you know",
            "And I will stay",
        ];
        let lines: Vec<LyricLine> = texts
            .iter()
            .enumerate()
            .map(|(index, text)| LyricLine { start_ticks: index as i64 * 50_000_000, text: (*text).to_owned() })
            .collect();
        let lyrics = std::sync::Arc::new(lyrics::Lyrics { lines });
        page.info.lyrics = LyricsStatus::Found(lyrics.clone());
        page.lyrics_enabled = lyrics_on;
        page.refresh_lyric_wraps(Some(&lyrics), fonts);
        if lyrics_on {
            page.theme_color = Some(Color::rgb(0.2, 0.65, 0.75));
            page.backdrop.jump_to(1.0);
        }
        if lyrics_on {
            page.follower.update(Some(&lyrics), 2 * 50_000_000 + 10_000_000, true, fonts);
            page.follower.animate(5.0);
        }
        page.refresh_title_lines(fonts);
        page
    }
}

impl Page for MediaPage {
    fn id(&self) -> &'static str {
        "media"
    }

    fn uses_units(&self) -> bool {
        true
    }

    fn name(&self) -> String {
        crate::i18n::tr!("page.media").into_owned()
    }

    fn priority(&self, mode: Mode) -> Option<Priority> {
        // Collapsed requires playback; expanded remains available while paused or empty.
        match mode {
            Mode::Resting => self.claims_pill().then_some(Priority::NORMAL),
            Mode::Expanded if self.info.available => Some(Priority::NORMAL),
            Mode::Expanded => Some(Priority::BACKGROUND),
        }
    }

    fn is_left_aligned(&self) -> bool {
        true
    }

    fn sizes(&self) -> PageSizes {
        let width =
            if self.is_lyric_panel_visible() { layout::LYRICS_WIDTH } else { layout::plain_width(self.text_width) };
        PageSizes {
            resting_width: resting::width(self.follower.width()),
            expanded: Size::new(width, layout::PLAIN_HEIGHT),
        }
    }

    fn shows_live_text(&self) -> bool {
        self.claims_pill()
    }

    fn resting_lead_width(&self) -> f32 {
        resting::ARTWORK_SIZE
    }

    fn presence(&self) -> Option<Presence> {
        self.is_wave_in_pill().then_some(Presence::Badge { width: wave::WIDTH })
    }

    fn draw_presence(&self, canvas: &Canvas, area: Rect, _scale: f32, opacity: f32) {
        let center = Point::new(area.center_x(), area.center_y());
        let wave_area = Rect::around(center, wave::WIDTH / 2.0, wave::HEIGHT / 2.0);
        wave::draw(canvas, wave_area, wave::now_seconds(), opacity);
    }

    fn header_status_width(&self) -> Option<f32> {
        self.is_wave_shown().then_some(wave::WIDTH)
    }

    fn draw_header_status(&self, canvas: &Canvas, area: Rect, _scale: f32, opacity: f32) {
        wave::draw(canvas, area, wave::now_seconds(), opacity);
    }

    fn header_status_rank(&self) -> u8 {
        0
    }

    fn header_status_page(&self) -> Option<&'static str> {
        Some("media")
    }

    fn draw_resting(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        resting::draw(self, canvas, frame, opacity);
    }

    fn draw_expanded(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        expanded::draw(self, canvas, frame, opacity);
    }

    fn on_start(&mut self, context: &mut Context) {
        self.lyrics_enabled = context.settings.get_bool(SETTING_LYRICS, true);
        let notifier = context.notifier();
        self.watcher = MediaWatcher::start(move || notifier.notify())
            .map_err(|error| log::debug(&format!("media watcher unavailable: {error}")))
            .ok();
        self.apply_snapshot(context);
    }

    fn on_hover_changed(&mut self, is_hovering: bool, context: &mut Context) {
        self.is_hovered = is_hovering;
        self.sync_wave_timer(context);
    }

    fn on_notify(&mut self, context: &mut Context) {
        self.apply_snapshot(context);
    }

    fn on_settings_changed(&mut self, context: &mut Context) {
        let is_enabled = context.settings.get_bool(SETTING_LYRICS, self.lyrics_enabled);
        if is_enabled != self.lyrics_enabled {
            self.set_lyrics_enabled(is_enabled, context);
        }
    }

    fn on_pointer(&mut self, event: PointerEvent, frame: &Frame, context: &mut Context) {
        let layout = Layout::new(frame.bounds, frame.expanded.width, self.title_lines.len(), self.text_width);
        let can_use_controls = frame.is_open() && self.info.available;
        match event {
            PointerEvent::Moved(point) => {
                let hovered = if can_use_controls { controls::hit_test(&layout, point) } else { None };
                if self.hovered_control.set(hovered) {
                    context.request_repaint();
                }
            }
            PointerEvent::Released(point) if can_use_controls => {
                if let Some(control) = controls::hit_test(&layout, point) {
                    self.activate(control, context);
                }
            }
            PointerEvent::Left | PointerEvent::CaptureLost => {
                self.hovered_control.clear();
                context.request_repaint();
            }
            _ => {}
        }
    }

    fn on_timer(&mut self, timer: TimerId, context: &mut Context) {
        match timer {
            LYRIC_TIMER => {
                self.update_lyrics(context);
                context.request_repaint();
            }
            WAVE_TIMER => context.request_repaint(),
            _ => {}
        }
    }

    fn animate(&mut self, elapsed_seconds: f32) -> bool {
        let is_following = self.follower.animate(elapsed_seconds);
        self.backdrop.step(elapsed_seconds);
        let is_fading = !self.backdrop.is_settled();
        if !is_fading {
            self.backdrop.snap_to_target();
        }
        is_following || is_fading
    }

    /// The host draws the backdrop in device pixels, so `frame.scale` is the display scale here.
    fn draw_backdrop(&self, canvas: &Canvas, frame: &Frame, pill: Rect, radius: f32, opacity: f32) {
        if let Some(color) = self.theme_color {
            backdrop::draw(canvas, pill, radius, frame.scale, color, self.backdrop.position.clamp(0.0, 1.0) * opacity);
        }
    }
}
