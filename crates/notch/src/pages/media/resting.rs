use super::{MediaPage, styles};
use crate::{
    gfx::{Canvas, Color, Rect},
    page::{Frame, RESTING_HEIGHT},
};

/// Width in DIP of the artwork at the left end of the claimed pill.
pub const ARTWORK_SIZE: f32 = 20.0;
const ARTWORK_RADIUS: f32 = 5.0;
const LYRIC_SLACK: f32 = 8.0;
/// Smallest collapsed pill while lyrics are on, so a short line or a rest in the music does not shrink it.
const MIN_LYRIC_WIDTH: f32 = 240.0;
const NOTE: &str = "\u{266A}";
const MAX_LYRIC_WIDTH: f32 = 520.0;
const LYRIC_TRAVEL: f32 = 6.0;

/// Content width of the claimed pill: the artwork and the line; the host adds its own gaps and edges.
pub fn width(lyric_width: f32) -> f32 {
    (ARTWORK_SIZE + lyric_width + LYRIC_SLACK).clamp(MIN_LYRIC_WIDTH, MAX_LYRIC_WIDTH)
}

/// Where the lyric line starts, in device pixels: after the artwork and the gap the host reserved.
fn lyric_left(frame: &Frame) -> f32 {
    frame.bounds.left + (ARTWORK_SIZE + frame.lead_gap) * frame.scale
}

fn draw_art(page: &MediaPage, canvas: &Canvas, area: Rect, opacity: f32) {
    let radius = ARTWORK_RADIUS * canvas.scale();
    match page.artwork.brush().or(page.app_icon.brush()) {
        Some(image) => canvas.image(image, area, radius, opacity),
        None => canvas.text(&styles::RESTING_LABEL, NOTE, area, Color::WHITE.faded(opacity)),
    }
}

pub fn draw(page: &MediaPage, canvas: &Canvas, frame: &Frame, opacity: f32) {
    let scale = frame.scale;
    let bounds = frame.bounds;
    let row_top = bounds.top;
    let row_bottom = bounds.top + RESTING_HEIGHT * scale;
    let center_y = (row_top + row_bottom) / 2.0;
    let art_size = ARTWORK_SIZE * scale;
    let art_area = Rect::new(bounds.left, center_y - art_size / 2.0, bounds.left + art_size, center_y + art_size / 2.0);
    draw_art(page, canvas, art_area, opacity);

    let (left, right) = (lyric_left(frame), bounds.right);
    let transition = page.follower.transition().clamp(0.0, 1.0);
    let travel = LYRIC_TRAVEL * scale;
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
    fn the_line_starts_after_the_artwork_and_the_gap_the_host_reserved() {
        let frame = |lead_gap| Frame {
            scale: 2.0,
            bounds: Rect::new(100.0, 0.0, 600.0, 74.0),
            expansion: 0.0,
            expanded: Size::new(0.0, 0.0),
            lead_gap,
        };
        assert_eq!(lyric_left(&frame(0.0)), 100.0 + ARTWORK_SIZE * 2.0);
        assert_eq!(lyric_left(&frame(26.0)), 100.0 + (ARTWORK_SIZE + 26.0) * 2.0);
    }
}
