use self::items::{MAX_FILES, Shelf, added_text};
use crate::{
    activity::{FLASH_TIMER, Flash, Presence},
    gfx::{Canvas, Color, ImageBrush, ImageData, Point, Rect, Size, Spring},
    page::{Context, Frame, Mode, Page, PageSizes, PointerEvent, Priority, TimerId},
    ui::{self, Icon},
};
use layout::{
    Target, content_height, file_index, fitted, is_double_click, name_lines, preview_rect, remove_center, shown_count,
    slot_rect, target_at,
};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Instant,
};

mod actions;
mod document;
mod draw;
mod empty;
mod input;
mod items;
mod layout;
#[cfg(test)]
mod sample;
#[cfg(test)]
mod tests;

const EXPANDED_WIDTH: f32 =
    2.0 * ZONE_PADDING + SLOTS_PER_ROW as f32 * FILE_WIDTH + (SLOTS_PER_ROW - 1) as f32 * FILE_GAP;
const RESTING_WIDTH: f32 = 150.0;
const ZONE_RADIUS: f32 = 14.0;
const BORDER_WIDTH: f32 = 1.5;
const DASH: f32 = 10.0;
const DASH_GAP: f32 = 6.0;
const ZONE_PADDING: f32 = 9.5;
const FILE_WIDTH: f32 = 78.0;
const FILE_GAP: f32 = 4.0;
const NAME_TOP: f32 = 3.0;
const NAME_LINE_HEIGHT: f32 = 12.0;

/// Files in a row; the seventh starts a second row.
const SLOTS_PER_ROW: usize = 6;
const ROW_GAP: f32 = 6.0;
const DRAG_START_DISTANCE: f32 = 5.0;

/// Pixels asked of Explorer for each preview; enough for a 40 px tile on a screen scaled to 200 %.
const PREVIEW_PIXELS: i32 = 96;
const ARRIVAL_RESPONSE: (f32, f32) = (200.0, 19.0);
const ARRIVAL_STAGGER: f32 = 0.18;
const BORDER: Color = Color::rgb(0.353, 0.353, 0.408);
const FULL_BORDER: Color = Color::rgb(0.839, 0.271, 0.239);
const FULL_FILL: Color = Color::rgb(1.0, 0.271, 0.227);
const ACCENT_FILL_ALPHA: f32 = 0.12;
const FULL_FILL_ALPHA: f32 = 0.08;
const NAME_COLOR: Color = Color::rgb(0.839, 0.839, 0.871);
#[derive(Clone, Copy, Debug)]
struct Press {
    target: Target,
    origin: Point,
}

struct Preview {
    size: (u32, u32),
    waiting: Option<ImageData>,
    brush: Option<ImageBrush>,
}

#[derive(Default)]
struct Inbox {
    previews: Vec<(PathBuf, Option<ImageData>)>,
    drags: Vec<(PathBuf, bool)>,
}

pub struct ShelfPage {
    shelf: Shelf,
    storage: Option<PathBuf>,
    alert: Flash,
    alert_title: String,
    alert_value: String,
    alert_width: f32,
    hovered: Option<Target>,
    press: Option<Press>,
    is_drag_over: bool,
    dragging: Option<PathBuf>,
    names: Vec<Vec<String>>,
    previews: RefCell<HashMap<PathBuf, Preview>>,
    arrivals: HashMap<PathBuf, Spring>,
    last_click: Option<(PathBuf, Instant)>,
    requested: HashSet<PathBuf>,
    inbox: Arc<Mutex<Inbox>>,
}

impl ShelfPage {
    pub fn new() -> Self {
        let storage =
            std::env::var_os("APPDATA").map(|directory| PathBuf::from(directory).join("notch").join("shelf.txt"));
        Self {
            shelf: Shelf::default(),
            storage,
            alert: Flash::default(),
            alert_title: String::new(),
            alert_value: String::new(),
            alert_width: 0.0,
            hovered: None,
            press: None,
            is_drag_over: false,
            dragging: None,
            names: Vec::new(),
            previews: RefCell::new(HashMap::new()),
            arrivals: HashMap::new(),
            last_click: None,
            requested: HashSet::new(),
            inbox: Arc::default(),
        }
    }
}

impl Preview {
    fn new(image: ImageData) -> Self {
        Self { size: (image.width, image.height), waiting: Some(image), brush: None }
    }
}

impl Page for ShelfPage {
    fn id(&self) -> &'static str {
        "shelf"
    }

    fn name(&self) -> &'static str {
        "File shelf"
    }

    fn priority(&self, mode: Mode) -> Option<Priority> {
        (mode == Mode::Expanded).then_some(Priority::BACKGROUND)
    }

    fn sizes(&self) -> PageSizes {
        PageSizes {
            resting_width: RESTING_WIDTH,
            expanded: Size::new(EXPANDED_WIDTH, content_height(self.shelf.len())),
        }
    }

    fn draw_resting(&self, _canvas: &Canvas, _frame: &Frame, _opacity: f32) {}

    fn draw_expanded(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        self.draw_zone(canvas, frame, opacity);
    }

    fn presence(&self) -> Option<Presence> {
        self.alert.is_visible().then_some(Presence::Transient { width: self.alert_width })
    }

    fn draw_presence(&self, canvas: &Canvas, area: Rect, _scale: f32, opacity: f32) {
        ui::draw_message(canvas, area, opacity, Icon::Folder, Color::WHITE, &self.alert_title, Some(&self.alert_value));
    }

    fn accepts_drops(&self) -> bool {
        true
    }

    fn on_file_drag_changed(&mut self, is_over: bool, context: &mut Context) {
        if self.is_drag_over != is_over {
            self.is_drag_over = is_over;
            context.request_repaint();
        }
    }

    fn on_drop(&mut self, paths: Vec<PathBuf>, context: &mut Context) {
        let before = self.shelf.len();
        let added = self.shelf.add(paths);
        if added > 0 {
            for (order, path) in self.shelf.items()[before..].iter().enumerate() {
                // Starting below zero holds each later file back until its turn.
                let mut spring = Spring::new(-ARRIVAL_STAGGER * order as f32, ARRIVAL_RESPONSE.0, ARRIVAL_RESPONSE.1);
                spring.target = 1.0;
                self.arrivals.insert(path.clone(), spring);
            }
            context.request_animation();
            self.shelf_changed(context);
            self.announce(added_text(added), context);
        }
        context.request_repaint();
    }

    fn on_start(&mut self, context: &mut Context) {
        if let Some(storage) = &self.storage {
            self.shelf = Shelf::load(storage);
        }
        self.refresh_names(context.fonts());
        self.request_previews(context);
    }

    fn on_notify(&mut self, context: &mut Context) {
        let (previews, drags) = match self.inbox.lock() {
            Ok(mut inbox) => (std::mem::take(&mut inbox.previews), std::mem::take(&mut inbox.drags)),
            Err(_) => return,
        };
        for (path, image) in previews {
            if let Some(image) = image
                && self.shelf.items().contains(&path)
            {
                self.previews.get_mut().insert(path, Preview::new(image));
            }
        }
        for (path, was_taken) in drags {
            if self.dragging.as_ref() == Some(&path) {
                self.dragging = None;
            }
            if was_taken && let Some(index) = self.shelf.items().iter().position(|item| *item == path) {
                self.remove(index, context);
            }
        }
        context.request_repaint();
    }

    fn on_pointer(&mut self, event: PointerEvent, frame: &Frame, context: &mut Context) {
        self.handle_pointer(event, frame, context);
    }

    fn uses_right_click(&self, point: Point, frame: &Frame) -> bool {
        frame.is_open() && target_at(point, frame.bounds, frame.scale, self.shelf.len()).is_some()
    }

    fn on_timer(&mut self, timer: TimerId, context: &mut Context) {
        if timer == FLASH_TIMER {
            self.alert.on_timer(timer, context);
        }
    }

    fn animate(&mut self, elapsed_seconds: f32) -> bool {
        for spring in self.arrivals.values_mut() {
            spring.step(elapsed_seconds);
        }
        self.arrivals.retain(|_, spring| !spring.is_settled());
        !self.arrivals.is_empty()
    }
}
