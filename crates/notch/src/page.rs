//! Host/page contracts; see docs/how-to/add-a-page.md.

use windows::{
    Win32::{
        Foundation::{HANDLE, HWND, LPARAM, WPARAM},
        System::{DataExchange::AddClipboardFormatListener, Power::RegisterPowerSettingNotification},
        UI::WindowsAndMessaging::{KillTimer, PostMessageW, REGISTER_NOTIFICATION_FLAGS, SetTimer, WM_APP},
    },
    core::GUID,
};

use crate::{
    activity::{Presence, SystemMessage},
    gfx::{Canvas, Fonts, ImageBrush, ImageData, Point, Rect, Size, Surface},
    host::Settings,
};

mod timer_keys;

pub use crate::ui::{KeyInput, PointerEvent};
#[cfg(test)]
pub(crate) use timer_keys::timer_key;
pub(crate) use timer_keys::{owned_timer_key, split_owned_timer_key, split_timer_key};

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
    /// Units, starting at zero (device pixels in `draw_backdrop`); pointer input uses the same space.
    pub bounds: Rect,
    /// 0 collapsed, 1 fully open; the spring can overshoot a little past 1.
    pub expansion: f32,
    /// The content size in device-independent pixels; visible pages retain their own width during navigation.
    pub expanded: Size,
    /// Width in DIP the host keeps free after `Page::resting_lead_width` while the page claims the pill:
    /// `Page::resting_lead_gap`, or 24.
    pub lead_gap: f32,
}

impl Frame {
    pub fn is_open(&self) -> bool {
        self.expansion >= 0.8
    }
}

/// Who holds the keyboard; a page and an activity each count from zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyboardOwner {
    Page(usize),
    Activity(usize),
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

    /// The notch opened or closed (by the pointer, a page or the pin); all pages receive it, including those not shown.
    fn on_open_changed(&mut self, _is_open: bool, _context: &mut Context) {}

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
    /// `PointerEvent::SecondaryReleased`; otherwise the right-click does nothing.
    fn uses_right_click(&self, _point: Point, _frame: &Frame) -> bool {
        false
    }

    /// Whether the page sits at the left padding when the open notch is wider than it, instead of centered.
    fn is_left_aligned(&self) -> bool {
        false
    }

    /// Whether the vertical wheel at `point` scrolls something on this page instead of switching pages.
    fn claims_scroll(&self, _point: Point, _frame: &Frame) -> bool {
        false
    }

    fn on_timer(&mut self, _timer: TimerId, _context: &mut Context) {}

    /// Typing, delivered only while this page holds the keyboard.
    fn on_key(&mut self, _key: KeyInput, _context: &mut Context) {}

    /// The keyboard went elsewhere without the page releasing it (a click in another window): stop editing.
    fn on_keyboard_lost(&mut self, _context: &mut Context) {}

    /// A background thread called this page's `Notifier`: read the shared state and request a repaint.
    fn on_notify(&mut self, _context: &mut Context) {}

    /// A Windows message a page asked to hear about (`Context::listen_to_clipboard`); only pages that are switched on
    /// receive it, so a page that is off never sees what the user copies.
    fn on_system_message(&mut self, _message: &SystemMessage, _context: &mut Context) {}

    /// The user switched this page on or off in the settings window; also called once at start with the current state.
    /// A page that holds personal data drops it when `is_enabled` is false.
    fn on_enabled_changed(&mut self, _is_enabled: bool, _context: &mut Context) {}

    /// Called after another program changed settings.txt and the host re-read it: re-read this page's keys here.
    fn on_settings_changed(&mut self, _context: &mut Context) {}

    /// The language changed, or the notch moved to a monitor of another scale: rebuild any cached text. Called once per change, after `on_settings_changed`.
    fn on_language_changed(&mut self, _context: &mut Context) {}

    /// Collapsed activity content, independent of page selection; re-evaluated after every event.
    fn presence(&self) -> Option<Presence> {
        None
    }

    /// Width in DIP of what the claimed pill draws at its left end; the host leaves `Frame::lead_gap` after it.
    fn resting_lead_width(&self) -> f32 {
        0.0
    }

    /// Room in DIP the host keeps between the lead and the rest of the claimed pill, passed as `Frame::lead_gap`;
    /// `None` keeps the host's 24.
    fn resting_lead_gap(&self) -> Option<f32> {
        None
    }

    /// Draws what `presence` asked for into `area` (units, starting at zero), faded by `opacity`.
    fn draw_presence(&self, _canvas: &Canvas, _area: Rect, _opacity: f32) {}

    /// Pointer input over this page's banner or badge while the notch is collapsed; `area` is what the presence was
    /// drawn with. `PointerEvent::Left` means the pointer left it.
    fn on_presence_pointer(&mut self, _event: PointerEvent, _area: Rect, _context: &mut Context) {}

    /// Width in DIP of the badge in its short form, used when several badges share the pill; `None` has no short form.
    fn short_presence_width(&self) -> Option<f32> {
        None
    }

    /// Draws the badge in its short form into `area`; the long form when the page has no short one.
    fn draw_short_presence(&self, canvas: &Canvas, area: Rect, opacity: f32) {
        self.draw_presence(canvas, area, opacity);
    }

    /// The id of the page a click on this page's header status opens; `None` when it opens nothing.
    fn header_status_page(&self) -> Option<&'static str> {
        None
    }

    /// Seconds left on the countdown this page shows as a badge or header mark; `None` when it shows no countdown.
    fn countdown_seconds(&self) -> Option<u64> {
        None
    }

    /// Draws behind the header and content; pill is the whole notch, frame is this page's content. Unlike every other
    /// draw hook it draws in device pixels: bounds and radius are device pixels and `canvas.scale()` is the display scale.
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

    /// Draws the status mark into `area` (units, starting at zero, as high as the header), faded by `opacity`.
    fn draw_header_status(&self, _canvas: &Canvas, _area: Rect, _opacity: f32) {}

    /// What the settings window's page list calls the page.
    fn name(&self) -> String {
        self.id().to_owned()
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
    /// The page or activity that asked for the keyboard (`true`) or gave it back (`false`).
    pub keyboard: Option<(KeyboardOwner, bool)>,
    /// A page asked the notch to close, for example after it pasted into another window.
    pub close: bool,
    /// A page asked the notch to open on it, for example from a button on its banner.
    pub open_page: Option<usize>,
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
        self.requests.keyboard = Some((self.keyboard_owner(), true));
    }

    /// Gives the keyboard back to the app that had it.
    pub fn release_keyboard(&mut self) {
        self.requests.keyboard = Some((self.keyboard_owner(), false));
    }

    fn keyboard_owner(&self) -> KeyboardOwner {
        if self.timer_base == PAGE_TIMER_BASE {
            KeyboardOwner::Page(self.owner_index)
        } else {
            KeyboardOwner::Activity(self.owner_index)
        }
    }

    /// Closes the open notch as if the pointer had left it; does nothing while the user has pinned it open.
    pub fn request_close(&mut self) {
        self.requests.close = true;
    }

    /// Opens the notch on this page; an activity calling it changes nothing, since it has no page to open.
    pub fn request_open(&mut self) {
        if self.timer_base == PAGE_TIMER_BASE {
            self.requests.open_page = Some(self.owner_index);
        }
    }

    /// The notch window, for the few Windows calls that need an owner (writing the clipboard).
    pub(crate) fn window(&self) -> HWND {
        self.window
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
        let frame =
            |expansion| Frame { bounds: Rect::default(), expansion, expanded: Size::new(0.0, 0.0), lead_gap: 0.0 };
        assert!(!frame(0.79).is_open());
        assert!(frame(0.8).is_open());
        assert!(frame(1.05).is_open());
    }
}
