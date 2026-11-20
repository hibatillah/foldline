//! The {{Title}} page. TODO: say in one line what it shows.

use crate::{
    gfx::{Align, Canvas, Color, FontStyle, Rect, Size, UI_FONT, Weight},
    page::{Frame, Mode, Page, PageSizes, Priority, RESTING_HEIGHT},
};

const TEXT_STYLE: FontStyle = FontStyle::new(UI_FONT, 14.0, Weight::Medium, Align::Center);

pub struct {{Name}}Page;

impl {{Name}}Page {
    pub fn new() -> Self {
        Self
    }

    fn title(&self) -> String {
        self.name()
    }
}

impl Page for {{Name}}Page {
    fn id(&self) -> &'static str {
        "{{id}}"
    }

    fn name(&self) -> String {
        crate::i18n::tr!("page.{{id}}").into_owned()
    }

    /// Answers only while the notch is open, so the page never claims the collapsed pill; it is reached by scrolling.
    fn priority(&self, mode: Mode) -> Option<Priority> {
        (mode == Mode::Expanded).then_some(Priority::BACKGROUND)
    }

    fn sizes(&self) -> PageSizes {
        PageSizes { resting_width: 150.0, expanded: Size::new(320.0, 86.0) }
    }

    fn draw_resting(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        let bounds = frame.bounds;
        let row = Rect::new(bounds.left, bounds.top, bounds.right, bounds.top + RESTING_HEIGHT);
        canvas.text(&TEXT_STYLE, &self.title(), row, Color::WHITE.faded(opacity));
    }

    fn draw_expanded(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        canvas.text(&TEXT_STYLE, &self.title(), frame.bounds, Color::WHITE.faded(opacity));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_page_is_reached_by_scrolling_and_never_claims_the_collapsed_pill() {
        let page = {{Name}}Page::new();
        assert_eq!(page.id(), "{{id}}");
        assert!(page.priority(Mode::Resting).is_none());
        assert!(page.priority(Mode::Expanded).is_some());
    }
}
