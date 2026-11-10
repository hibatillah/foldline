//! Where everything sits, in the design's device-independent units, and what a click lands on.

use std::time::{Duration, Instant};

use crate::gfx::{Point, Rect};

pub(super) const CONTENT_WIDTH: f32 = 508.0;
pub(super) const CONTENT_HEIGHT: f32 = 179.0;
pub(super) const LIST_WIDTH: f32 = 214.0;
const COLUMN_GAP: f32 = 12.0;
pub(super) const ROW_HEIGHT: f32 = 23.0;
const ROW_GAP: f32 = 3.0;
pub(super) const BAR_HEIGHT: f32 = 26.0;
const BAR_UNDO_WIDTH: f32 = 64.0;
const ACTION_HEIGHT: f32 = 26.0;
/// The gap above the action row (6) plus its top margin (2).
const PANEL_GAP: f32 = 8.0;
pub(super) const BUTTON_SIZE: f32 = 26.0;
const BUTTON_GAP: f32 = 6.0;
/// "Paste" and "Pasted" share one width so the button does not jump when it turns green.
const PASTE_WIDTH: f32 = 76.0;
/// How far the pointer may move between the two presses of a double click.
pub(super) const DOUBLE_CLICK_SLOP: f32 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Target {
    Row(usize),
    Delete,
    Copy,
    Paste,
    Undo,
}

#[derive(Clone, Copy)]
pub(super) struct Layout {
    bounds: Rect,
}

impl Layout {
    pub fn new(bounds: Rect) -> Self {
        Self { bounds }
    }

    fn column(&self, left: f32, top: f32, width: f32, height: f32) -> Rect {
        let (left, top) = (self.bounds.left + left, self.bounds.top + top);
        Rect::new(left, top, left + width, top + height)
    }

    pub fn row(&self, index: usize) -> Rect {
        self.column(0.0, index as f32 * (ROW_HEIGHT + ROW_GAP), LIST_WIDTH, ROW_HEIGHT)
    }

    /// Sits over the bottom of the list, covering the last row; follows the page bottom, which is shorter when empty.
    pub fn bar(&self) -> Rect {
        self.column(0.0, self.bounds.height() - BAR_HEIGHT, LIST_WIDTH, BAR_HEIGHT)
    }

    fn undo_area(&self) -> Rect {
        let bar = self.bar();
        Rect { left: bar.right - BAR_UNDO_WIDTH, ..bar }
    }

    fn right_left(&self) -> f32 {
        LIST_WIDTH + COLUMN_GAP
    }

    pub fn right_width(&self) -> f32 {
        self.bounds.width() - self.right_left()
    }

    pub fn panel(&self) -> Rect {
        self.column(self.right_left(), 0.0, self.right_width(), CONTENT_HEIGHT - ACTION_HEIGHT - PANEL_GAP)
    }

    fn actions(&self) -> Rect {
        self.column(self.right_left(), CONTENT_HEIGHT - ACTION_HEIGHT, self.right_width(), ACTION_HEIGHT)
    }

    pub fn paste_button(&self) -> Rect {
        let actions = self.actions();
        Rect { left: actions.right - PASTE_WIDTH, ..actions }
    }

    pub fn copy_button(&self) -> Rect {
        let paste = self.paste_button();
        let right = paste.left - BUTTON_GAP;
        Rect { left: right - BUTTON_SIZE, right, ..paste }
    }

    pub fn delete_button(&self) -> Rect {
        let copy = self.copy_button();
        let right = copy.left - BUTTON_GAP;
        Rect { left: right - BUTTON_SIZE, right, ..copy }
    }

    /// The two-line details, left of the buttons.
    pub fn details(&self) -> Rect {
        let actions = self.actions();
        Rect { right: self.delete_button().left - BUTTON_GAP, ..actions }
    }

    /// What a press at `point` lands on. A bar over the list takes the clicks on it, and the buttons exist only while
    /// the list has an item.
    pub fn hit(&self, point: Point, row_count: usize, has_bar: bool, has_undo: bool) -> Option<Target> {
        if has_bar && self.bar().contains(point) {
            return (has_undo && self.undo_area().contains(point)).then_some(Target::Undo);
        }
        if let Some(index) = (0..row_count).find(|&index| self.row(index).contains(point)) {
            return Some(Target::Row(index));
        }
        if row_count == 0 {
            return None;
        }
        [
            (Target::Delete, self.delete_button()),
            (Target::Copy, self.copy_button()),
            (Target::Paste, self.paste_button()),
        ]
        .into_iter()
        .find_map(|(target, area)| area.contains(point).then_some(target))
    }
}

/// Derives a double click from two presses, since the host reports only presses and releases.
#[derive(Default)]
pub(super) struct DoubleClick {
    last: Option<(Instant, Point)>,
}

impl DoubleClick {
    /// Records a press; true when it completes a double click, which then needs a fresh pair for the next one.
    pub fn press(&mut self, now: Instant, point: Point, limit: Duration, slop: f32) -> bool {
        let is_double = self.last.is_some_and(|(then, first)| {
            now.saturating_duration_since(then) <= limit
                && (point.x - first.x).abs() <= slop
                && (point.y - first.y).abs() <= slop
        });
        self.last = if is_double { None } else { Some((now, point)) };
        is_double
    }

    pub fn reset(&mut self) {
        self.last = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout() -> Layout {
        Layout::new(Rect::new(100.0, 50.0, 100.0 + CONTENT_WIDTH, 50.0 + CONTENT_HEIGHT))
    }

    #[test]
    fn seven_rows_fill_the_list_height_exactly() {
        let layout = layout();
        assert_eq!(layout.row(0).top, 50.0);
        assert_eq!(layout.row(6).bottom, 50.0 + CONTENT_HEIGHT);
        assert_eq!(layout.row(1).top - layout.row(0).bottom, 3.0);
    }

    #[test]
    fn the_preview_sits_right_of_the_list_with_a_twelve_pixel_gap() {
        let layout = layout();
        assert_eq!(layout.panel().left - layout.row(0).right, 12.0);
        assert_eq!(layout.panel().right, 100.0 + CONTENT_WIDTH);
    }

    #[test]
    fn the_buttons_run_delete_copy_paste_from_the_right_edge_with_six_pixel_gaps() {
        let layout = layout();
        assert_eq!(layout.paste_button().right, 100.0 + CONTENT_WIDTH);
        assert_eq!(layout.paste_button().left - layout.copy_button().right, 6.0);
        assert_eq!(layout.copy_button().left - layout.delete_button().right, 6.0);
        assert_eq!(layout.delete_button().width(), 26.0);
        assert!(layout.details().right <= layout.delete_button().left);
        assert_eq!(layout.paste_button().bottom, 50.0 + CONTENT_HEIGHT);
    }

    #[test]
    fn the_preview_ends_above_the_action_row() {
        let layout = layout();
        assert_eq!(layout.paste_button().top - layout.panel().bottom, 8.0);
    }

    #[test]
    fn a_press_lands_on_the_row_button_or_nothing() {
        let layout = layout();
        let center = |rect: Rect| Point::new(rect.center_x(), rect.center_y());
        assert_eq!(layout.hit(center(layout.row(2)), 3, false, false), Some(Target::Row(2)));
        assert_eq!(layout.hit(center(layout.row(4)), 3, false, false), None);
        assert_eq!(layout.hit(center(layout.paste_button()), 3, false, false), Some(Target::Paste));
        assert_eq!(layout.hit(center(layout.copy_button()), 3, false, false), Some(Target::Copy));
        assert_eq!(layout.hit(center(layout.delete_button()), 3, false, false), Some(Target::Delete));
        assert_eq!(layout.hit(center(layout.paste_button()), 0, false, false), None);
    }

    #[test]
    fn the_bar_covers_the_last_row_and_only_its_right_end_undoes() {
        let layout = layout();
        let on_last_row = Point::new(layout.row(6).left + 20.0, layout.row(6).center_y());
        assert_eq!(layout.hit(on_last_row, 7, true, true), None);
        assert_eq!(layout.hit(on_last_row, 7, false, false), Some(Target::Row(6)));
        let on_undo = Point::new(layout.bar().right - 10.0, layout.bar().center_y());
        assert_eq!(layout.hit(on_undo, 7, true, true), Some(Target::Undo));
        assert_eq!(layout.hit(on_undo, 7, true, false), None);
    }

    #[test]
    fn the_bar_follows_the_page_bottom_when_the_page_is_short() {
        let short = Layout::new(Rect::new(0.0, 0.0, CONTENT_WIDTH, 86.0));
        assert_eq!(short.bar().bottom, 86.0);
        assert_eq!(layout().bar().bottom, 50.0 + CONTENT_HEIGHT);
    }

    #[test]
    fn two_presses_close_together_in_time_and_place_make_a_double_click() {
        let mut clicks = DoubleClick::default();
        let start = Instant::now();
        let limit = Duration::from_millis(500);
        assert!(!clicks.press(start, Point::new(10.0, 10.0), limit, 4.0));
        assert!(clicks.press(start + Duration::from_millis(300), Point::new(12.0, 9.0), limit, 4.0));
        // The pair is used up, so a third quick press starts a new one.
        assert!(!clicks.press(start + Duration::from_millis(400), Point::new(12.0, 9.0), limit, 4.0));
    }

    #[test]
    fn a_slow_or_distant_second_press_is_not_a_double_click() {
        let mut clicks = DoubleClick::default();
        let start = Instant::now();
        let limit = Duration::from_millis(500);
        clicks.press(start, Point::new(10.0, 10.0), limit, 4.0);
        assert!(!clicks.press(start + Duration::from_millis(900), Point::new(10.0, 10.0), limit, 4.0));
        clicks.reset();
        clicks.press(start, Point::new(10.0, 10.0), limit, 4.0);
        assert!(!clicks.press(start + Duration::from_millis(100), Point::new(30.0, 10.0), limit, 4.0));
    }
}
