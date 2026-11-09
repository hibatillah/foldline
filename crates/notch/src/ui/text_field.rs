//! A one-line text field for pages that hold the keyboard; the caret is steady, so editing runs no timer.

use crate::ui::palette::TEXT_DIM;
use crate::{
    gfx::{Canvas, Color, FontStyle, Rect},
    page::KeyInput,
};

const CARET_WIDTH: f32 = 1.5;
const CARET_INSET: f32 = 3.0;
const CARET: Color = Color::rgb(0.251, 0.561, 1.0);

/// How a field's text looks: its font, color and the hint shown while it is empty.
pub struct FieldLook<'a> {
    pub style: &'a FontStyle,
    pub color: Color,
    pub placeholder: &'a str,
}

/// What a key did to the field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldOutcome {
    Changed,
    Unchanged,
    Submitted,
    Cancelled,
}

#[derive(Clone, Debug, Default)]
pub struct TextField {
    text: String,
    /// A byte offset on a character boundary.
    caret: usize,
    max_characters: usize,
}

impl TextField {
    pub fn new(max_characters: usize) -> Self {
        Self { text: String::new(), caret: 0, max_characters }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.caret = 0;
    }

    pub fn apply(&mut self, key: &KeyInput) -> FieldOutcome {
        match key {
            KeyInput::Text(typed) => {
                let room = self.max_characters.saturating_sub(self.text.chars().count());
                let inserted: String = typed.chars().take(room).collect();
                if inserted.is_empty() {
                    return FieldOutcome::Unchanged;
                }
                self.text.insert_str(self.caret, &inserted);
                self.caret += inserted.len();
                FieldOutcome::Changed
            }
            KeyInput::Backspace => match self.previous_boundary() {
                Some(start) => {
                    self.text.replace_range(start..self.caret, "");
                    self.caret = start;
                    FieldOutcome::Changed
                }
                None => FieldOutcome::Unchanged,
            },
            KeyInput::Delete => match self.next_boundary() {
                Some(end) => {
                    self.text.replace_range(self.caret..end, "");
                    FieldOutcome::Changed
                }
                None => FieldOutcome::Unchanged,
            },
            KeyInput::Left => self.move_caret(self.previous_boundary()),
            KeyInput::Right => self.move_caret(self.next_boundary()),
            KeyInput::Home => self.move_caret((self.caret > 0).then_some(0)),
            KeyInput::End => self.move_caret((self.caret < self.text.len()).then_some(self.text.len())),
            KeyInput::Enter => FieldOutcome::Submitted,
            KeyInput::Escape => FieldOutcome::Cancelled,
        }
    }

    fn move_caret(&mut self, to: Option<usize>) -> FieldOutcome {
        match to {
            Some(position) => {
                self.caret = position;
                FieldOutcome::Changed
            }
            None => FieldOutcome::Unchanged,
        }
    }

    fn previous_boundary(&self) -> Option<usize> {
        self.text[..self.caret].char_indices().next_back().map(|(index, _)| index)
    }

    fn next_boundary(&self) -> Option<usize> {
        self.text[self.caret..].chars().next().map(|character| self.caret + character.len_utf8())
    }

    /// Draws the text left-aligned in `area` (device pixels), scrolled so the caret stays visible.
    pub fn draw(&self, canvas: &Canvas, area: Rect, look: &FieldLook, has_keyboard: bool, opacity: f32) {
        let (scale, style) = (canvas.scale(), look.style);
        canvas.clipped(area, || {
            if self.text.is_empty() {
                canvas.text(style, look.placeholder, area, TEXT_DIM.faded(opacity));
            }
            let caret_x = canvas.measure(style, &self.text[..self.caret]) * scale;
            let shift = (caret_x - area.width() + CARET_INSET * scale).max(0.0);
            let text_area =
                Rect { left: area.left - shift, right: area.right.max(area.left + caret_x + area.width()), ..area };
            canvas.text(style, &self.text, text_area, look.color.faded(opacity));
            if has_keyboard {
                let left = area.left + caret_x - shift;
                let inset = (area.height() * 0.2).max(CARET_INSET * scale);
                let caret = Rect::new(left, area.top + inset, left + CARET_WIDTH * scale, area.bottom - inset);
                canvas.fill_rounded_rect(caret, CARET_WIDTH * scale / 2.0, CARET.faded(opacity));
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typed(field: &mut TextField, text: &str) {
        field.apply(&KeyInput::Text(text.to_owned()));
    }

    #[test]
    fn typing_inserts_at_the_caret_and_moves_it() {
        let mut field = TextField::new(50);
        typed(&mut field, "Buy milk");
        field.apply(&KeyInput::Home);
        typed(&mut field, "Please ");
        assert_eq!(field.text(), "Please Buy milk");
    }

    #[test]
    fn backspace_and_delete_remove_whole_characters() {
        let mut field = TextField::new(50);
        typed(&mut field, "café 🍵");
        assert_eq!(field.apply(&KeyInput::Backspace), FieldOutcome::Changed);
        assert_eq!(field.text(), "café ");
        field.apply(&KeyInput::Left);
        field.apply(&KeyInput::Left);
        assert_eq!(field.apply(&KeyInput::Delete), FieldOutcome::Changed);
        assert_eq!(field.text(), "caf ");
    }

    #[test]
    fn keys_at_the_edges_change_nothing() {
        let mut field = TextField::new(50);
        assert_eq!(field.apply(&KeyInput::Backspace), FieldOutcome::Unchanged);
        assert_eq!(field.apply(&KeyInput::Left), FieldOutcome::Unchanged);
        typed(&mut field, "a");
        assert_eq!(field.apply(&KeyInput::Delete), FieldOutcome::Unchanged);
        assert_eq!(field.apply(&KeyInput::End), FieldOutcome::Unchanged);
    }

    #[test]
    fn the_field_stops_at_its_character_limit() {
        let mut field = TextField::new(5);
        typed(&mut field, "abcdefgh");
        assert_eq!(field.text(), "abcde");
        assert_eq!(field.apply(&KeyInput::Text("x".to_owned())), FieldOutcome::Unchanged);
    }

    #[test]
    fn enter_submits_and_escape_cancels_without_changing_the_text() {
        let mut field = TextField::new(50);
        typed(&mut field, "Ship v0.5");
        assert_eq!(field.apply(&KeyInput::Enter), FieldOutcome::Submitted);
        assert_eq!(field.apply(&KeyInput::Escape), FieldOutcome::Cancelled);
        assert_eq!(field.text(), "Ship v0.5");
    }
}
