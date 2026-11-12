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
        let scale = canvas.scale();
        if self.is_hovered {
            let half = (size / 2.0 + WASH_MARGIN) * scale;
            let wash = Rect::around(self.center, half, half);
            canvas.fill_rounded_rect(wash, WASH_RADIUS * scale, Color::WHITE.with_alpha(WASH_ALPHA * opacity));
        }
        self.icon.draw(canvas, self.center, size * scale, self.color, opacity);
    }
}
