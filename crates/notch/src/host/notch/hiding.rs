use super::{
    EXPAND_TIMER_ID, HIDDEN_HEIGHT, HIDE_WHEN_IDLE_SETTING, HOVER_WATCH_TIMER_ID, Notch, RAISE_TIMER_ID,
    REVEAL_TIMER_ID, WINDOW_REVEAL_TIMER_ID,
};
use crate::host::settings::Settings;
use windows::Win32::UI::WindowsAndMessaging::{KillTimer, SW_HIDE, SW_SHOWNOACTIVATE, SetTimer, ShowWindow};

use crate::host::{
    screen_mode::ScreenMode,
    visibility::{self, Showing},
};
use crate::{log, page::PointerEvent};

/// What the user chose to show over maximized and full-screen windows, and what the foreground window is now.
pub(super) struct HidingState {
    pub(super) hides_when_idle: bool,
    pub(super) maximized_showing: Showing,
    pub(super) fullscreen_showing: Showing,
    pub(super) opens_on_hover_over_maximized: bool,
    pub(super) opens_on_hover_over_fullscreen: bool,
    /// The mode the notch follows; a mode that would show more waits in `pending_screen_mode` first.
    pub(super) screen_mode: ScreenMode,
    pub(super) pending_screen_mode: Option<ScreenMode>,
    pub(super) is_revealed: bool,
}

impl HidingState {
    pub(super) fn read(settings: &Settings) -> Self {
        Self {
            hides_when_idle: settings.get_bool(HIDE_WHEN_IDLE_SETTING, false),
            maximized_showing: read_showing(settings, visibility::MAXIMIZED_KEY, visibility::DEFAULT_MAXIMIZED),
            fullscreen_showing: read_showing(settings, visibility::FULLSCREEN_KEY, visibility::DEFAULT_FULLSCREEN),
            opens_on_hover_over_maximized: settings.get_bool(visibility::HOVER_MAXIMIZED_KEY, true),
            opens_on_hover_over_fullscreen: settings.get_bool(visibility::HOVER_FULLSCREEN_KEY, true),
            screen_mode: ScreenMode::Normal,
            pending_screen_mode: None,
            is_revealed: false,
        }
    }

    /// A missing or unreadable value keeps the current choice.
    pub(super) fn reload(&mut self, settings: &Settings) {
        self.hides_when_idle = settings.get_bool(HIDE_WHEN_IDLE_SETTING, self.hides_when_idle);
        self.maximized_showing = read_showing(settings, visibility::MAXIMIZED_KEY, self.maximized_showing);
        self.fullscreen_showing = read_showing(settings, visibility::FULLSCREEN_KEY, self.fullscreen_showing);
        self.opens_on_hover_over_maximized =
            settings.get_bool(visibility::HOVER_MAXIMIZED_KEY, self.opens_on_hover_over_maximized);
        self.opens_on_hover_over_fullscreen =
            settings.get_bool(visibility::HOVER_FULLSCREEN_KEY, self.opens_on_hover_over_fullscreen);
    }
}

/// A missing key or an unknown word gives `fallback`: the default at start, the current choice on reload.
fn read_showing(settings: &Settings, key: &str, fallback: Showing) -> Showing {
    settings.get_string(key).and_then(Showing::parse).unwrap_or(fallback)
}

const REVEAL_DELAY_MILLISECONDS: u32 = 150;
/// Over a maximized window the pointer must stay longer at the top, which is where its tabs and title bar are.
const REVEAL_DELAY_OVER_WINDOW_MILLISECONDS: u32 = 350;
const HIDDEN_HEIGHT_OVER_WINDOW: f32 = 3.0;
/// The window the user switched to finishes raising itself after the foreground event, so raise the notch again.
const RAISE_DELAY_MILLISECONDS: u32 = 250;
/// Alt+Tab or a click between two maximized windows passes through a regular foreground window for a moment.
pub(super) const WINDOW_REVEAL_DELAY_MILLISECONDS: u32 = 500;

impl Notch {
    pub(super) fn is_hidden(&self) -> bool {
        if self.hiding.is_revealed || self.springs.expansion.target != 0.0 {
            return false;
        }
        // In every mode only a message counts as activity; a lasting mark (muted, Caps Lock, a countdown, a privacy
        // dot) is state and would keep the pill up for as long as it lasts.
        let has_message = !self.presences.transients.is_empty() || self.presences.banner.is_some();
        match self.current_showing() {
            None => self.hiding.hides_when_idle && !has_message && !self.has_resting_content,
            // The suppressed window draws nothing; a full-size layout avoids a strip growing back when it returns.
            Some(Showing::Hidden) => false,
            Some(showing) => {
                let has_lyrics = self.resting_index.is_some_and(|index| self.pages[index].shows_live_text());
                !visibility::allows(showing, has_message, self.has_resting_content, has_lyrics)
            }
        }
    }

    /// `None` over a regular window, where `notch.hide_when_idle` decides instead.
    pub(super) fn current_showing(&self) -> Option<Showing> {
        visibility::showing_for(self.hiding.screen_mode, self.hiding.maximized_showing, self.hiding.fullscreen_showing)
    }

    pub(super) fn opens_on_hover(&self) -> bool {
        visibility::opens_on_hover(
            self.hiding.screen_mode,
            self.hiding.opens_on_hover_over_maximized,
            self.hiding.opens_on_hover_over_fullscreen,
        )
    }

    pub(super) fn is_suppressed(&self) -> bool {
        self.current_showing() == Some(Showing::Hidden)
    }

    pub(super) fn hidden_height(&self) -> f32 {
        if self.hiding.hides_when_idle { HIDDEN_HEIGHT } else { HIDDEN_HEIGHT_OVER_WINDOW }
    }

    pub(super) fn reveal_delay(&self) -> u32 {
        if self.hiding.hides_when_idle { REVEAL_DELAY_MILLISECONDS } else { REVEAL_DELAY_OVER_WINDOW_MILLISECONDS }
    }

    pub fn set_screen_mode(&mut self, mode: ScreenMode) {
        if visibility::showing_for(mode, self.hiding.maximized_showing, self.hiding.fullscreen_showing)
            != Some(Showing::Hidden)
        {
            crate::host::window::raise_topmost(self.window);
            unsafe { SetTimer(Some(self.window), RAISE_TIMER_ID, RAISE_DELAY_MILLISECONDS, None) };
        }
        if mode == self.hiding.screen_mode {
            self.cancel_window_reveal();
            return;
        }
        let showing = visibility::showing_for(mode, self.hiding.maximized_showing, self.hiding.fullscreen_showing);
        if visibility::reveals_more(self.current_showing(), showing) {
            if self.hiding.pending_screen_mode != Some(mode) {
                self.hiding.pending_screen_mode = Some(mode);
                unsafe { SetTimer(Some(self.window), WINDOW_REVEAL_TIMER_ID, WINDOW_REVEAL_DELAY_MILLISECONDS, None) };
            }
            return;
        }
        self.cancel_window_reveal();
        self.apply_screen_mode(mode);
    }

    /// The window reveal delay ended: the last mode reported during the wait applies.
    pub(super) fn finish_window_reveal(&mut self) {
        let pending = self.hiding.pending_screen_mode;
        self.cancel_window_reveal();
        if let Some(mode) = pending {
            self.apply_screen_mode(mode);
        }
    }

    fn cancel_window_reveal(&mut self) {
        if self.hiding.pending_screen_mode.take().is_some() {
            let _ = unsafe { KillTimer(Some(self.window), WINDOW_REVEAL_TIMER_ID) };
        }
    }

    fn apply_screen_mode(&mut self, mode: ScreenMode) {
        let was_suppressed = self.is_suppressed();
        self.hiding.screen_mode = mode;
        self.apply_suppression(was_suppressed);
        self.flush();
    }

    pub fn reset_session_interaction(&mut self) {
        self.hover.is_dragging_files = false;
        self.set_hovering(false);
        self.hiding.is_revealed = false;
        self.cancel_close();
        self.scroll_gate.reset();
        for timer_key in [REVEAL_TIMER_ID, EXPAND_TIMER_ID, HOVER_WATCH_TIMER_ID] {
            let _ = unsafe { KillTimer(Some(self.window), timer_key) };
        }
        self.route_pointer(PointerEvent::CaptureLost);
        self.springs.expansion.jump_to(0.0);
        crate::host::drop_target::reset_interaction();
        self.flush();
    }

    pub(super) fn apply_suppression(&mut self, was_suppressed: bool) {
        let is_suppressed = self.is_suppressed();
        if is_suppressed == was_suppressed {
            return;
        }
        log::debug(&format!("suppressed={is_suppressed}"));
        if is_suppressed {
            self.set_hovering(false);
            let _ = unsafe { ShowWindow(self.window, SW_HIDE) };
        } else {
            let _ = unsafe { ShowWindow(self.window, SW_SHOWNOACTIVATE) };
            // A reload can release the window while a maximized or full-screen window sits above it.
            crate::host::window::raise_topmost(self.window);
        }
        crate::host::drop_target::set_suppressed(is_suppressed);
        if !is_suppressed {
            self.paint();
        }
    }
}
