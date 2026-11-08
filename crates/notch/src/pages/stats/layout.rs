//! Where things sit inside the cards; every card is 132 x 86 and text keeps the card padding at both sides.

use super::{CARD_HEIGHT, CARD_WIDTH, TILE_GAP, TILE_PADDING_X};
use crate::gfx::{Canvas, Color, FontStyle, Rect};

const LABEL_TOP: f32 = 8.0;
const LABEL_HEIGHT: f32 = 14.0;
const VALUE_TOP: f32 = 19.0;
const VALUE_BOTTOM: f32 = 45.0;
const SUB_TOP: f32 = 64.0;
const NETWORK_KIND_TOP: f32 = 66.0;
const NETWORK_DOWNLOAD_TOP: f32 = 20.0;
const NETWORK_ROW_HEIGHT: f32 = 18.0;
/// From the top of the download row to the top of the upload row; the rows are 18 px high, so 5 px stay between them.
pub(super) const NETWORK_ROW_PITCH: f32 = 23.0;

/// `count` cards of the fixed size side by side from the left of `bounds`, a gap apart.
pub(super) fn card_rects(bounds: Rect, scale: f32, count: usize) -> Vec<Rect> {
    (0..count)
        .map(|index| {
            let left = bounds.left + index as f32 * (CARD_WIDTH + TILE_GAP) * scale;
            Rect::new(left, bounds.top, left + CARD_WIDTH * scale, bounds.top + CARD_HEIGHT * scale)
        })
        .collect()
}

/// The area from `left` (inside the card) to the card padding at the right, between two heights from the card top.
pub(super) fn inside(card: Rect, scale: f32, left: f32, top: f32, bottom: f32) -> Rect {
    Rect::new(
        card.left + left * scale,
        card.top + top * scale,
        card.right - TILE_PADDING_X * scale,
        card.top + bottom * scale,
    )
}

pub(super) fn label_area(card: Rect, scale: f32) -> Rect {
    inside(card, scale, TILE_PADDING_X, LABEL_TOP, LABEL_TOP + LABEL_HEIGHT)
}

pub(super) fn value_area(card: Rect, scale: f32) -> Rect {
    inside(card, scale, TILE_PADDING_X, VALUE_TOP, VALUE_BOTTOM)
}

pub(super) fn sub_area(card: Rect, scale: f32) -> Rect {
    inside(card, scale, TILE_PADDING_X, SUB_TOP, SUB_TOP + LABEL_HEIGHT)
}

pub(super) fn network_kind_area(card: Rect, scale: f32) -> Rect {
    inside(card, scale, TILE_PADDING_X, NETWORK_KIND_TOP, NETWORK_KIND_TOP + LABEL_HEIGHT)
}

/// The download row (`row` 0) or the upload row (`row` 1) of the network card.
pub(super) fn network_row_area(card: Rect, scale: f32, row: usize) -> Rect {
    let top = NETWORK_DOWNLOAD_TOP + row as f32 * NETWORK_ROW_PITCH;
    inside(card, scale, TILE_PADDING_X, top, top + NETWORK_ROW_HEIGHT)
}

pub(super) fn text(canvas: &Canvas, style: &FontStyle, content: &str, area: Rect, color: Color, opacity: f32) {
    canvas.text(style, content, area, color.faded(opacity));
}
