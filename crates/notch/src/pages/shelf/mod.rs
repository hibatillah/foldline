//! The shelf stores paths; adding, dragging, and removing entries never moves or deletes the originals.

use self::items::{MAX_FILES, Shelf, added_text};
use crate::{
    activity::{FLASH_TIMER, Flash, Presence},
    gfx::{Align, Canvas, Color, FontStyle, ImageBrush, ImageData, Point, Rect, Size, Spring, UI_FONT, Weight},
    page::{Context, Frame, Mode, Page, PageSizes, PointerEvent, Priority, TimerId},
    platform,
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
mod draw;
mod layout;

mod items;

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

const EMPTY_TITLE_HEIGHT: f32 = 17.0;

const EMPTY_DETAIL_HEIGHT: f32 = 14.0;

const EMPTY_GAP: f32 = 3.0;

const EMPTY_TEXT_HEIGHT: f32 = EMPTY_TITLE_HEIGHT + EMPTY_GAP + EMPTY_DETAIL_HEIGHT;

const ARRIVAL_RESPONSE: (f32, f32) = (200.0, 19.0);

const ARRIVAL_STAGGER: f32 = 0.18;

const NAME: FontStyle = FontStyle::new(UI_FONT, 10.0, Weight::Regular, Align::Center);

const EMPTY_TITLE: FontStyle = FontStyle::new(UI_FONT, 13.0, Weight::Medium, Align::Center);

const EMPTY_DETAIL: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Regular, Align::Center);

const BORDER: Color = Color::rgb(0.33, 0.33, 0.375);

const FULL_RED: Color = Color::rgb(1.0, 0.27, 0.23);

const ACCENT: Color = Color::rgb(0.25, 0.56, 1.0);

const NAME_COLOR: Color = Color::rgb(0.769, 0.769, 0.8);

const GRAY: Color = Color::rgb(0.604, 0.604, 0.651);

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

    #[cfg(test)]
    pub fn sample(directory: &std::path::Path, fonts: &crate::gfx::Fonts, is_drag_over: bool) -> Self {
        let mut page = Self::new();
        page.storage = None;
        page.is_drag_over = is_drag_over;
        let files: Vec<PathBuf> =
            ["hover-state-v3.png", "wallpaper-1440p.jpg", "notch-brief.pdf", "budget-q4.xlsx", "notes.txt", "tasks.md"]
                .iter()
                .map(|name| {
                    let path = directory.join(name);
                    let _ = std::fs::write(&path, "x");
                    path
                })
                .collect();
        page.shelf.add(files);
        page.refresh_names(fonts);
        for path in page.shelf.items().iter().take(MAX_FILES) {
            if let Some(image) = platform::file_preview(path, PREVIEW_PIXELS) {
                page.previews.get_mut().insert(path.clone(), Preview::new(image));
            }
        }
        page
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

    fn icon(&self) -> Icon {
        Icon::Folder
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
        let (bounds, scale) = (frame.bounds, frame.scale);
        let is_refusing = self.is_drag_over && self.shelf.is_full();
        let accent = if is_refusing { FULL_RED } else { ACCENT };
        if self.is_drag_over {
            canvas.fill_rounded_rect(bounds, ZONE_RADIUS * scale, accent.with_alpha(0.1 * opacity));
        }
        let border = if self.is_drag_over { accent } else { BORDER };
        ui::draw_dashed_outline(
            canvas,
            bounds,
            ZONE_RADIUS * scale,
            BORDER_WIDTH * scale,
            DASH * scale,
            DASH_GAP * scale,
            border.faded(opacity),
        );

        if is_refusing {
            let area =
                Rect::new(bounds.left, bounds.center_y() - 8.0 * scale, bounds.right, bounds.center_y() + 8.0 * scale);
            canvas.text(&EMPTY_DETAIL, "The shelf is full. Remove a file to add more.", area, FULL_RED.faded(opacity));
            return;
        }
        if self.shelf.len() == 0 {
            let (title, detail, title_color, detail_color) = if self.is_drag_over {
                ("Release to add", "Files stay where they are", Color::WHITE, ACCENT)
            } else {
                ("Drop files here", "Drag them back out when you need them", NAME_COLOR, GRAY)
            };
            let top = bounds.center_y() - EMPTY_TEXT_HEIGHT / 2.0 * scale;
            let title_area = Rect::new(bounds.left, top, bounds.right, top + EMPTY_TITLE_HEIGHT * scale);
            let detail_top = title_area.bottom + EMPTY_GAP * scale;
            let detail_area =
                Rect::new(bounds.left, detail_top, bounds.right, detail_top + EMPTY_DETAIL_HEIGHT * scale);
            canvas.text(&EMPTY_TITLE, title, title_area, title_color.faded(opacity));
            canvas.text(&EMPTY_DETAIL, detail, detail_area, detail_color.faded(opacity));
            return;
        }
        for position in 0..shown_count(self.shelf.len()) {
            self.draw_slot(canvas, position, bounds, opacity);
        }
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
        let (bounds, scale, count) = (frame.bounds, frame.scale, self.shelf.len());
        match event {
            PointerEvent::Moved(point) => {
                let hovered = target_at(point, bounds, scale, count);
                if hovered != self.hovered {
                    self.hovered = hovered;
                    context.request_repaint();
                }
                let Some(press) = self.press else { return };
                let Target::Slot(position) = press.target else { return };
                let moved = (point.x - press.origin.x).hypot(point.y - press.origin.y);
                if moved >= DRAG_START_DISTANCE * scale
                    && let Some(index) = file_index(position, count)
                {
                    self.press = None;
                    self.begin_drag(index, context);
                }
            }
            PointerEvent::Pressed(point) if frame.is_open() => {
                self.press = target_at(point, bounds, scale, count).map(|target| Press { target, origin: point });
            }
            PointerEvent::Released(point) => {
                let Some(press) = self.press.take() else { return };
                if !frame.is_open() || target_at(point, bounds, scale, count) != Some(press.target) {
                    return;
                }
                match press.target {
                    Target::Remove(position) => {
                        if let Some(index) = file_index(position, count) {
                            self.remove(index, context);
                        }
                    }
                    Target::Slot(position) => {
                        let Some(index) = file_index(position, count) else { return };
                        let path = self.shelf.items()[index].clone();
                        let (now, limit) = (Instant::now(), platform::double_click_time());
                        if is_double_click(self.last_click.as_ref(), &path, now, limit) {
                            self.last_click = None;
                            platform::open(&path.display().to_string());
                        } else {
                            self.last_click = Some((path, now));
                        }
                    }
                }
            }
            PointerEvent::SecondaryReleased(point) if frame.is_open() => {
                let Some(Target::Slot(position) | Target::Remove(position)) = target_at(point, bounds, scale, count)
                else {
                    return;
                };
                if let Some(index) = file_index(position, count) {
                    self.remove(index, context);
                }
            }
            PointerEvent::Left | PointerEvent::CaptureLost => {
                self.press = None;
                if self.hovered.take().is_some() {
                    context.request_repaint();
                }
            }
            _ => {}
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shelf_is_a_page_to_open_that_takes_drops() {
        let page = ShelfPage::new();
        assert!(page.accepts_drops());
        assert_eq!(page.priority(Mode::Resting), None);
        assert!(page.priority(Mode::Expanded).is_some());
        assert_eq!(page.presence(), None);
    }

    #[test]
    fn a_new_file_arrives_and_then_the_page_stops_animating() {
        let mut page = ShelfPage::new();
        let mut spring = Spring::new(0.0, ARRIVAL_RESPONSE.0, ARRIVAL_RESPONSE.1);
        spring.target = 1.0;
        page.arrivals.insert(PathBuf::from("a.txt"), spring);
        assert!(page.animate(0.016));
        for _ in 0..200 {
            page.animate(0.016);
        }
        assert!(!page.animate(0.016));
        assert!(page.arrivals.is_empty());
    }
}
