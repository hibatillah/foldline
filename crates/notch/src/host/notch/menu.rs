//! The right-click menu: Settings, Edit pages, Start with Windows and Quit, drawn inside the open notch.

use windows::Win32::UI::WindowsAndMessaging::PostQuitMessage;

use super::Notch;
use crate::{
    gfx::{Align, Canvas, Color, FontStyle, Point, Rect, UI_FONT, Weight},
    host::{settings_panel::Tab, startup},
};

const MENU_WIDTH: f32 = 188.0;
const MENU_PADDING: f32 = 5.0;
const MENU_RADIUS: f32 = 10.0;
const ITEM_HEIGHT: f32 = 26.0;
const ITEM_RADIUS: f32 = 6.0;
const ITEM_PADDING: f32 = 10.0;
const SEPARATOR_HEIGHT: f32 = 9.0;
/// Space kept between the menu and the notch's edge.
const EDGE_MARGIN: f32 = 6.0;

const ITEM_TEXT: FontStyle = FontStyle::new(UI_FONT, 13.0, Weight::Regular, Align::Leading);
const MENU_FILL: Color = Color::rgb(0.11, 0.11, 0.129);
const MENU_BORDER: Color = Color::rgb(0.2, 0.2, 0.231);
const ITEM_HOVERED: Color = Color::rgb(0.165, 0.165, 0.192);
const SEPARATOR: Color = Color::rgb(0.2, 0.2, 0.231);
const QUIT_RED: Color = Color::rgb(1.0, 0.27, 0.23);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MenuAction {
    Settings,
    EditPages,
    StartWithWindows,
    Quit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MenuItem {
    label: &'static str,
    action: MenuAction,
    /// A check mark after the label, for the one setting the menu shows.
    is_checked: bool,
    /// A line above this item.
    has_separator: bool,
}

pub(super) struct ContextMenu {
    area: Rect,
    items: Vec<MenuItem>,
    hovered: Option<usize>,
}

fn items(starts_with_windows: bool) -> Vec<MenuItem> {
    let item = |label, action, is_checked, has_separator| MenuItem { label, action, is_checked, has_separator };
    vec![
        item("Settings\u{2026}", MenuAction::Settings, false, false),
        item("Edit pages", MenuAction::EditPages, false, false),
        item("Start with Windows", MenuAction::StartWithWindows, starts_with_windows, false),
        item("Quit Notch", MenuAction::Quit, false, true),
    ]
}

fn menu_height(items: &[MenuItem]) -> f32 {
    let separators = items.iter().filter(|item| item.has_separator).count() as f32;
    2.0 * MENU_PADDING + items.len() as f32 * ITEM_HEIGHT + separators * SEPARATOR_HEIGHT
}

impl ContextMenu {
    /// A menu whose corner is at `point`, moved to stay inside `bounds`.
    pub(super) fn open(point: Point, bounds: Rect, scale: f32, starts_with_windows: bool) -> Self {
        let items = items(starts_with_windows);
        let (width, height) = (MENU_WIDTH * scale, menu_height(&items) * scale);
        let margin = EDGE_MARGIN * scale;
        let left = point.x.min(bounds.right - width - margin).max(bounds.left + margin);
        let top = point.y.min(bounds.bottom - height - margin).max(bounds.top + margin);
        Self { area: Rect::new(left, top, left + width, top + height), items, hovered: None }
    }

    fn item_rect(&self, position: usize, scale: f32) -> Rect {
        let above = &self.items[..position];
        let separators = above.iter().filter(|item| item.has_separator).count() as f32
            + f32::from(u8::from(self.items[position].has_separator));
        let top =
            self.area.top + (MENU_PADDING + position as f32 * ITEM_HEIGHT + separators * SEPARATOR_HEIGHT) * scale;
        Rect::new(
            self.area.left + MENU_PADDING * scale,
            top,
            self.area.right - MENU_PADDING * scale,
            top + ITEM_HEIGHT * scale,
        )
    }

    pub(super) fn contains(&self, point: Point) -> bool {
        self.area.contains(point)
    }

    fn item_at(&self, point: Point, scale: f32) -> Option<usize> {
        (0..self.items.len()).find(|&position| self.item_rect(position, scale).contains(point))
    }

    /// Moves the highlight; true when it changed.
    pub(super) fn moved(&mut self, point: Point, scale: f32) -> bool {
        let hovered = self.item_at(point, scale);
        let changed = hovered != self.hovered;
        self.hovered = hovered;
        changed
    }

    pub(super) fn action_at(&self, point: Point, scale: f32) -> Option<MenuAction> {
        self.item_at(point, scale).map(|position| self.items[position].action)
    }

    pub(super) fn draw(&self, canvas: &Canvas, opacity: f32) {
        let scale = canvas.scale();
        canvas.fill_rounded_rect(self.area, MENU_RADIUS * scale, MENU_BORDER.faded(opacity));
        let inner =
            Rect::new(self.area.left + scale, self.area.top + scale, self.area.right - scale, self.area.bottom - scale);
        canvas.fill_rounded_rect(inner, (MENU_RADIUS - 1.0) * scale, MENU_FILL.faded(opacity));
        for (position, item) in self.items.iter().enumerate() {
            let rect = self.item_rect(position, scale);
            if item.has_separator {
                let line_top = rect.top - (SEPARATOR_HEIGHT / 2.0) * scale;
                let line = Rect::new(
                    self.area.left + MENU_PADDING * scale,
                    line_top,
                    self.area.right - MENU_PADDING * scale,
                    line_top + scale,
                );
                canvas.fill_rounded_rect(line, 0.0, SEPARATOR.faded(opacity));
            }
            if self.hovered == Some(position) {
                canvas.fill_rounded_rect(rect, ITEM_RADIUS * scale, ITEM_HOVERED.faded(opacity));
            }
            let color = if item.action == MenuAction::Quit { QUIT_RED } else { Color::WHITE };
            let label = if item.is_checked { format!("{} \u{2713}", item.label) } else { item.label.to_owned() };
            let text_area = Rect { left: rect.left + ITEM_PADDING * scale, ..rect };
            canvas.text(&ITEM_TEXT, &label, text_area, color.faded(opacity));
        }
    }
}

impl Notch {
    pub(super) fn open_menu(&mut self, point: Point) {
        let frame = self.frame();
        self.menu = Some(ContextMenu::open(point, frame.bounds, self.scale, startup::is_enabled()));
        self.pending.repaint = true;
    }

    pub(super) fn close_menu(&mut self) {
        if self.menu.take().is_some() {
            self.pending.repaint = true;
        }
    }

    pub(super) fn run_menu_action(&mut self, action: MenuAction) {
        self.close_menu();
        match action {
            MenuAction::Settings => self.open_panel(),
            MenuAction::EditPages => {
                self.open_panel();
                self.panel.tab = Tab::Pages;
            }
            MenuAction::StartWithWindows => startup::set_enabled(!startup::is_enabled()),
            MenuAction::Quit => unsafe { PostQuitMessage(0) },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOUNDS: Rect = Rect::new(100.0, 0.0, 700.0, 150.0);

    #[test]
    fn the_menu_lists_settings_pages_startup_and_quit_with_the_check_on_startup() {
        let labels: Vec<_> = items(true).iter().map(|item| (item.label, item.is_checked)).collect();
        assert_eq!(
            labels,
            [("Settings\u{2026}", false), ("Edit pages", false), ("Start with Windows", true), ("Quit Notch", false)]
        );
        assert!(items(false).iter().all(|item| !item.is_checked));
    }

    #[test]
    fn the_menu_opens_at_the_pointer_when_there_is_room() {
        let menu = ContextMenu::open(Point::new(200.0, 20.0), BOUNDS, 1.0, false);
        assert_eq!((menu.area.left, menu.area.top), (200.0, 20.0));
    }

    #[test]
    fn the_menu_moves_to_stay_inside_the_notch() {
        let menu = ContextMenu::open(Point::new(690.0, 140.0), BOUNDS, 1.0, false);
        assert!(menu.area.right <= BOUNDS.right - EDGE_MARGIN + 0.01);
        assert!(menu.area.bottom <= BOUNDS.bottom - EDGE_MARGIN + 0.01);
        let menu = ContextMenu::open(Point::new(-50.0, -50.0), BOUNDS, 1.0, false);
        assert!(menu.area.left >= BOUNDS.left && menu.area.top >= BOUNDS.top);
    }

    #[test]
    fn a_press_finds_the_item_under_it_and_nothing_between_or_outside() {
        let menu = ContextMenu::open(Point::new(200.0, 20.0), BOUNDS, 1.0, false);
        for (position, expected) in
            [MenuAction::Settings, MenuAction::EditPages, MenuAction::StartWithWindows, MenuAction::Quit]
                .into_iter()
                .enumerate()
        {
            let rect = menu.item_rect(position, 1.0);
            assert_eq!(menu.action_at(Point::new(rect.center_x(), rect.center_y()), 1.0), Some(expected));
        }
        assert_eq!(menu.action_at(Point::new(10.0, 10.0), 1.0), None);
        let gap = menu.item_rect(3, 1.0).top - 1.0;
        assert_eq!(menu.action_at(Point::new(menu.area.left + 20.0, gap), 1.0), None, "the separator is not an item");
    }

    #[test]
    fn the_highlight_follows_the_pointer_and_reports_a_change_once() {
        let mut menu = ContextMenu::open(Point::new(200.0, 20.0), BOUNDS, 1.0, false);
        let rect = menu.item_rect(1, 1.0);
        let inside = Point::new(rect.center_x(), rect.center_y());
        assert!(menu.moved(inside, 1.0));
        assert!(!menu.moved(inside, 1.0));
        assert!(menu.moved(Point::new(0.0, 0.0), 1.0));
    }

    #[test]
    fn the_menu_fits_inside_the_smallest_open_notch() {
        let smallest_open_height = crate::host::chrome::HEIGHT_EXTRA + crate::page::DEFAULT_CONTENT_HEIGHT;
        assert!(menu_height(&items(false)) + 2.0 * EDGE_MARGIN <= smallest_open_height);
    }
}
