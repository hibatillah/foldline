use std::cell::Cell;

use store::language::{Language, resolve_setting};

use windows::Win32::Foundation::{HWND, POINT};

use super::{
    display, navigation,
    navigation::PageList,
    notification_banners,
    page_swap::PageSwap,
    scroll_gate::ScrollGate,
    settings::Settings,
    visibility::{self},
    volume_flyout,
};
use crate::{
    activity::Activity,
    gfx::{Fonts, Spring, Surface},
    log,
    page::{Page, RESTING_HEIGHT, Requests},
};
use animation::{FrameTimer, Springs};
use badge_pointer::BadgePointer;
use header_row::HeaderMarks;
use hiding::HidingState;
use hover::HoverState;
use keyboard::KeyboardState;
use page_hotkeys::PageHotkeys;
use pages::activity_setting_key;
use presences::ShownPresences;
use resting_row::RestingRow;
use row_motion::RowMotion;

mod animation;
mod badge_pointer;
mod header_row;
mod hiding;
mod hover;
mod input;
mod keyboard;
mod layout;
mod location;
mod page_hotkeys;
mod pages;
mod paint;
mod presences;
mod resting_row;
mod row_motion;
mod settings_reload;
mod timers;

#[cfg(test)]
mod tests;

pub(super) const WINDOW_WIDTH: f32 = 1200.0;
/// Room for the tallest page (`page::MAX_CONTENT_HEIGHT`), its header, the drag growth and the shadow.
pub(super) const WINDOW_HEIGHT: f32 = 340.0;
const RESTING_BOTTOM_PADDING: f32 = 3.0;
const RESTING_PILL_HEIGHT: f32 = RESTING_HEIGHT + RESTING_BOTTOM_PADDING;
const SHADOW_LAYERS: usize = 10;
const SHADOW_STEP: f32 = 2.0;
const SHADOW_OFFSET: f32 = 4.0;
const RESTING_FLARE: f32 = 6.0;
const ANIMATION_TIMER_ID: usize = 1;
/// Safety net while the notch is open: the leave notification can be lost when the window resizes under the pointer.
const HOVER_WATCH_TIMER_ID: usize = 2;
const REVEAL_TIMER_ID: usize = 3;
const EXPAND_TIMER_ID: usize = 4;
const HIDDEN_WIDTH: f32 = 240.0;
const HIDDEN_HEIGHT: f32 = 6.0;
const HIDE_WHEN_IDLE_SETTING: &str = "notch.hide_when_idle";
/// The open amount at which the header content is fully shown and the page content has not started; 1 is fully open.
const HEADER_STAGE: f32 = 0.5;
const RESTING_FADE_END: f32 = 0.25;
const HEADER_FADE: (f32, f32) = (0.2, 0.45);
const CONTENT_FADE: (f32, f32) = (0.62, 0.9);
const CLOSE_TIMER_ID: usize = 8;
const RAISE_TIMER_ID: usize = 9;
/// One-shot: a window that stopped covering the screen must stay so for a moment before the notch comes back.
const WINDOW_REVEAL_TIMER_ID: usize = 10;
/// One-shot: network changes come in bursts, so the position is read once they have been quiet for a while.
const LOCATION_SETTLE_TIMER_ID: usize = 11;
const LAST_PAGE_SETTING: &str = "navigation.page";
const PAGE_ORDER_SETTING: &str = "navigation.order";
const PAGES_OFF_SETTING: &str = "navigation.disabled";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Source {
    Activity(usize),
    Page(usize),
}

struct ActivitySlot {
    activity: Box<dyn Activity>,
    is_enabled: bool,
    is_started: bool,
}

pub(super) struct Notch {
    window: HWND,
    scale: f32,
    screen_position: POINT,
    surface: Surface,
    fonts: Fonts,
    settings: Settings,
    /// The Windows display language, read once at start; `auto` resolves to it.
    system_language: Language,
    language: Language,
    pages: Vec<Box<dyn Page>>,
    activities: Vec<ActivitySlot>,
    presences: ShownPresences,
    resting_row: RestingRow,
    header_marks: HeaderMarks,
    resting_index: Option<usize>,
    expanded_index: Option<usize>,
    selected_page: Option<usize>,
    shown_page: Option<usize>,
    page_list: PageList,
    navigable: Vec<usize>,
    track: Spring,
    page_swap: PageSwap,
    scroll_gate: ScrollGate,
    pending: Requests,
    springs: Springs,
    row_motion: RowMotion,
    hiding: HidingState,
    /// Whether the saved city follows the device; see `location_tracker`.
    is_tracking_location: bool,
    has_resting_content: bool,
    hover: HoverState,
    badge_pointer: BadgePointer,
    frame_timer: FrameTimer,
    keyboard: KeyboardState,
    page_hotkeys: PageHotkeys,
    region_origin: Cell<(i32, i32)>,
}

pub(super) struct NotchParts {
    pub window: HWND,
    pub scale: f32,
    pub screen_position: POINT,
    pub surface: Surface,
    pub fonts: Fonts,
    pub settings: Settings,
    pub pages: Vec<Box<dyn Page>>,
    pub activities: Vec<Box<dyn Activity>>,
    pub ticks_per_second: i64,
    pub system_language: Language,
}

impl Notch {
    pub fn new(parts: NotchParts) -> Self {
        let NotchParts {
            window,
            scale,
            screen_position,
            surface,
            fonts,
            mut settings,
            pages,
            activities,
            ticks_per_second,
            system_language,
        } = parts;
        visibility::migrate(&mut settings);
        let activities = activities
            .into_iter()
            .map(|activity| {
                let is_enabled =
                    !activity.is_optional() || settings.get_bool(&activity_setting_key(activity.id()), true);
                ActivitySlot { activity, is_enabled, is_started: false }
            })
            .collect();
        let hiding = HidingState::read(&settings);
        let language = resolve_setting(&settings, system_language);
        Self {
            window,
            scale,
            screen_position,
            surface,
            fonts,
            settings,
            system_language,
            language,
            pages,
            activities,
            presences: ShownPresences::default(),
            resting_row: RestingRow::default(),
            header_marks: HeaderMarks::default(),
            resting_index: None,
            expanded_index: None,
            selected_page: None,
            shown_page: None,
            page_list: PageList::new(0),
            navigable: Vec::new(),
            track: navigation::page_transition(0.0),
            page_swap: PageSwap::default(),
            scroll_gate: ScrollGate::default(),
            pending: Requests::default(),
            springs: Springs::new(),
            row_motion: RowMotion::new(),
            hiding,
            is_tracking_location: false,
            has_resting_content: false,
            hover: HoverState::default(),
            badge_pointer: BadgePointer::default(),
            frame_timer: FrameTimer::new(ticks_per_second, display::frame_interval_milliseconds()),
            keyboard: KeyboardState::default(),
            page_hotkeys: PageHotkeys::default(),
            region_origin: Cell::new((0, 0)),
        }
    }

    pub fn start(&mut self) {
        for (index, page) in self.pages.iter().enumerate() {
            log::debug(&format!("page {index}: {}", page.id()));
        }
        // Pages are built before the window sets the language, so text they cached then is made again now.
        for index in 0..self.pages.len() {
            self.with_page(index, |page, context| page.on_language_changed(context));
        }
        self.selected_page = self
            .settings
            .get_string(LAST_PAGE_SETTING)
            .and_then(|id| self.pages.iter().position(|page| page.id() == id));
        let ids = self.page_ids();
        let opt_in: Vec<bool> = self.pages.iter().map(|page| page.is_opt_in()).collect();
        self.publish_known_pages(&ids, &opt_in);
        self.page_list = PageList::load(
            &ids,
            &opt_in,
            self.settings.get_string(PAGE_ORDER_SETTING),
            self.settings.get_string(PAGES_OFF_SETTING),
        );
        for index in 0..self.pages.len() {
            self.with_page(index, |page, context| page.on_start(context));
            let is_enabled = self.page_list.is_enabled(index);
            self.with_page(index, |page, context| page.on_enabled_changed(is_enabled, context));
        }
        for index in 0..self.activities.len() {
            if self.activities[index].is_enabled {
                self.start_activity(index);
            }
        }
        volume_flyout::set_hidden(volume_flyout::wants_hidden(&self.settings));
        notification_banners::set_hidden(notification_banners::wants_hidden(&self.settings));
        self.sync_location_tracking();
        self.arm_clock_timer();
        self.refresh_layout();
        for spring in self.layout_springs() {
            spring.snap_to_target();
        }
        self.flush();
        self.paint();
    }
}
