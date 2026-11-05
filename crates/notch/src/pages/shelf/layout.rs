//! Where the shelf's places, previews and remove buttons sit, and what a press hits.

use super::{EXPANDED_WIDTH, FILE_GAP, FILE_WIDTH, MAX_SLOTS, NAME, NAME_LINE_HEIGHT, NAME_TOP, ZONE_PADDING};
use crate::gfx::{Fonts, Point, Rect, clamp_lines};
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub(super) const NAME_WIDTH: f32 = FILE_WIDTH - 2.0;
pub(super) const COLUMN_HEIGHT: f32 = PREVIEW_SIZE + NAME_TOP + NAME_LINE_HEIGHT * NAME_LINES as f32;
pub(super) const REMOVE_HIT_RADIUS: f32 = 9.0;
pub(super) const PREVIEW_SIZE: f32 = 40.0;
pub(super) const NAME_LINES: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Slot {
    File(usize),
    More(usize),
}

pub(super) fn slots(count: usize) -> Vec<Slot> {
    if count <= MAX_SLOTS {
        (0..count).map(Slot::File).collect()
    } else {
        (0..MAX_SLOTS - 1).map(Slot::File).chain(std::iter::once(Slot::More(count - (MAX_SLOTS - 1)))).collect()
    }
}

pub(super) fn file_index(position: usize, count: usize) -> Option<usize> {
    match slots(count).get(position) {
        Some(Slot::File(index)) => Some(*index),
        _ => None,
    }
}

/// Keep slots centered while the drop target expands the page.
pub(super) fn slot_rect(position: usize, bounds: Rect, scale: f32) -> Rect {
    let spare_width = (bounds.width() - EXPANDED_WIDTH * scale).max(0.0) / 2.0;
    let spare_height = (bounds.height() - (COLUMN_HEIGHT + 2.0 * ZONE_PADDING) * scale).max(0.0) / 2.0;
    let left = bounds.left + spare_width + (ZONE_PADDING + position as f32 * (FILE_WIDTH + FILE_GAP)) * scale;
    let top = bounds.top + spare_height + ZONE_PADDING * scale;
    Rect::new(left, top, left + FILE_WIDTH * scale, top + COLUMN_HEIGHT * scale)
}

pub(super) fn preview_rect(column: Rect, scale: f32) -> Rect {
    let left = column.left + (FILE_WIDTH - PREVIEW_SIZE) / 2.0 * scale;
    Rect::new(left, column.top, left + PREVIEW_SIZE * scale, column.top + PREVIEW_SIZE * scale)
}

pub(super) fn remove_center(preview: Rect, scale: f32) -> Point {
    Point::new(preview.right - scale, preview.top + scale)
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
    let places = slots(count);
    for (position, slot) in places.iter().enumerate() {
        if !matches!(slot, Slot::File(_)) {
            continue;
        }
        let center = remove_center(preview_rect(slot_rect(position, bounds, scale), scale), scale);
        if (point.x - center.x).hypot(point.y - center.y) <= REMOVE_HIT_RADIUS * scale {
            return Some(Target::Remove(position));
        }
    }
    (0..places.len()).find(|&position| slot_rect(position, bounds, scale).contains(point)).map(Target::Slot)
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
    fn six_files_fit_and_a_seventh_uses_the_last_place_for_overflow() {
        assert_eq!(slots(6), (0..6).map(Slot::File).collect::<Vec<_>>());
        assert_eq!(
            slots(7),
            vec![Slot::File(0), Slot::File(1), Slot::File(2), Slot::File(3), Slot::File(4), Slot::More(2)]
        );
        assert_eq!(EXPANDED_SIZE.width, 507.0);
    }

    fn middle(rect: Rect) -> Point {
        Point::new(rect.center_x(), rect.center_y())
    }

    #[test]
    fn a_few_files_each_get_a_place() {
        assert!(slots(0).is_empty());
        assert_eq!(slots(3), vec![Slot::File(0), Slot::File(1), Slot::File(2)]);
        assert_eq!(slots(MAX_SLOTS).len(), MAX_SLOTS);
    }

    #[test]
    fn too_many_files_end_in_a_place_that_counts_the_rest() {
        let places = slots(12);
        assert_eq!(places.len(), MAX_SLOTS);
        assert_eq!(places[MAX_SLOTS - 1], Slot::More(12 - (MAX_SLOTS - 1)));
        assert_eq!(places[0], Slot::File(0));
    }

    #[test]
    fn only_a_place_that_shows_a_file_has_a_file() {
        assert_eq!(file_index(1, 3), Some(1));
        assert_eq!(file_index(MAX_SLOTS - 1, 12), None);
        assert_eq!(file_index(4, 3), None);
    }

    #[test]
    fn the_places_sit_side_by_side_and_fill_the_page_from_edge_to_edge_of_its_padding() {
        let first = slot_rect(0, BOUNDS, 1.0);
        let second = slot_rect(1, BOUNDS, 1.0);
        assert_eq!(second.left - first.right, FILE_GAP);
        assert_eq!(first.left, ZONE_PADDING);
        let last = slot_rect(MAX_SLOTS - 1, BOUNDS, 1.0);
        assert!((BOUNDS.right - last.right - ZONE_PADDING).abs() < 0.01, "{last:?}");
    }

    #[test]
    fn the_padding_is_the_same_on_every_side_of_a_column() {
        let column = slot_rect(0, BOUNDS, 1.0);
        assert_eq!(column.left - BOUNDS.left, column.top - BOUNDS.top);
        assert!((BOUNDS.bottom - column.bottom - ZONE_PADDING).abs() < 0.01, "{column:?}");
    }

    #[test]
    fn a_press_finds_the_file_or_the_cross_under_it() {
        let column = slot_rect(1, BOUNDS, 1.0);
        let inside = Point::new(column.center_x(), column.bottom - 2.0);
        assert_eq!(target_at(inside, BOUNDS, 1.0, 3), Some(Target::Slot(1)));
        let cross = remove_center(preview_rect(column, 1.0), 1.0);
        assert_eq!(target_at(cross, BOUNDS, 1.0, 3), Some(Target::Remove(1)));
        assert_eq!(target_at(Point::new(BOUNDS.right - 5.0, 80.0), BOUNDS, 1.0, 3), None);
    }

    #[test]
    fn the_more_place_has_no_cross() {
        let more = slot_rect(MAX_SLOTS - 1, BOUNDS, 1.0);
        let corner = remove_center(preview_rect(more, 1.0), 1.0);
        assert_eq!(target_at(corner, BOUNDS, 1.0, 12), Some(Target::Slot(MAX_SLOTS - 1)));
    }

    #[test]
    fn an_empty_shelf_has_nothing_to_press() {
        assert_eq!(target_at(Point::new(40.0, 40.0), BOUNDS, 1.0, 0), None);
        assert_eq!(target_at(middle(slot_rect(0, BOUNDS, 1.0)), BOUNDS, 1.0, 0), None);
    }

    #[test]
    fn the_page_is_exactly_as_wide_as_its_places_and_padding() {
        let places = MAX_SLOTS as f32 * FILE_WIDTH + (MAX_SLOTS - 1) as f32 * FILE_GAP;
        assert_eq!(EXPANDED_SIZE.width, places + 2.0 * ZONE_PADDING);
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
