//! Where each part of a tile and each message sits; all numbers are units (device-independent pixels from the design).

use crate::gfx::{Point, Rect};

/// Width of the page while it shows a message: the row of badge and text with room to spare, 350 px with the host padding.
pub(super) const MESSAGE_WIDTH: f32 = 318.0;
pub(super) const CONTENT_HEIGHT: f32 = 86.0;
pub(super) const CELL_WIDTH: f32 = 88.0;
/// The page is never narrower than this (285 px with the host's padding), so one or two tiles do not make it tiny.
const MIN_TILES_WIDTH: f32 = 253.0;
pub(super) const RING_DIAMETER: f32 = 58.0;

const RING_TOP: f32 = 6.0;
const ICON_CENTER_FROM_RING_TOP: f32 = 22.0;
const PERCENT_CENTER_FROM_RING_TOP: f32 = 41.0;
const LINE_HEIGHT: f32 = 12.0;
const NAME_TOP: f32 = 68.0;
const NAME_MAX_WIDTH: f32 = 76.0;
pub(super) const MESSAGE_ICON_SIZE: f32 = 22.0;
/// The Bluetooth mark sits in a round badge of this diameter.
pub(super) const MESSAGE_BADGE_DIAMETER: f32 = 36.0;
const MESSAGE_BADGE_GAP: f32 = 10.0;
const MESSAGE_TITLE_HEIGHT: f32 = 18.0;
const MESSAGE_LINES_GAP: f32 = 3.0;
const MESSAGE_DETAIL_HEIGHT: f32 = 16.0;
const MESSAGE_TEXT_HEIGHT: f32 = MESSAGE_TITLE_HEIGHT + MESSAGE_LINES_GAP + MESSAGE_DETAIL_HEIGHT;

/// Content width of the page for `device_count` tiles: 88 px each, at least `MIN_TILES_WIDTH`.
pub(super) fn tiles_width(device_count: usize) -> f32 {
    (device_count as f32 * CELL_WIDTH).max(MIN_TILES_WIDTH)
}

/// The full-height 88 px cell of `index`; the row of `count` cells is centered in `bounds`.
pub(super) fn cell(bounds: Rect, index: usize, count: usize) -> Rect {
    let left = bounds.center_x() - count as f32 * CELL_WIDTH / 2.0 + index as f32 * CELL_WIDTH;
    Rect::new(left, bounds.top, left + CELL_WIDTH, bounds.bottom)
}

pub(super) fn ring_center(cell: Rect) -> Point {
    Point::new(cell.center_x(), cell.top + RING_TOP + RING_DIAMETER / 2.0)
}

pub(super) fn icon_center(cell: Rect) -> Point {
    Point::new(cell.center_x(), cell.top + RING_TOP + ICON_CENTER_FROM_RING_TOP)
}

/// A line as high as the percentage text, centered under the icon.
pub(super) fn percent_line(cell: Rect) -> Rect {
    let middle = cell.top + RING_TOP + PERCENT_CENTER_FROM_RING_TOP;
    Rect::new(cell.left, middle - LINE_HEIGHT / 2.0, cell.right, middle + LINE_HEIGHT / 2.0)
}

/// The name, at most 76 px wide and centered in the cell.
pub(super) fn name_line(cell: Rect) -> Rect {
    let half_width = (cell.width() / 2.0).min(NAME_MAX_WIDTH / 2.0);
    Rect::new(
        cell.center_x() - half_width,
        cell.top + NAME_TOP,
        cell.center_x() + half_width,
        cell.top + NAME_TOP + LINE_HEIGHT,
    )
}

/// The badge center and the title and detail lines of a whole-page message: one row, centered as a whole, with the
/// badge at the left and the left-aligned text block (`text_width` wide, in device-independent pixels) at the right.
pub(super) fn message_lines(bounds: Rect, text_width: f32) -> (Point, Rect, Rect) {
    let row_width = MESSAGE_BADGE_DIAMETER + MESSAGE_BADGE_GAP + text_width;
    let left = bounds.center_x() - row_width / 2.0;
    let middle = bounds.top + CONTENT_HEIGHT / 2.0;
    let text_left = left + MESSAGE_BADGE_DIAMETER + MESSAGE_BADGE_GAP;
    let text_top = middle - MESSAGE_TEXT_HEIGHT / 2.0;
    let line = |from: f32, height: f32| {
        Rect::new(text_left, text_top + from, text_left + text_width + 1.0, text_top + from + height)
    };
    let badge = Point::new(left + MESSAGE_BADGE_DIAMETER / 2.0, middle);
    (badge, line(0.0, MESSAGE_TITLE_HEIGHT), line(MESSAGE_TITLE_HEIGHT + MESSAGE_LINES_GAP, MESSAGE_DETAIL_HEIGHT))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds() -> Rect {
        Rect::new(100.0, 40.0, 100.0 + 5.0 * CELL_WIDTH, 40.0 + CONTENT_HEIGHT)
    }

    #[test]
    fn the_page_is_as_wide_as_its_tiles_but_never_narrower_than_the_minimum() {
        assert_eq!(tiles_width(5), 440.0);
        assert_eq!(tiles_width(4), 352.0);
        assert_eq!(tiles_width(1), MIN_TILES_WIDTH);
        assert_eq!(tiles_width(2), MIN_TILES_WIDTH);
        assert_eq!(tiles_width(0), MIN_TILES_WIDTH);
    }

    #[test]
    fn cells_are_88_px_wide_and_the_row_is_centered() {
        let cells: Vec<Rect> = (0..5).map(|index| cell(bounds(), index, 5)).collect();
        assert_eq!(cells[0].left, bounds().left);
        assert!((cells[4].right - bounds().right).abs() < 0.01);
        assert!(cells.windows(2).all(|pair| (pair[0].right - pair[1].left).abs() < 0.01));
        assert!(cells.iter().all(|cell| cell.width() == CELL_WIDTH));
        let wide = Rect::new(0.0, 0.0, MIN_TILES_WIDTH, CONTENT_HEIGHT);
        let (first, second) = (cell(wide, 0, 2), cell(wide, 1, 2));
        assert!((first.left - (wide.width() - 2.0 * CELL_WIDTH) / 2.0).abs() < 0.01);
        assert_eq!(second.left, first.right);
    }

    #[test]
    fn the_ring_icon_percentage_and_name_stack_inside_the_content_area() {
        let cell = cell(bounds(), 0, 5);
        let ring = ring_center(cell);
        assert_eq!(ring.y - cell.top, 35.0);
        assert!(ring.y - RING_DIAMETER / 2.0 >= cell.top && ring.y + RING_DIAMETER / 2.0 <= cell.bottom);
        assert!(icon_center(cell).y < percent_line(cell).top + 6.0);
        assert!(percent_line(cell).bottom <= ring.y + RING_DIAMETER / 2.0);
        assert_eq!(name_line(cell).bottom - cell.top, 80.0);
        assert!(name_line(cell).bottom <= cell.bottom);
    }

    #[test]
    fn a_name_is_at_most_76_px_wide_inside_its_88_px_cell() {
        let cell = cell(bounds(), 0, 5);
        assert_eq!(name_line(cell).width(), NAME_MAX_WIDTH);
        assert!(name_line(cell).width() <= cell.width());
    }

    #[test]
    fn a_message_is_one_centered_row_with_the_badge_left_of_left_aligned_text() {
        let area = Rect::new(100.0, 40.0, 100.0 + MESSAGE_WIDTH, 40.0 + CONTENT_HEIGHT);
        let text_width = 220.0;
        let (badge, title, detail) = message_lines(area, text_width);
        let row_width = MESSAGE_BADGE_DIAMETER + MESSAGE_BADGE_GAP + text_width;
        assert!((badge.x - MESSAGE_BADGE_DIAMETER / 2.0 - (area.center_x() - row_width / 2.0)).abs() < 0.01);
        assert_eq!(badge.y, area.top + CONTENT_HEIGHT / 2.0);
        assert!((title.left - (badge.x + MESSAGE_BADGE_DIAMETER / 2.0 + MESSAGE_BADGE_GAP)).abs() < 0.01);
        assert_eq!(detail.left, title.left);
        assert!((detail.top - title.bottom - MESSAGE_LINES_GAP).abs() < 0.01);
        assert!(title.bottom <= detail.top && detail.bottom <= area.bottom && title.top >= area.top);
    }

    #[test]
    fn the_message_row_fits_the_message_width() {
        const {
            assert!(
                MESSAGE_BADGE_DIAMETER + MESSAGE_BADGE_GAP + 260.0 <= MESSAGE_WIDTH,
                "the longest detail is about 260 px"
            );
        }
    }
}
