//! Host/page contracts; see docs/tutorials/add-a-page.md.

use windows::{
    Win32::{
        Foundation::{HANDLE, HWND, LPARAM, WPARAM},
        System::{DataExchange::AddClipboardFormatListener, Power::RegisterPowerSettingNotification},
        UI::WindowsAndMessaging::{KillTimer, PostMessageW, REGISTER_NOTIFICATION_FLAGS, SetTimer, WM_APP},
    },
    core::GUID,
};

use crate::{
    activity::Presence,
    gfx::{Canvas, Fonts, ImageBrush, ImageData, Point, Rect, Size, Surface},
    host::Settings,
    ui::Icon,
};

/// Device-independent pixels; resting content is centered within this height.
pub const RESTING_HEIGHT: f32 = 33.0;
pub const RESTING_MIN_WIDTH: f32 = 150.0;
/// The open content height most pages use.
pub const DEFAULT_CONTENT_HEIGHT: f32 = 86.0;
/// The tallest open content the notch window holds, below its header and padding.
pub const MAX_CONTENT_HEIGHT: f32 = 220.0;

pub(crate) const WM_PAGE_NOTIFY: u32 = WM_APP + 1;
/// Win32 timer ids from this up to `PAGE_TIMER_LIMIT` belong to pages; each page owns `TIMERS_PER_PAGE` of them.
pub(crate) const PAGE_TIMER_BASE: usize = 100;
pub(crate) const TIMERS_PER_PAGE: usize = 16;
pub(crate) const PAGE_TIMER_LIMIT: usize = 1000;

/// How much a page wants to be shown right now; when several pages want to, the highest priority wins.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Priority(pub u8);

impl Priority {
    /// For pages that are available whenever the user opens the notch, such as the shelf.
    pub const BACKGROUND: Priority = Priority(0);
    /// For pages that show live content, such as music that is playing.
    pub const NORMAL: Priority = Priority(10);
    /// For pages that must interrupt everything else, such as a notification that has just arrived.
    pub const URGENT: Priority = Priority(20);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Resting,
    Expanded,
}

/// Content sizes in DIP; the host adds padding and a header. `expanded.height` defaults to `DEFAULT_CONTENT_HEIGHT`
/// and is capped at `MAX_CONTENT_HEIGHT`.
#[derive(Clone, Copy, Debug)]
pub struct PageSizes {
    pub resting_width: f32,
    pub expanded: Size,
}

#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub scale: f32,
    /// Device pixels, shared with pointer input; visible pages keep their natural width during swipes.
    pub bounds: Rect,
    /// 0 collapsed, 1 fully open; the spring can overshoot a little past 1.
    pub expansion: f32,
    /// The content size in device-independent pixels; visible pages retain their own width during navigation.
    pub expanded: Size,
}

impl Frame {
    pub fn is_open(&self) -> bool {
        self.expansion >= 0.8
    }
}

/// Pointer input, delivered to visible open pages in their own shifted frames. Positions are device pixels.
#[derive(Clone, Copy, Debug)]
pub enum PointerEvent {
    Moved(Point),
    Pressed(Point),
    Released(Point),
    Left,
    CaptureLost,
    SecondaryReleased(Point),
}

/// Typing delivered to the page that holds the keyboard (`Context::take_keyboard`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyInput {
    /// Typed or pasted characters, without line breaks or other control characters.
    Text(String),
    Backspace,
    Delete,
    Left,
    Right,
    Home,
    End,
    Enter,
    Escape,
}

/// One setting the notch draws in its settings panel, saved under one settings key.
#[derive(Clone, Debug, PartialEq)]
pub enum Control {
    /// Saved as `on` or `off`.
    Switch { key: &'static str, label: &'static str, hint: &'static str, default: bool },
    /// A whole number changed by - and + in steps, saved as digits (with a minus sign when negative).
    Stepper { key: &'static str, label: &'static str, unit: &'static str, default: i32, min: i32, max: i32, step: i32 },
    /// One word out of `options` (saved word, shown label).
    Choice {
        key: &'static str,
        label: &'static str,
        options: &'static [(&'static str, &'static str)],
        default: &'static str,
    },
    /// Typed in the notch; a secret shows dots.
    Text { key: &'static str, label: &'static str, placeholder: &'static str, is_secret: bool },
    /// Typed in the notch; `Page::search` lists matches and picking one saves its values. `key` holds the shown name.
    Search { key: &'static str, label: &'static str, placeholder: &'static str },
    /// Up to `max` of `options`, saved as a comma list in the order picked; no saved list means the first ones.
    Picks { key: &'static str, label: &'static str, options: Vec<String>, max: usize },
}

impl Control {
    pub fn key(&self) -> &'static str {
        match self {
            Control::Switch { key, .. }
            | Control::Stepper { key, .. }
            | Control::Choice { key, .. }
            | Control::Text { key, .. }
            | Control::Search { key, .. }
            | Control::Picks { key, .. } => key,
        }
    }
}

/// One match of a `Control::Search`: what it shows, and the settings keys picking it writes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchHit {
    pub label: String,
    pub values: Vec<(&'static str, String)>,
}

/// A tab of a page's settings in the notch's panel.
pub struct SettingsGroup {
    pub title: &'static str,
    pub controls: Vec<Control>,
}

/// Identifies one of a page's timers. Must be below `TIMERS_PER_PAGE` (16).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimerId(pub u8);

pub trait Page {
    /// Stable name used in logs and settings keys.
    fn id(&self) -> &'static str;

    /// Re-evaluated after every event; None means unavailable in this mode.
    fn priority(&self, mode: Mode) -> Option<Priority>;

    /// Re-evaluated after every event; sizes describe content without host chrome.
    fn sizes(&self) -> PageSizes;

    /// Draws the collapsed pill content. `opacity` fades it out while the notch opens.
    fn draw_resting(&self, canvas: &Canvas, frame: &Frame, opacity: f32);

    /// Draws the open content. `opacity` fades it in while the notch opens.
    fn draw_expanded(&self, canvas: &Canvas, frame: &Frame, opacity: f32);

    /// Called once after the window exists: start background work and timers here.
    fn on_start(&mut self, _context: &mut Context) {}

    /// All pages receive hover changes, including those not currently selected.
    fn on_hover_changed(&mut self, _is_hovering: bool, _context: &mut Context) {}

    /// True for a page that takes files dropped on the notch; the first such page that is switched on gets them.
    fn accepts_drops(&self) -> bool {
        false
    }

    fn on_drop(&mut self, _paths: Vec<std::path::PathBuf>, _context: &mut Context) {}

    /// Only the drop-accepting page receives this; false means the drag left or was dropped.
    fn on_file_drag_changed(&mut self, _is_over: bool, _context: &mut Context) {}

    /// Start or stop work that is needed only while this page is visible in the open notch.
    fn on_shown_changed(&mut self, _is_shown: bool, _context: &mut Context) {}

    fn on_pointer(&mut self, _event: PointerEvent, _frame: &Frame, _context: &mut Context) {}

    /// Whether a right-click at `point` means something on this page. When it does the page gets
    /// `PointerEvent::SecondaryReleased`; otherwise the host opens its own menu.
    fn uses_right_click(&self, _point: Point, _frame: &Frame) -> bool {
        false
    }

    fn on_timer(&mut self, _timer: TimerId, _context: &mut Context) {}

    /// Typing, delivered only while this page holds the keyboard.
    fn on_key(&mut self, _key: KeyInput, _context: &mut Context) {}

    /// The keyboard went elsewhere without the page releasing it (a click in another window): stop editing.
    fn on_keyboard_lost(&mut self, _context: &mut Context) {}

    /// A background thread called this page's `Notifier`: read the shared state and request a repaint.
    fn on_notify(&mut self, _context: &mut Context) {}

    /// The page's own settings, shown in the first tab of the notch's settings panel; empty means none.
    fn settings(&self) -> Vec<Control> {
        Vec::new()
    }

    /// The page's settings as tabs; a page with more settings than one tab holds overrides this.
    fn settings_groups(&self) -> Vec<SettingsGroup> {
        let controls = self.settings();
        if controls.is_empty() { Vec::new() } else { vec![SettingsGroup { title: self.name(), controls }] }
    }

    /// Matches for what was typed into a `Control::Search`, best first; at most a handful.
    fn search(&self, _key: &str, _query: &str) -> Vec<SearchHit> {
        Vec::new()
    }

    /// Called after another program changed settings.txt and the host re-read it: re-read this page's keys here.
    fn on_settings_changed(&mut self, _context: &mut Context) {}

    /// Collapsed activity content, independent of page selection; re-evaluated after every event.
    fn presence(&self) -> Option<Presence> {
        None
    }

    /// Draws what `presence` asked for into `area` (device pixels), faded by `opacity`.
    fn draw_presence(&self, _canvas: &Canvas, _area: Rect, _scale: f32, _opacity: f32) {}

    /// Draws behind the header and content; pill is the whole notch, frame is this page's content.
    /// Bounds and radius are device pixels.
    fn draw_backdrop(&self, _canvas: &Canvas, _frame: &Frame, _pill: Rect, _radius: f32, _opacity: f32) {}

    /// Changing text the user is reading may keep the pill visible over a maximized window.
    fn shows_live_text(&self) -> bool {
        false
    }

    /// Status width in DIP, independent of the selected page; None hides it. Asked on every paint.
    fn header_status_width(&self) -> Option<f32> {
        None
    }

    /// Lower ranks sit nearer the time; equal ranks retain page order.
    fn header_status_rank(&self) -> u8 {
        0
    }

    /// Draws the status mark into `area` (device pixels, as high as the header), faded by `opacity`.
    fn draw_header_status(&self, _canvas: &Canvas, _area: Rect, _scale: f32, _opacity: f32) {}

    /// What the Pages tab and the settings panel call the page.
    fn name(&self) -> &'static str {
        self.id()
    }

    fn icon(&self) -> Icon {
        Icon::Clock
    }

    /// True for a page that starts switched off until the user adds it.
    fn is_opt_in(&self) -> bool {
        false
    }

    /// Called only while animating; return true to keep the frame timer running.
    fn animate(&mut self, _elapsed_seconds: f32) -> bool {
        false
    }
}

#[derive(Default)]
pub(crate) struct Requests {
    pub repaint: bool,
    pub animate: bool,
    /// The page that asked for the keyboard (`true`) or gave it back (`false`).
    pub keyboard: Option<(usize, bool)>,
}

pub struct Context<'a> {
    window: HWND,
    owner_index: usize,
    timer_base: usize,
    notify_message: u32,
    surface: &'a Surface,
    fonts: &'a Fonts,
    requests: &'a mut Requests,
    /// Persistent settings shared by all pages and activities; prefix keys with the page or activity id.
    pub settings: &'a mut Settings,
}

impl<'a> Context<'a> {
    pub(crate) fn new(
        window: HWND,
        page_index: usize,
        surface: &'a Surface,
        fonts: &'a Fonts,
        requests: &'a mut Requests,
        settings: &'a mut Settings,
    ) -> Self {
        Self {
            window,
            owner_index: page_index,
            timer_base: PAGE_TIMER_BASE,
            notify_message: WM_PAGE_NOTIFY,
            surface,
            fonts,
            requests,
            settings,
        }
    }

    pub(crate) fn for_activity(
        window: HWND,
        activity_index: usize,
        surface: &'a Surface,
        fonts: &'a Fonts,
        requests: &'a mut Requests,
        settings: &'a mut Settings,
    ) -> Self {
        Self {
            window,
            owner_index: activity_index,
            timer_base: crate::activity::ACTIVITY_TIMER_BASE,
            notify_message: crate::activity::WM_ACTIVITY_NOTIFY,
            surface,
            fonts,
            requests,
            settings,
        }
    }

    pub fn fonts(&self) -> &Fonts {
        self.fonts
    }

    pub fn request_repaint(&mut self) {
        self.requests.repaint = true;
    }

    /// Start the frame timer so `Page::animate` gets called; use after starting an animation of the page's own.
    pub fn request_animation(&mut self) {
        self.requests.animate = true;
    }

    /// Calls `Page::on_timer` with `timer` after `milliseconds`, repeating until killed. Setting it again re-arms it.
    pub fn set_timer(&mut self, timer: TimerId, milliseconds: u32) {
        unsafe { SetTimer(Some(self.window), self.timer_key(timer), milliseconds, None) };
    }

    pub fn kill_timer(&mut self, timer: TimerId) {
        unsafe {
            let _ = KillTimer(Some(self.window), self.timer_key(timer));
        }
    }

    /// Takes the keyboard from the app in front so `Page::on_key` gets typing; call it only from a click, never on
    /// hover. The notch stays open until `release_keyboard` or `Page::on_keyboard_lost`.
    pub fn take_keyboard(&mut self) {
        self.requests.keyboard = Some((self.owner_index, true));
    }

    /// Gives the keyboard back to the app that had it.
    pub fn release_keyboard(&mut self) {
        self.requests.keyboard = Some((self.owner_index, false));
    }

    pub fn notifier(&self) -> Notifier {
        Notifier { window_address: self.window.0 as isize, owner_index: self.owner_index, message: self.notify_message }
    }

    /// First delivers the current value, then changes; registration lasts until the notch exits.
    pub fn listen_to_power_setting(&self, setting: &GUID) {
        let flags = REGISTER_NOTIFICATION_FLAGS(0);
        let _ = unsafe { RegisterPowerSettingNotification(HANDLE(self.window.0), setting, flags) };
    }

    pub fn listen_to_clipboard(&self) {
        let _ = unsafe { AddClipboardFormatListener(self.window) };
    }

    pub fn upload_image(&self, image: &ImageData) -> Option<ImageBrush> {
        self.surface.create_image_brush(image).ok()
    }

    fn timer_key(&self, timer: TimerId) -> usize {
        owned_timer_key(self.timer_base, self.owner_index, timer)
    }
}

#[cfg(test)]
pub(crate) fn timer_key(page_index: usize, timer: TimerId) -> usize {
    owned_timer_key(PAGE_TIMER_BASE, page_index, timer)
}

pub(crate) fn owned_timer_key(timer_base: usize, owner_index: usize, timer: TimerId) -> usize {
    debug_assert!(usize::from(timer.0) < TIMERS_PER_PAGE, "timer ids must be below {TIMERS_PER_PAGE}");
    timer_base + owner_index * TIMERS_PER_PAGE + usize::from(timer.0)
}

pub(crate) fn split_timer_key(key: usize) -> Option<(usize, TimerId)> {
    split_owned_timer_key(PAGE_TIMER_BASE, PAGE_TIMER_LIMIT, key)
}

pub(crate) fn split_owned_timer_key(timer_base: usize, limit: usize, key: usize) -> Option<(usize, TimerId)> {
    if key >= limit {
        return None;
    }
    let offset = key.checked_sub(timer_base)?;
    Some((offset / TIMERS_PER_PAGE, TimerId((offset % TIMERS_PER_PAGE) as u8)))
}

/// Wakes a page or activity from any thread. Cheap to clone; keep one in the background worker.
#[derive(Clone, Copy, Debug)]
pub struct Notifier {
    // The window handle crosses threads as an integer, since the raw handle type is not thread-safe.
    window_address: isize,
    owner_index: usize,
    message: u32,
}

impl Notifier {
    pub fn notify(&self) {
        let window = HWND(self.window_address as *mut _);
        unsafe {
            let _ = PostMessageW(Some(window), self.message, WPARAM(self.owner_index), LPARAM(0));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priorities_are_ordered() {
        assert!(Priority::BACKGROUND < Priority::NORMAL);
        assert!(Priority::NORMAL < Priority::URGENT);
    }

    #[test]
    fn the_notch_counts_as_open_from_80_percent() {
        let frame = |expansion| Frame { scale: 1.0, bounds: Rect::default(), expansion, expanded: Size::new(0.0, 0.0) };
        assert!(!frame(0.79).is_open());
        assert!(frame(0.8).is_open());
        assert!(frame(1.05).is_open());
    }

    #[test]
    fn timer_keys_round_trip_to_page_and_timer() {
        for (page, timer) in [(0, 0), (0, 15), (1, 0), (3, 7)] {
            let key = timer_key(page, TimerId(timer));
            assert_eq!(split_timer_key(key), Some((page, TimerId(timer))));
        }
    }

    #[test]
    fn timers_of_different_pages_never_collide() {
        assert_ne!(timer_key(0, TimerId(15)), timer_key(1, TimerId(0)));
    }

    #[test]
    fn keys_below_the_page_range_belong_to_the_host() {
        assert_eq!(split_timer_key(0), None);
        assert_eq!(split_timer_key(PAGE_TIMER_BASE - 1), None);
    }

    #[test]
    fn keys_from_the_activity_range_up_are_not_page_timers() {
        assert_eq!(split_timer_key(PAGE_TIMER_LIMIT), None);
        assert!(split_timer_key(PAGE_TIMER_LIMIT - 1).is_some());
    }
}
