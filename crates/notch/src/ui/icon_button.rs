use super::icons::Icon;
use crate::gfx::{Canvas, Color, Point, Rect};

/// The wash reaches this far past the icon's edge.
const WASH_MARGIN: f32 = 8.0;
const WASH_RADIUS: f32 = 8.0;
const WASH_ALPHA: f32 = 0.12;

pub struct IconButton {
    pub icon: Icon,
    pub center: Point,
    pub color: Color,
    pub is_hovered: bool,
}

impl IconButton {
    /// `size` is the icon's edge in DIP; `Icon::size` gives its preferred one.
    pub fn draw(&self, canvas: &Canvas, opacity: f32, size: f32) {
        if self.is_hovered {
            draw_hover_wash(canvas, self.center, size + 2.0 * WASH_MARGIN, opacity);
        }
        self.icon.draw(canvas, self.center, size * canvas.scale(), self.color, opacity);
    }
}

/// The hovered icon button's rounded white square, `edge` DIP wide around `center` (in the canvas's space).
pub fn draw_hover_wash(canvas: &Canvas, center: Point, edge: f32, opacity: f32) {
    let scale = canvas.scale();
    let half = edge / 2.0 * scale;
    let wash = Rect::around(center, half, half);
    canvas.fill_rounded_rect(wash, WASH_RADIUS * scale, Color::WHITE.with_alpha(WASH_ALPHA * opacity));
}
