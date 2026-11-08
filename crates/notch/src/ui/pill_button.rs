use crate::gfx::{Align, Canvas, Color, FontStyle, Rect, UI_FONT, Weight};

const LABEL: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Medium, Align::Center);
const FILL: Color = Color::gray(0.137);
const FILL_HOVERED: Color = Color::gray(0.2);
const DANGER_FILL: Color = Color::rgb(0.55, 0.1, 0.12);
const DANGER_FILL_HOVERED: Color = Color::rgb(0.67, 0.14, 0.16);
const ACCENT_LABEL: Color = Color::gray(0.05);

pub struct PillButton<'a> {
    pub label: &'a str,
    pub area: Rect,
    pub is_primary: bool,
    pub is_destructive: bool,
    pub is_hovered: bool,
    pub accent: Color,
}

impl PillButton<'_> {
    pub fn draw(&self, canvas: &Canvas, opacity: f32) {
        let radius = self.area.height() / 2.0;
        let (fill, text) = if self.is_destructive {
            (if self.is_hovered { DANGER_FILL_HOVERED } else { DANGER_FILL }, Color::WHITE)
        } else if self.is_primary {
            let accent = self.accent;
            let is_light = accent.r + accent.g + accent.b > 2.7;
            let lift = match (self.is_hovered, is_light) {
                (false, _) => 0.0,
                (true, false) => 0.12,
                (true, true) => -0.08,
            };
            (
                Color::rgb(
                    (accent.r + lift).clamp(0.0, 1.0),
                    (accent.g + lift).clamp(0.0, 1.0),
                    (accent.b + lift).clamp(0.0, 1.0),
                ),
                ACCENT_LABEL,
            )
        } else {
            (if self.is_hovered { FILL_HOVERED } else { FILL }, Color::WHITE)
        };
        canvas.fill_rounded_rect(self.area, radius, fill.faded(opacity));
        canvas.text(&LABEL, self.label, self.area, text.faded(opacity));
    }
}
