//! Where the shelf's places, previews and remove buttons sit, and what a press hits.

use super::{
    EXPANDED_WIDTH, FILE_GAP, FILE_WIDTH, MAX_FILES, NAME, NAME_LINE_HEIGHT, NAME_TOP, ROW_GAP, SLOTS_PER_ROW,
    ZONE_PADDING,
};
use crate::gfx::{Fonts, Point, Rect, clamp_lines};
use crate::page::DEFAULT_CONTENT_HEIGHT;
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub(super) const NAME_WIDTH: f32 = FILE_WIDTH - 2.0;
pub(super) const COLUMN_HEIGHT: f32 = PREVIEW_SIZE + NAME_TOP + NAME_LINE_HEIGHT * NAME_LINES as f32;
pub(super) const REMOVE_HIT_RADIUS: f32 = 9.0;
/// The cross button hangs 3 px over the preview's corner, so its center is its 8 px radius less inside it.
const REMOVE_INSET: f32 = 5.0;
pub(super) const PREVIEW_SIZE: f32 = 40.0;
pub(super) const NAME_LINES: usize = 2;

/// How many files get a place; the shelf never holds more than `MAX_FILES`.
pub(super) fn shown_count(count: usize) -> usize {
    count.min(MAX_FILES)
}

/// Rows of six the files need, at least one so an empty shelf keeps its height.
pub(super) fn rows(count: usize) -> usize {
    shown_count(count).div_ceil(SLOTS_PER_ROW).max(1)
}

pub(super) fn file_index(position: usize, count: usize) -> Option<usize> {
    (position < shown_count(count)).then_some(position)
}

/// The open height: the usual page height for one row, taller for each further row.
pub(super) fn content_height(count: usize) -> f32 {
    let rows = rows(count) as f32;
    (2.0 * ZONE_PADDING + rows * COLUMN_HEIGHT + (rows - 1.0) * ROW_GAP).max(DEFAULT_CONTENT_HEIGHT)
}

/// Keep the places centered while the drop target expands the page.
pub(super) fn slot_rect(position: usize, count: usize, bounds: Rect, scale: f32) -> Rect {
    let needed = 2.0 * ZONE_PADDING + rows(count) as f32 * COLUMN_HEIGHT + (rows(count) - 1) as f32 * ROW_GAP;
    let spare_width = (bounds.width() - EXPANDED_WIDTH * scale).max(0.0) / 2.0;
    let spare_height = (bounds.height() - needed * scale).max(0.0) / 2.0;
    let (row, column) = (position / SLOTS_PER_ROW, position % SLOTS_PER_ROW);
    let left = bounds.left + spare_width + (ZONE_PADDING + column as f32 * (FILE_WIDTH + FILE_GAP)) * scale;
    let top = bounds.top + spare_height + (ZONE_PADDING + row as f32 * (COLUMN_HEIGHT + ROW_GAP)) * scale;
    Rect::new(left, top, left + FILE_WIDTH * scale, top + COLUMN_HEIGHT * scale)
}

pub(super) fn preview_rect(column: Rect, scale: f32) -> Rect {
    let left = column.left + (FILE_WIDTH - PREVIEW_SIZE) / 2.0 * scale;
    Rect::new(left, column.top, left + PREVIEW_SIZE * scale, column.top + PREVIEW_SIZE * scale)
}

pub(super) fn remove_center(preview: Rect, scale: f32) -> Point {
    Point::new(preview.right - REMOVE_INSET * scale, preview.top + REMOVE_INSET * scale)
}

pub(super) fn fitted(size: (u32, u32), area: Rect) -> Rect {
    let (width, height) = (size.0.max(1) as f32, size.1.max(1) as f32);
    let factor = (area.width() / width).min(area.height() / height);
    Rect::around(Point::new(area.center_x(), area.center_y()), width * factor / 2.0, height * factor / 2.0)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Target {
    Slot(usize),
    Remove(usize),
}

pub(super) fn target_at(point: Point, bounds: Rect, scale: f32, count: usize) -> Option<Target> {
    let places = shown_count(count);
    for position in 0..places {
        let center = remove_center(preview_rect(slot_rect(position, count, bounds, scale), scale), scale);
        if (point.x - center.x).hypot(point.y - center.y) <= REMOVE_HIT_RADIUS * scale {
            return Some(Target::Remove(position));
        }
    }
    (0..places).find(|&position| slot_rect(position, count, bounds, scale).contains(point)).map(Target::Slot)
}

pub(super) fn name_lines(name: &str, fonts: &Fonts) -> Vec<String> {
    clamp_lines(name, NAME_LINES, NAME_WIDTH, |text| fonts.measure(&NAME, text))
}

pub(super) fn is_double_click(
    previous: Option<&(PathBuf, Instant)>,
    path: &Path,
    now: Instant,
    limit: Duration,
) -> bool {
    previous.is_some_and(|(last_path, at)| last_path == path && now.duration_since(*at) <= limit)
}

#[cfg(test)]
mod tests {
    use super::{super::*, *};

    const BOUNDS: Rect = Rect::new(0.0, 0.0, EXPANDED_WIDTH, 86.0);

    #[test]
    fn six_files_fit_a_row_and_a_seventh_starts_a_second_one() {
        assert_eq!(rows(0), 1);
        assert_eq!(rows(6), 1);
        assert_eq!(rows(7), 2);
        assert_eq!(rows(MAX_FILES), 2);
        assert_eq!(EXPANDED_WIDTH, 507.0);
    }

    fn middle(rect: Rect) -> Point {
        Point::new(rect.center_x(), rect.center_y())
    }

    #[test]
    fn the_shelf_holds_twelve_files_and_no_place_exists_past_them() {
        assert_eq!(shown_count(3), 3);
        assert_eq!(shown_count(40), MAX_FILES);
        assert_eq!(file_index(11, 12), Some(11));
        assert_eq!(file_index(12, 40), None);
        assert_eq!(file_index(4, 3), None);
    }

    #[test]
    fn the_page_is_the_usual_height_for_one_row_and_taller_for_two() {
        assert_eq!(content_height(0), DEFAULT_CONTENT_HEIGHT);
        assert_eq!(content_height(6), DEFAULT_CONTENT_HEIGHT);
        let two = 2.0 * ZONE_PADDING + 2.0 * COLUMN_HEIGHT + ROW_GAP;
        assert_eq!(content_height(7), two);
        assert_eq!(content_height(MAX_FILES), two);
        assert!(two <= crate::page::MAX_CONTENT_HEIGHT);
    }

    #[test]
    fn the_places_sit_side_by_side_and_fill_the_page_from_edge_to_edge_of_its_padding() {
        let first = slot_rect(0, 3, BOUNDS, 1.0);
        let second = slot_rect(1, 3, BOUNDS, 1.0);
        assert_eq!(second.left - first.right, FILE_GAP);
        assert_eq!(first.left, ZONE_PADDING);
        let last = slot_rect(SLOTS_PER_ROW - 1, 6, BOUNDS, 1.0);
        assert!((BOUNDS.right - last.right - ZONE_PADDING).abs() < 0.01, "{last:?}");
    }

    #[test]
    fn the_seventh_file_starts_the_second_row_under_the_first() {
        let tall = Rect::new(0.0, 0.0, EXPANDED_WIDTH, content_height(7));
        let first = slot_rect(0, 7, tall, 1.0);
        let seventh = slot_rect(SLOTS_PER_ROW, 7, tall, 1.0);
        assert_eq!(seventh.left, first.left);
        assert_eq!(seventh.top - first.bottom, ROW_GAP);
        assert!((tall.bottom - seventh.bottom - ZONE_PADDING).abs() < 0.01, "{seventh:?}");
    }

    #[test]
    fn the_padding_is_the_same_on_every_side_of_a_column() {
        let column = slot_rect(0, 3, BOUNDS, 1.0);
        assert_eq!(column.left - BOUNDS.left, column.top - BOUNDS.top);
        assert!((BOUNDS.bottom - column.bottom - ZONE_PADDING).abs() < 0.01, "{column:?}");
    }

    #[test]
    fn a_press_finds_the_file_or_the_cross_under_it() {
        let column = slot_rect(1, 3, BOUNDS, 1.0);
        let inside = Point::new(column.center_x(), column.bottom - 2.0);
        assert_eq!(target_at(inside, BOUNDS, 1.0, 3), Some(Target::Slot(1)));
        let cross = remove_center(preview_rect(column, 1.0), 1.0);
        assert_eq!(target_at(cross, BOUNDS, 1.0, 3), Some(Target::Remove(1)));
        assert_eq!(target_at(Point::new(BOUNDS.right - 5.0, 80.0), BOUNDS, 1.0, 3), None);
    }

    #[test]
    fn every_file_of_a_full_shelf_has_a_cross() {
        let tall = Rect::new(0.0, 0.0, EXPANDED_WIDTH, content_height(MAX_FILES));
        let last = slot_rect(MAX_FILES - 1, MAX_FILES, tall, 1.0);
        let corner = remove_center(preview_rect(last, 1.0), 1.0);
        assert_eq!(target_at(corner, tall, 1.0, MAX_FILES), Some(Target::Remove(MAX_FILES - 1)));
    }

    #[test]
    fn an_empty_shelf_has_nothing_to_press() {
        assert_eq!(target_at(Point::new(40.0, 40.0), BOUNDS, 1.0, 0), None);
        assert_eq!(target_at(middle(slot_rect(0, 0, BOUNDS, 1.0)), BOUNDS, 1.0, 0), None);
    }

    #[test]
    fn the_page_is_exactly_as_wide_as_its_places_and_padding() {
        let places = SLOTS_PER_ROW as f32 * FILE_WIDTH + (SLOTS_PER_ROW - 1) as f32 * FILE_GAP;
        assert_eq!(EXPANDED_WIDTH, places + 2.0 * ZONE_PADDING);
    }

    #[test]
    fn a_picture_keeps_its_proportions_inside_its_box() {
        let area = Rect::new(0.0, 0.0, 40.0, 40.0);
        let wide = fitted((96, 54), area);
        assert_eq!(wide.width(), 40.0);
        assert!((wide.height() - 22.5).abs() < 0.01);
        assert_eq!((wide.center_x(), wide.center_y()), (20.0, 20.0));
        let tall = fitted((50, 100), area);
        assert_eq!((tall.width(), tall.height()), (20.0, 40.0));
    }

    #[test]
    fn a_long_name_takes_two_lines_and_ends_in_an_ellipsis() {
        let fonts = Fonts::new(1.0).unwrap();
        assert_eq!(name_lines("notes.txt", &fonts), vec!["notes.txt"]);
        let lines = name_lines("a-very-long-file-name-for-the-quarterly-budget-review.xlsx", &fonts);
        assert_eq!(lines.len(), NAME_LINES);
        assert!(lines[NAME_LINES - 1].ends_with('\u{2026}'));
        assert!(lines.iter().all(|line| fonts.measure(&NAME, line) <= NAME_WIDTH + 0.5));
    }

    #[test]
    fn two_clicks_on_the_same_file_in_time_make_a_double_click() {
        let (path, now) = (PathBuf::from("a.txt"), Instant::now());
        let limit = Duration::from_millis(500);
        let first = Some((path.clone(), now));
        assert!(is_double_click(first.as_ref(), &path, now + Duration::from_millis(300), limit));
        assert!(!is_double_click(first.as_ref(), &path, now + Duration::from_millis(900), limit));
        assert!(!is_double_click(first.as_ref(), Path::new("b.txt"), now + Duration::from_millis(100), limit));
        assert!(!is_double_click(None, &path, now, limit));
    }
}
