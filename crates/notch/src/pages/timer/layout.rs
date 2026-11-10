//! Where everything in the open timer page goes. Sizes are in device-independent pixels from the page's top-left.

use crate::gfx::{Point, Rect, Size};

pub const EXPANDED_SIZE: Size = Size::new(290.0, 86.0);
const _: () = assert!(EXPANDED_SIZE.height <= 190.0);
const TIME_TOP: f32 = 2.0;
const TIME_HEIGHT: f32 = 42.0;
/// Width of the widest time ("00:00" at 38 px with tabular digits), so the words never move while it counts.
pub const TIME_WIDTH: f32 = 105.0;
const WORDS_GAP: f32 = 12.0;
/// Next to the time the words sit on the row's bottom edge (flex-end): a 13 px label line over an 11 px detail line.
const LABEL_HEIGHT: f32 = 16.0;
const LABEL_DETAIL_GAP: f32 = 1.0;
const LABEL_DETAIL_HEIGHT: f32 = 14.0;
/// Without a time the 17 px headline and its detail start at the top of the row.
const HEADLINE_HEIGHT: f32 = 21.0;
const HEADLINE_DETAIL_GAP: f32 = 2.0;
const HEADLINE_DETAIL_HEIGHT: f32 = 15.0;
/// Clear space between the time row and the buttons.
const WORDS_BUTTONS_GAP: f32 = 12.0;
const ROW_BOTTOM: f32 = TIME_TOP + TIME_HEIGHT;
const BUTTON_TOP: f32 = ROW_BOTTOM + WORDS_BUTTONS_GAP;
const BUTTON_HEIGHT: f32 = 30.0;
const BUTTON_GAP: f32 = 9.0;
/// The time fills the room above the buttons; a headline without a time is centered there.
const TEXT_ZONE_HEIGHT: f32 = TIME_HEIGHT;

pub struct Layout {
    pub bounds: Rect,
}

impl Layout {
    fn rect(&self, left: f32, top: f32, right: f32, bottom: f32) -> Rect {
        let (origin_x, origin_y) = (self.bounds.left, self.bounds.top);
        Rect::new(origin_x + left, origin_y + top, origin_x + right, origin_y + bottom)
    }

    fn width(&self) -> f32 {
        self.bounds.width()
    }

    fn words_left(&self, has_time: bool) -> f32 {
        if has_time { TIME_WIDTH + WORDS_GAP } else { 0.0 }
    }

    /// How far a block of `content_height` moves down to sit in the middle of the text zone.
    fn centered_shift(content_height: f32) -> f32 {
        ((TEXT_ZONE_HEIGHT - content_height) / 2.0).floor()
    }

    /// Shift of everything above the buttons: none with a time and a detail line (the idle page), centered otherwise.
    fn shift(has_time: bool, has_detail: bool) -> f32 {
        match (has_time, has_detail) {
            (true, true) => 0.0,
            (true, false) => Self::centered_shift(TIME_HEIGHT),
            (false, _) => Self::centered_shift(HEADLINE_HEIGHT + HEADLINE_DETAIL_GAP + HEADLINE_DETAIL_HEIGHT),
        }
    }

    pub fn title(&self, has_time: bool, has_detail: bool) -> Rect {
        let top = if has_time {
            let block = if has_detail { LABEL_HEIGHT + LABEL_DETAIL_GAP + LABEL_DETAIL_HEIGHT } else { LABEL_HEIGHT };
            ROW_BOTTOM - block
        } else {
            TIME_TOP + Self::shift(false, has_detail)
        };
        let height = if has_time { LABEL_HEIGHT } else { HEADLINE_HEIGHT };
        self.rect(self.words_left(has_time), top, self.width(), top + height)
    }

    pub fn detail(&self, has_time: bool, has_detail: bool) -> Rect {
        let (top, height) = if has_time {
            (ROW_BOTTOM - LABEL_DETAIL_HEIGHT, LABEL_DETAIL_HEIGHT)
        } else {
            (TIME_TOP + HEADLINE_HEIGHT + HEADLINE_DETAIL_GAP + Self::shift(false, has_detail), HEADLINE_DETAIL_HEIGHT)
        };
        self.rect(self.words_left(has_time), top, self.width(), top + height)
    }

    pub fn time(&self, has_detail: bool) -> Rect {
        let top = TIME_TOP + Self::shift(true, has_detail);
        self.rect(0.0, top, TIME_WIDTH, top + TIME_HEIGHT)
    }

    pub fn buttons(&self, count: usize) -> Vec<Rect> {
        if count == 0 {
            return Vec::new();
        }
        let available = self.width() - BUTTON_GAP * (count - 1) as f32;
        let button_width = available / count as f32;
        (0..count)
            .map(|index| {
                let left = index as f32 * (button_width + BUTTON_GAP);
                self.rect(left, BUTTON_TOP, left + button_width, BUTTON_TOP + BUTTON_HEIGHT)
            })
            .collect()
    }

    pub fn button_at(&self, count: usize, point: Point) -> Option<usize> {
        self.buttons(count).iter().position(|rect| rect.contains(point))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout() -> Layout {
        Layout { bounds: Rect::new(100.0, 0.0, 100.0 + EXPANDED_SIZE.width, EXPANDED_SIZE.height) }
    }

    #[test]
    fn the_buttons_share_the_width_evenly_and_fit_the_page() {
        let layout = layout();
        let rects = layout.buttons(3);
        assert_eq!(rects.len(), 3);
        assert!((rects[0].width() - rects[2].width()).abs() < 0.01);
        assert!(rects[0].left >= layout.bounds.left && rects[2].right <= layout.bounds.right + 0.01);
        assert!(rects[1].left > rects[0].right);
        assert!(rects.iter().all(|rect| rect.bottom <= layout.bounds.bottom));
    }

    #[test]
    fn two_buttons_are_wider_than_three() {
        let layout = layout();
        assert!(layout.buttons(2)[0].width() > layout.buttons(3)[0].width());
        assert!(layout.buttons(0).is_empty());
    }

    #[test]
    fn a_press_finds_the_button_under_it() {
        let layout = layout();
        let rects = layout.buttons(3);
        let inside = |rect: &Rect| Point::new(rect.center_x(), rect.center_y());
        assert_eq!(layout.button_at(3, inside(&rects[1])), Some(1));
        assert_eq!(layout.button_at(3, Point::new(0.0, 0.0)), None);
    }

    #[test]
    fn the_words_sit_right_of_the_time_only_when_there_is_one() {
        let layout = layout();
        assert!(layout.title(true, true).left >= layout.time(true).right);
        assert!(layout.detail(true, true).left >= layout.time(true).right);
        assert_eq!(layout.title(false, true).left, layout.bounds.left);
    }

    #[test]
    fn the_words_stack_above_the_buttons_in_every_state() {
        let layout = layout();
        for (has_time, has_detail) in [(true, true), (true, false), (false, true)] {
            if has_detail {
                assert!(layout.title(has_time, has_detail).bottom <= layout.detail(has_time, has_detail).top);
            }
            assert!(layout.title(has_time, has_detail).top >= layout.bounds.top);
            assert!(layout.time(has_detail).bottom <= layout.buttons(3)[0].top);
        }
        assert!(layout.detail(true, true).bottom <= layout.buttons(3)[0].top);
        assert!(layout.detail(false, true).bottom <= layout.buttons(3)[0].top);
    }

    #[test]
    fn the_detail_line_keeps_at_least_ten_px_clear_of_the_buttons() {
        let layout = layout();
        let buttons_top = layout.buttons(3)[0].top;
        assert!(buttons_top - layout.detail(true, true).bottom >= 10.0);
        assert!(buttons_top - layout.detail(false, true).bottom >= 10.0);
        assert!(buttons_top - layout.time(false).bottom >= 10.0);
    }

    #[test]
    fn the_time_fills_its_row_and_a_headline_is_centered_in_it() {
        let layout = layout();
        assert_eq!(layout.time(true).top, layout.time(false).top);
        assert_eq!(layout.time(true).height(), TIME_HEIGHT);
        assert!(layout.title(false, true).top > TIME_TOP);
        assert!(layout.detail(false, true).bottom <= layout.time(true).bottom);
    }

    #[test]
    fn the_words_next_to_the_time_end_on_its_bottom_edge_with_a_12_px_gap() {
        let layout = layout();
        assert!(layout.title(true, true).top > layout.title(false, true).top);
        assert_eq!(layout.title(true, true).left - layout.time(true).right, WORDS_GAP);
        assert_eq!(layout.detail(true, true).bottom, layout.time(true).bottom);
        assert_eq!(layout.title(true, false).bottom, layout.time(true).bottom, "a lone label sits on the same edge");
        assert_eq!(layout.buttons(3)[0].top - layout.time(true).bottom, 12.0);
        assert_eq!(EXPANDED_SIZE.height, 86.0);
    }

    #[test]
    fn the_buttons_end_at_the_bottom_of_the_page() {
        let layout = layout();
        assert_eq!(layout.buttons(3)[0].bottom, layout.bounds.bottom);
    }

    #[test]
    fn the_button_labels_fit_their_buttons_in_both_languages() {
        let fonts = crate::gfx::Fonts::new(1.0).unwrap();
        let layout = layout();
        let (three, two) = (layout.buttons(3)[0].width(), layout.buttons(2)[0].width());
        for language in [store::language::Language::English, store::language::Language::Indonesian] {
            store::language::with_override(language, || {
                for (key, room) in [
                    ("timer.start", three),
                    ("timer.pause", three),
                    ("timer.extend", three),
                    ("timer.stop", three),
                    ("timer.resume", three),
                    ("timer.start_short_break", two),
                    ("timer.start_long_break", two),
                    ("timer.start_focus", two),
                    ("timer.skip", two),
                ] {
                    let width = fonts.measure(&crate::ui::type_styles::MEDIUM_12, &crate::i18n::tr!(key));
                    assert!(width <= room - 16.0, "{language:?}: {key} is {width} px in a {room} px button");
                }
            });
        }
    }
}
