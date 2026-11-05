use std::cell::Cell;

use windows::Win32::Foundation::{HWND, POINT};

use super::{
    display, navigation,
    navigation::PageList,
    page_editor::PageEditor,
    scroll_gate::ScrollGate,
    settings::Settings,
    visibility::{self},
};
use crate::{
    activity::Activity,
    gfx::{Fonts, Spring, Surface},
    log,
    page::{Page, RESTING_HEIGHT, Requests},
};
use animation::Springs;
use hiding::{HidingState, read_showing};
use hover::HoverState;
use keyboard::{KeyboardOwner, KeyboardState};
use menu::ContextMenu;
use pages::activity_setting_key;
use settings_ui::PanelState;

mod animation;
mod hiding;
mod hover;
mod input;
mod keyboard;
mod layout;
mod menu;
mod pages;
mod paint;
mod settings_ui;
mod timers;

#[cfg(test)]
mod tests;

pub const WINDOW_WIDTH: f32 = 1200.0;
/// Room for the tallest page (`page::MAX_CONTENT_HEIGHT`), its header, the drag growth and the shadow.
pub const WINDOW_HEIGHT: f32 = 340.0;
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
const PAGE_GAP: f32 = 24.0;
const LAST_PAGE_SETTING: &str = "navigation.page";
const PAGE_ORDER_SETTING: &str = "navigation.order";
const PAGES_OFF_SETTING: &str = "navigation.disabled";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Source {
    Activity(usize),
    Page(usize),
}

#[derive(Clone, Copy)]
enum HeaderMark {
    Page(usize),
    Activity(usize),
}

struct ActivitySlot {
    activity: Box<dyn Activity>,
    is_enabled: bool,
    is_started: bool,
}

pub struct Notch {
    window: HWND,
    scale: f32,
    screen_position: POINT,
    surface: Surface,
    fonts: Fonts,
    settings: Settings,
    pages: Vec<Box<dyn Page>>,
    activities: Vec<ActivitySlot>,
    transient_order: Vec<Source>,
    shown_transients: Vec<(Source, f32)>,
    shown_banner: Option<(Source, f32, f32)>,
    shown_badges: Vec<(Source, f32)>,
    resting_index: Option<usize>,
    expanded_index: Option<usize>,
    selected_page: Option<usize>,
    shown_page: Option<usize>,
    page_list: PageList,
    editor: Option<PageEditor>,
    navigable: Vec<usize>,
    track: Spring,
    scroll_gate: ScrollGate,
    pending: Requests,
    springs: Springs,
    hiding: HidingState,
    has_resting_content: bool,
    hover: HoverState,
    /// The right-click menu while it is open.
    menu: Option<ContextMenu>,
    is_animating: bool,
    keyboard: KeyboardState,
    panel: PanelState,
    last_tick: i64,
    ticks_per_second: i64,
    frame_interval_milliseconds: u32,
    region_origin: Cell<(i32, i32)>,
}

pub struct NotchParts {
    pub window: HWND,
    pub scale: f32,
    pub screen_position: POINT,
    pub surface: Surface,
    pub fonts: Fonts,
    pub settings: Settings,
    pub pages: Vec<Box<dyn Page>>,
    pub activities: Vec<Box<dyn Activity>>,
    pub ticks_per_second: i64,
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
        Self {
            window,
            scale,
            screen_position,
            surface,
            fonts,
            settings,
            pages,
            activities,
            transient_order: Vec::new(),
            shown_transients: Vec::new(),
            shown_banner: None,
            shown_badges: Vec::new(),
            resting_index: None,
            expanded_index: None,
            selected_page: None,
            shown_page: None,
            page_list: PageList::new(0),
            editor: None,
            navigable: Vec::new(),
            track: navigation::page_transition(0.0),
            scroll_gate: ScrollGate::default(),
            pending: Requests::default(),
            springs: Springs::new(),
            hiding,
            has_resting_content: false,
            hover: HoverState::default(),
            menu: None,
            is_animating: false,
            keyboard: KeyboardState::default(),
            panel: PanelState::default(),
            last_tick: 0,
            ticks_per_second,
            frame_interval_milliseconds: display::frame_interval_milliseconds(),
            region_origin: Cell::new((0, 0)),
        }
    }

    pub fn start(&mut self) {
        for (index, page) in self.pages.iter().enumerate() {
            log::debug(&format!("page {index}: {}", page.id()));
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
        }
        for index in 0..self.activities.len() {
            if self.activities[index].is_enabled {
                self.start_activity(index);
            }
        }
        self.arm_clock_timer();
        self.refresh_layout();
        for spring in self.layout_springs() {
            spring.snap_to_target();
        }
        self.flush();
        self.paint();
    }
}
