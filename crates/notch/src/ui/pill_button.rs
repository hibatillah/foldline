use crate::gfx::{Align, Canvas, Color, FontStyle, Rect, UI_FONT, Weight};

const LABEL: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Medium, Align::Center);
const FILL: Color = Color::gray(0.137);
const FILL_HOVERED: Color = Color::gray(0.2);
const DANGER_FILL: Color = Color::rgb(0.55, 0.1, 0.12);
const DANGER_FILL_HOVERED: Color = Color::rgb(0.67, 0.14, 0.16);
const ACCENT_LABEL: Color = Color::gray(0.05);
/// Accent colors whose channels sum above this are light, so hovering darkens them instead of lightening.
const LIGHT_ACCENT_SUM: f32 = 2.7;

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
        let (fill, text) = self.colors();
        canvas.fill_rounded_rect(self.area, radius, fill.faded(opacity));
        canvas.text(&LABEL, self.label, self.area, text.faded(opacity));
    }

    /// The background and label colors for the button's kind and hover state.
    fn colors(&self) -> (Color, Color) {
        if self.is_destructive {
            (if self.is_hovered { DANGER_FILL_HOVERED } else { DANGER_FILL }, Color::WHITE)
        } else if self.is_primary {
            let accent = self.accent;
            let is_light = accent.r + accent.g + accent.b > LIGHT_ACCENT_SUM;
            let lift = match (self.is_hovered, is_light) {
                (false, _) => 0.0,
                (true, false) => 0.12,
                (true, true) => -0.08,
            };
            let lifted = |channel: f32| (channel + lift).clamp(0.0, 1.0);
            (Color::rgb(lifted(accent.r), lifted(accent.g), lifted(accent.b)), ACCENT_LABEL)
        } else {
            (if self.is_hovered { FILL_HOVERED } else { FILL }, Color::WHITE)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn button(is_primary: bool, is_destructive: bool, is_hovered: bool, accent: Color) -> PillButton<'static> {
        PillButton { label: "Save", area: Rect::default(), is_primary, is_destructive, is_hovered, accent }
    }

    #[test]
    fn hovering_a_dark_accent_lightens_it() {
        let accent = Color::rgb(0.2, 0.4, 0.8);
        let (resting, _) = button(true, false, false, accent).colors();
        let (hovered, _) = button(true, false, true, accent).colors();
        assert_eq!(resting, accent);
        assert!(hovered.r > resting.r && hovered.b > resting.b);
    }

    #[test]
    fn hovering_a_light_accent_darkens_it() {
        let accent = Color::rgb(0.95, 0.95, 0.95);
        let (hovered, _) = button(true, false, true, accent).colors();
        assert!(hovered.r < accent.r);
    }

    #[test]
    fn a_destructive_button_ignores_the_accent_and_keeps_a_white_label() {
        let (fill, label) = button(true, true, false, Color::rgb(0.2, 0.4, 0.8)).colors();
        assert_eq!((fill, label), (DANGER_FILL, Color::WHITE));
    }

    #[test]
    fn a_plain_button_shows_a_lighter_fill_when_hovered() {
        let (resting, _) = button(false, false, false, Color::WHITE).colors();
        let (hovered, _) = button(false, false, true, Color::WHITE).colors();
        assert!(hovered.r > resting.r);
    }
}
