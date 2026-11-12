//! A one-line text field for pages that hold the keyboard; the caret is steady, so editing runs no timer.

use super::{KeyInput, palette::TEXT_DIM};
use crate::gfx::{Canvas, Color, FontStyle, Rect};

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
            KeyInput::BackspaceWord => self.remove_range(word_start_before(&self.text, self.caret), self.caret),
            KeyInput::DeleteWord => self.remove_range(self.caret, word_end_after(&self.text, self.caret)),
            KeyInput::Left => self.move_caret(self.previous_boundary()),
            KeyInput::Right => self.move_caret(self.next_boundary()),
            KeyInput::Home => self.move_caret((self.caret > 0).then_some(0)),
            KeyInput::End => self.move_caret((self.caret < self.text.len()).then_some(self.text.len())),
            KeyInput::Enter => FieldOutcome::Submitted,
            KeyInput::Escape => FieldOutcome::Cancelled,
        }
    }

    fn remove_range(&mut self, start: usize, end: usize) -> FieldOutcome {
        if start == end {
            return FieldOutcome::Unchanged;
        }
        self.text.replace_range(start..end, "");
        self.caret = start;
        FieldOutcome::Changed
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

    /// Draws the text left-aligned in `area` (the canvas's space), scrolled so the caret stays visible.
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

/// What a word delete treats as one run: letters, digits and `_` form words, other visible characters form their own.
#[derive(Clone, Copy, PartialEq, Eq)]
enum CharacterClass {
    Space,
    Word,
    Symbol,
}

impl CharacterClass {
    fn of(character: char) -> Self {
        if character.is_whitespace() {
            CharacterClass::Space
        } else if character.is_alphanumeric() || character == '_' {
            CharacterClass::Word
        } else {
            CharacterClass::Symbol
        }
    }
}

/// Where the run of `class` that ends at byte `end` starts.
fn run_start(text: &str, end: usize, class: CharacterClass) -> usize {
    text[..end]
        .char_indices()
        .rev()
        .take_while(|&(_, character)| CharacterClass::of(character) == class)
        .last()
        .map_or(end, |(index, _)| index)
}

/// Where the run of `class` that starts at byte `start` ends.
fn run_end(text: &str, start: usize, class: CharacterClass) -> usize {
    text[start..]
        .char_indices()
        .take_while(|&(_, character)| CharacterClass::of(character) == class)
        .last()
        .map_or(start, |(index, character)| start + index + character.len_utf8())
}

/// The start of the word Ctrl+Backspace removes: the spaces before `caret`, then the run before them.
fn word_start_before(text: &str, caret: usize) -> usize {
    let before_spaces = run_start(text, caret, CharacterClass::Space);
    match text[..before_spaces].chars().next_back() {
        Some(character) => run_start(text, before_spaces, CharacterClass::of(character)),
        None => before_spaces,
    }
}

/// The end of the word Ctrl+Delete removes: the spaces after `caret`, then the run after them.
fn word_end_after(text: &str, caret: usize) -> usize {
    let after_spaces = run_end(text, caret, CharacterClass::Space);
    match text[after_spaces..].chars().next() {
        Some(character) => run_end(text, after_spaces, CharacterClass::of(character)),
        None => after_spaces,
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

    fn field_with_caret(text: &str, left_presses: usize) -> TextField {
        let mut field = TextField::new(50);
        typed(&mut field, text);
        for _ in 0..left_presses {
            field.apply(&KeyInput::Left);
        }
        field
    }

    #[test]
    fn ctrl_backspace_removes_the_spaces_before_the_caret_and_the_word_before_them() {
        let mut field = field_with_caret("Buy fresh milk  ", 0);
        assert_eq!(field.apply(&KeyInput::BackspaceWord), FieldOutcome::Changed);
        assert_eq!(field.text(), "Buy fresh ");
        field.apply(&KeyInput::BackspaceWord);
        assert_eq!(field.text(), "Buy ");
        typed(&mut field, "x");
        assert_eq!(field.text(), "Buy x", "the caret sits where the word was");
    }

    #[test]
    fn ctrl_delete_removes_the_spaces_after_the_caret_and_the_word_after_them() {
        let mut field = field_with_caret("Buy   fresh milk", 13);
        assert_eq!(field.apply(&KeyInput::DeleteWord), FieldOutcome::Changed);
        assert_eq!(field.text(), "Buy milk");
        field.apply(&KeyInput::Home);
        field.apply(&KeyInput::DeleteWord);
        assert_eq!(field.text(), " milk");
    }

    #[test]
    fn punctuation_is_a_word_of_its_own() {
        let mut field = field_with_caret("call_back(sam)...", 0);
        field.apply(&KeyInput::BackspaceWord);
        assert_eq!(field.text(), "call_back(sam");
        field.apply(&KeyInput::BackspaceWord);
        assert_eq!(field.text(), "call_back(");
        field.apply(&KeyInput::BackspaceWord);
        assert_eq!(field.text(), "call_back", "the underscore belongs to the word");
        field.apply(&KeyInput::BackspaceWord);
        assert_eq!(field.text(), "");
    }

    #[test]
    fn word_deletes_at_the_ends_of_the_text_change_nothing() {
        let mut field = field_with_caret("milk", 4);
        assert_eq!(field.apply(&KeyInput::BackspaceWord), FieldOutcome::Unchanged);
        field.apply(&KeyInput::End);
        assert_eq!(field.apply(&KeyInput::DeleteWord), FieldOutcome::Unchanged);
        assert_eq!(field.text(), "milk");
        assert_eq!(TextField::new(5).apply(&KeyInput::BackspaceWord), FieldOutcome::Unchanged);
    }

    #[test]
    fn only_spaces_before_the_caret_are_removed_as_a_word() {
        let mut field = field_with_caret("   milk", 4);
        assert_eq!(field.apply(&KeyInput::BackspaceWord), FieldOutcome::Changed);
        assert_eq!(field.text(), "milk");
    }

    #[test]
    fn word_deletes_keep_whole_characters_outside_ascii() {
        let mut field = field_with_caret("kopi Café 🍵 susu", 7);
        field.apply(&KeyInput::BackspaceWord);
        assert_eq!(field.text(), "kopi  🍵 susu", "the accented letter belongs to the word");
        let mut field = field_with_caret("kopi Café 🍵 susu", 5);
        field.apply(&KeyInput::BackspaceWord);
        assert_eq!(field.text(), "kopi Café  susu", "the cup is a symbol run of its own");
        let mut field = field_with_caret("kopi Café 🍵 susu", 5);
        field.apply(&KeyInput::DeleteWord);
        assert_eq!(field.text(), "kopi Café 🍵");
        let mut field = field_with_caret("日本語 テスト", 0);
        field.apply(&KeyInput::BackspaceWord);
        assert_eq!(field.text(), "日本語 ");
    }

    #[test]
    fn a_word_delete_frees_room_under_the_character_limit() {
        let mut field = TextField::new(9);
        typed(&mut field, "milk eggs tea");
        assert_eq!(field.text(), "milk eggs");
        field.apply(&KeyInput::BackspaceWord);
        typed(&mut field, "bread tea");
        assert_eq!(field.text(), "milk brea");
    }
}
