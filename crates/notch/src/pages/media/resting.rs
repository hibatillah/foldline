use super::{MediaPage, styles};
use crate::{
    gfx::{Canvas, Color, Rect},
    page::{Frame, RESTING_HEIGHT},
};

/// Width in DIP of the artwork at the left end of the claimed pill.
pub const ARTWORK_SIZE: f32 = 20.0;
const ARTWORK_RADIUS: f32 = 5.0;
const LYRIC_SLACK: f32 = 8.0;
/// Room in DIP between the artwork and the lyric line, asked of the host through `Page::resting_lead_gap`.
pub const LYRIC_LEAD_GAP: f32 = 12.0;
/// Smallest line area while lyrics are on, so a short line or a rest in the music does not shrink the pill.
const MIN_LYRIC_WIDTH: f32 = 128.0;
const NOTE: &str = "\u{266A}";
const MAX_LYRIC_WIDTH: f32 = 500.0;
const LYRIC_TRAVEL: f32 = 6.0;

/// Width of the line area of the claimed pill; the host adds the artwork, its gaps and edges.
pub fn width(lyric_width: f32) -> f32 {
    (lyric_width + LYRIC_SLACK).clamp(MIN_LYRIC_WIDTH, MAX_LYRIC_WIDTH)
}

/// Where the lyric line starts: after the artwork and the gap the host reserved.
fn lyric_left(frame: &Frame) -> f32 {
    frame.bounds.left + ARTWORK_SIZE + frame.lead_gap
}

fn draw_art(page: &MediaPage, canvas: &Canvas, area: Rect, opacity: f32) {
    match page.artwork.brush().or(page.app_icon.brush()) {
        Some(image) => canvas.image(image, area, ARTWORK_RADIUS, opacity),
        None => canvas.text(&styles::RESTING_LABEL, NOTE, area, Color::WHITE.faded(opacity)),
    }
}

pub fn draw(page: &MediaPage, canvas: &Canvas, frame: &Frame, opacity: f32) {
    let bounds = frame.bounds;
    let row_top = bounds.top;
    let row_bottom = bounds.top + RESTING_HEIGHT;
    let center_y = (row_top + row_bottom) / 2.0;
    let half_art = ARTWORK_SIZE / 2.0;
    let art_area = Rect::new(bounds.left, center_y - half_art, bounds.left + ARTWORK_SIZE, center_y + half_art);
    draw_art(page, canvas, art_area, opacity);

    let (left, right) = (lyric_left(frame), bounds.right);
    let transition = page.follower.transition().clamp(0.0, 1.0);
    let travel = LYRIC_TRAVEL;
    let band = |offset: f32| Rect::new(left, row_top + offset, right, row_bottom + offset);
    let previous = Color::WHITE.faded(opacity * (1.0 - transition));
    let current = Color::WHITE.faded(opacity * transition);
    let shown = |text: &str| if text.is_empty() { NOTE.to_owned() } else { text.to_owned() };
    canvas.text(&styles::RESTING_LYRIC, &shown(page.follower.previous()), band(-transition * travel), previous);
    canvas.text(&styles::RESTING_LYRIC, &shown(page.follower.current()), band((1.0 - transition) * travel), current);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::Size;

    #[test]
    fn the_lyrics_pill_grows_with_the_line_up_to_a_maximum() {
        assert!(width(300.0) > width(150.0));
        assert_eq!(width(5000.0), MAX_LYRIC_WIDTH);
        assert_eq!(width(1.0), MIN_LYRIC_WIDTH);
    }

    #[test]
    fn a_short_lyric_line_keeps_a_line_area_of_128_units_and_adds_no_artwork_width() {
        assert_eq!(width(1.0), 128.0);
        assert_eq!(width(0.0), 128.0);
        assert_eq!(width(200.0), 200.0 + LYRIC_SLACK);
    }

    #[test]
    fn the_line_starts_after_the_artwork_and_the_gap_the_host_reserved() {
        let frame = |lead_gap| Frame {
            bounds: Rect::new(100.0, 0.0, 350.0, 37.0),
            expansion: 0.0,
            expanded: Size::new(0.0, 0.0),
            lead_gap,
        };
        assert_eq!(lyric_left(&frame(0.0)), 100.0 + ARTWORK_SIZE);
        assert_eq!(lyric_left(&frame(26.0)), 100.0 + ARTWORK_SIZE + 26.0);
    }
}
