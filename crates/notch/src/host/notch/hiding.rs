use super::{
    EXPAND_TIMER_ID, HIDDEN_HEIGHT, HIDE_WHEN_IDLE_SETTING, HOVER_WATCH_TIMER_ID, Notch, RAISE_TIMER_ID,
    REVEAL_TIMER_ID, WINDOW_REVEAL_TIMER_ID,
};
use crate::host::settings::Settings;
use windows::Win32::UI::WindowsAndMessaging::{KillTimer, SW_HIDE, SW_SHOWNOACTIVATE, SetTimer, ShowWindow};

use crate::host::{
    screen_mode::ScreenMode,
    visibility::{self, Collapsed, Showing},
    volume_flyout,
};
use crate::{log, page::PointerEvent};

/// What the user chose to show over maximized and full-screen windows, and what the foreground window is now.
pub(super) struct HidingState {
    pub(super) hides_when_idle: bool,
    pub(super) maximized_showing: Showing,
    pub(super) fullscreen_showing: Showing,
    pub(super) opens_on_pointer_over_maximized: bool,
    pub(super) opens_on_pointer_over_fullscreen: bool,
    /// `notch.hide_windows_volume_popup`: while the notch replaces Windows' flyout, volume and brightness always show.
    pub(super) replaces_windows_flyout: bool,
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
            opens_on_pointer_over_maximized: settings.get_bool(visibility::HOVER_MAXIMIZED_KEY, true),
            opens_on_pointer_over_fullscreen: settings.get_bool(visibility::HOVER_FULLSCREEN_KEY, true),
            replaces_windows_flyout: volume_flyout::wants_hidden(settings),
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
        self.opens_on_pointer_over_maximized =
            settings.get_bool(visibility::HOVER_MAXIMIZED_KEY, self.opens_on_pointer_over_maximized);
        self.opens_on_pointer_over_fullscreen =
            settings.get_bool(visibility::HOVER_FULLSCREEN_KEY, self.opens_on_pointer_over_fullscreen);
        self.replaces_windows_flyout = settings.get_bool(volume_flyout::SETTING_KEY, self.replaces_windows_flyout);
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
pub(super) const RAISE_DELAY_MILLISECONDS: u32 = 250;
/// Alt+Tab or a click between two maximized windows passes through a regular foreground window for a moment.
pub(super) const WINDOW_REVEAL_DELAY_MILLISECONDS: u32 = 500;

impl Notch {
    pub(super) fn is_hidden(&self) -> bool {
        if self.hiding.is_revealed || self.springs.expansion.target != 0.0 {
            return false;
        }
        let collapsed = Collapsed {
            has_message: !self.presences.transients.is_empty() || self.presences.banner.is_some(),
            has_resting_content: self.has_resting_content,
            has_lyrics: self.resting_index.is_some_and(|index| self.pages[index].shows_live_text()),
        };
        visibility::hides_to_strip(
            self.current_showing(),
            self.hiding.hides_when_idle,
            collapsed,
            self.priority_shows(),
            visibility::pin_shows(self.hiding.screen_mode, self.open.is_pinned),
        )
    }

    /// A priority activity's presence (a volume or brightness level, the muted badge) overrides the word now.
    fn priority_shows(&self) -> bool {
        visibility::priority_shows(
            self.hiding.screen_mode,
            self.hiding.replaces_windows_flyout,
            self.presences.has_priority,
        )
    }

    /// `None` over a regular window, where `notch.hide_when_idle` decides instead.
    pub(super) fn current_showing(&self) -> Option<Showing> {
        visibility::showing_for(self.hiding.screen_mode, self.hiding.maximized_showing, self.hiding.fullscreen_showing)
    }

    /// Under the word `hidden` the pointer never opens the notch, even while a priority activity shows it.
    pub(super) fn opens_on_pointer(&self) -> bool {
        self.current_showing() != Some(Showing::Hidden)
            && visibility::opens_on_pointer(
                self.hiding.screen_mode,
                self.hiding.opens_on_pointer_over_maximized,
                self.hiding.opens_on_pointer_over_fullscreen,
            )
    }

    /// A pause hides the notch before any screen mode or pin is asked.
    pub(super) fn is_suppressed(&self) -> bool {
        self.is_paused() || !self.stays_on_screen_in(self.hiding.screen_mode)
    }

    pub(super) fn hidden_height(&self) -> f32 {
        if self.hiding.hides_when_idle { HIDDEN_HEIGHT } else { HIDDEN_HEIGHT_OVER_WINDOW }
    }

    pub(super) fn reveal_delay(&self) -> u32 {
        if self.hiding.hides_when_idle { REVEAL_DELAY_MILLISECONDS } else { REVEAL_DELAY_OVER_WINDOW_MILLISECONDS }
    }

    /// Whether the notch stays on screen in front of a window in `mode`, so it must raise itself above that window.
    pub(super) fn stays_on_screen_in(&self, mode: ScreenMode) -> bool {
        visibility::stays_on_screen(
            mode,
            self.hiding.maximized_showing,
            self.hiding.fullscreen_showing,
            self.hiding.replaces_windows_flyout,
            self.presences.has_priority,
            self.open.is_pinned,
        )
    }

    pub fn set_screen_mode(&mut self, mode: ScreenMode) {
        // The window is hidden: keep the mode for the resume and arm no timer.
        if self.is_paused() {
            self.hiding.screen_mode = mode;
            return;
        }
        if self.stays_on_screen_in(mode) {
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

    /// A pinned notch stays open; only the pointer and drag state are cleared.
    pub fn reset_session_interaction(&mut self) {
        self.open.is_dragging_files = false;
        if !self.open.is_pinned {
            self.set_open(false);
            self.hiding.is_revealed = false;
        }
        self.cancel_close();
        self.scroll_gate.reset();
        for timer_key in [REVEAL_TIMER_ID, EXPAND_TIMER_ID, HOVER_WATCH_TIMER_ID] {
            let _ = unsafe { KillTimer(Some(self.window), timer_key) };
        }
        self.route_pointer(PointerEvent::CaptureLost);
        if !self.open.is_pinned {
            self.springs.expansion.jump_to(0.0);
        }
        crate::host::drop_target::reset_interaction();
        // The reset tells the overlay the notch closed; posted after it, this keeps it off a pinned open header.
        if self.drop_target_stays_told_open() {
            crate::host::drop_target::set_open(true);
        }
        self.flush();
    }

    /// Whether the drop overlay must hear again that the notch is open after a session reset.
    pub(super) fn drop_target_stays_told_open(&self) -> bool {
        self.open.is_pinned && self.open.is_open
    }

    pub(super) fn apply_suppression(&mut self, was_suppressed: bool) {
        let is_suppressed = self.is_suppressed();
        if is_suppressed == was_suppressed {
            return;
        }
        log::debug(&format!("suppressed={is_suppressed}"));
        // A screenshot overlay closes a pinned notch but keeps the pin, so it opens again when the overlay ends.
        if is_suppressed {
            self.set_open(false);
            let _ = unsafe { ShowWindow(self.window, SW_HIDE) };
        } else {
            let _ = unsafe { ShowWindow(self.window, SW_SHOWNOACTIVATE) };
            // A reload can release the window while a maximized or full-screen window sits above it.
            crate::host::window::raise_topmost(self.window);
        }
        crate::host::drop_target::set_suppressed(is_suppressed);
        if !is_suppressed && self.open.is_pinned {
            self.set_open(true);
        }
        if !is_suppressed {
            self.paint();
        }
    }
}
