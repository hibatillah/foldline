//! Answering a notification from the banner: whether its toast offers a reply, and the text being typed.

use crate::i18n::tr;
use crate::{
    gfx::{Canvas, Color, Rect},
    page::KeyInput,
    platform::toast_activation::{self, ReplyAction},
    ui::{BANNER_BUTTON_HEIGHT, BANNER_BUTTONS_GAP_ABOVE, FieldLook, FieldOutcome, TextField, type_styles::REGULAR_13},
};

/// Longest reply the field takes.
const MAX_CHARACTERS: usize = 500;
const FIELD_PADDING: f32 = 14.0;
const FIELD_FILL: Color = Color::WHITE.with_alpha(0.14);

/// What a key did to the reply being typed.
#[derive(Debug, PartialEq, Eq)]
pub enum Typed {
    Editing,
    Send(String),
    Cancelled,
}

#[derive(Default)]
pub struct ReplyState {
    offer: Option<ReplyAction>,
    is_checked: bool,
    field: Option<TextField>,
}

impl ReplyState {
    /// Reads the toast once for a reply it offers; returns whether the card's buttons must change for it.
    pub fn check(&mut self, notification_id: u32) -> bool {
        if std::mem::replace(&mut self.is_checked, true) {
            return false;
        }
        self.offer = toast_activation::reply_offered(notification_id);
        self.offer.is_some()
    }

    pub fn offer(&self) -> Option<&ReplyAction> {
        self.offer.as_ref()
    }

    pub fn is_typing(&self) -> bool {
        self.field.is_some()
    }

    pub fn start_typing(&mut self) {
        if self.offer.is_some() {
            self.field = Some(TextField::new(MAX_CHARACTERS));
        }
    }

    pub fn stop_typing(&mut self) {
        self.field = None;
    }

    pub fn apply(&mut self, key: &KeyInput) -> (Typed, bool) {
        let Some(field) = &mut self.field else { return (Typed::Cancelled, false) };
        match field.apply(key) {
            FieldOutcome::Changed => (Typed::Editing, true),
            FieldOutcome::Unchanged => (Typed::Editing, false),
            FieldOutcome::Cancelled => (Typed::Cancelled, true),
            FieldOutcome::Submitted => {
                let text = field.text().trim().to_owned();
                (if text.is_empty() { Typed::Editing } else { Typed::Send(text) }, false)
            }
        }
    }

    /// Draws the field in the row the buttons use; `row_right` and `row_top` are the buttons' origin in units.
    pub fn draw(&self, canvas: &Canvas, row_left: f32, row_right: f32, row_top: f32, opacity: f32) {
        let (Some(field), Some(offer)) = (&self.field, &self.offer) else { return };
        let top = row_top + BANNER_BUTTONS_GAP_ABOVE;
        let row = Rect::new(row_left, top, row_right, top + BANNER_BUTTON_HEIGHT);
        canvas.fill_rounded_rect(row, row.height() / 2.0, FIELD_FILL.faded(opacity));
        let text_area = Rect::new(row.left + FIELD_PADDING, row.top, row.right - FIELD_PADDING, row.bottom);
        let hint = if offer.placeholder.is_empty() {
            tr!("notification.reply_hint").into_owned()
        } else {
            offer.placeholder.clone()
        };
        let look = FieldLook { style: &REGULAR_13, color: Color::WHITE, placeholder: &hint };
        field.draw(canvas, text_area, &look, true, opacity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typing() -> ReplyState {
        let offer =
            ReplyAction { input_id: "reply".to_owned(), arguments: "key=1".to_owned(), placeholder: String::new() };
        let mut state = ReplyState { offer: Some(offer), is_checked: true, field: None };
        state.start_typing();
        state
    }

    #[test]
    fn typing_starts_only_when_the_toast_offers_a_reply() {
        let mut state = ReplyState::default();
        state.start_typing();
        assert!(!state.is_typing());
        assert!(typing().is_typing());
    }

    #[test]
    fn enter_sends_the_trimmed_text() {
        let mut state = typing();
        state.apply(&KeyInput::Text("  on my way ".to_owned()));
        assert_eq!(state.apply(&KeyInput::Enter).0, Typed::Send("on my way".to_owned()));
    }

    #[test]
    fn enter_on_an_empty_field_sends_nothing() {
        assert_eq!(typing().apply(&KeyInput::Enter).0, Typed::Editing);
    }

    #[test]
    fn escape_cancels() {
        assert_eq!(typing().apply(&KeyInput::Escape).0, Typed::Cancelled);
    }
}
