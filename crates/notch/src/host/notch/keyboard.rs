use super::Notch;
use crate::{log, page::KeyInput};
use windows::Win32::Foundation::HWND;

/// The page typing goes to, the window that had the keyboard before it, and half-received characters.
#[derive(Default)]
pub(super) struct KeyboardState {
    /// Index of the page that holds the keyboard.
    pub(super) owner: Option<usize>,
    pub(super) previous: Option<HWND>,
    /// The first half of a character outside the basic plane, waiting for its second `WM_CHAR`.
    pub(super) pending_high_surrogate: Option<u16>,
}

/// The keys a text field uses; Ctrl+V pastes the clipboard's text on one line.
pub(super) fn key_input(virtual_key: u16, is_control_down: bool) -> Option<KeyInput> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        VK_BACK, VK_DELETE, VK_END, VK_ESCAPE, VK_HOME, VK_LEFT, VK_RETURN, VK_RIGHT,
    };
    const VK_V: u16 = 0x56;
    Some(match virtual_key {
        key if key == VK_BACK.0 => KeyInput::Backspace,
        key if key == VK_DELETE.0 => KeyInput::Delete,
        key if key == VK_LEFT.0 => KeyInput::Left,
        key if key == VK_RIGHT.0 => KeyInput::Right,
        key if key == VK_HOME.0 => KeyInput::Home,
        key if key == VK_END.0 => KeyInput::End,
        key if key == VK_RETURN.0 => KeyInput::Enter,
        key if key == VK_ESCAPE.0 => KeyInput::Escape,
        VK_V if is_control_down => KeyInput::Text(single_line(&crate::platform::clipboard_text()?)),
        _ => return None,
    })
}

/// Pasted text on one line: line breaks and tabs become spaces, other control characters go.
pub(super) fn single_line(text: &str) -> String {
    let joined: String = text
        .chars()
        .map(|character| if matches!(character, '\r' | '\n' | '\t') { ' ' } else { character })
        .filter(|character| !character.is_control())
        .collect();
    joined.split_whitespace().collect::<Vec<_>>().join(" ")
}

impl Notch {
    pub(super) fn set_keyboard(&mut self, index: usize, wants_keyboard: bool) {
        if wants_keyboard {
            self.take_keyboard_for(index);
        } else if self.keyboard.owner == Some(index) {
            self.give_keyboard_back();
        }
    }

    pub(super) fn take_keyboard_for(&mut self, owner: usize) {
        if self.keyboard.owner == Some(owner) {
            return;
        }
        match self.keyboard.owner.replace(owner) {
            Some(previous_owner) => self.keyboard_lost_by(previous_owner),
            None => self.keyboard.previous = crate::host::window::take_keyboard(self.window),
        }
        log::debug(&format!("keyboard taken by page {owner}"));
    }

    pub(super) fn give_keyboard_back(&mut self) {
        let Some(owner) = self.keyboard.owner.take() else { return };
        self.keyboard.pending_high_surrogate = None;
        crate::host::window::give_back_keyboard(self.keyboard.previous.take());
        log::debug(&format!("keyboard given back by page {owner}"));
        self.close_if_pointer_is_away();
    }

    /// The notch lost activation to another window while a page was typing.
    pub fn deactivated(&mut self) {
        let Some(owner) = self.keyboard.owner.take() else { return };
        self.keyboard.previous = None;
        self.keyboard.pending_high_surrogate = None;
        log::debug(&format!("keyboard lost by page {owner}"));
        self.keyboard_lost_by(owner);
        self.close_if_pointer_is_away();
        self.flush();
    }

    pub(super) fn keyboard_lost_by(&mut self, owner: usize) {
        self.with_page(owner, |page, context| page.on_keyboard_lost(context));
    }

    /// Handles `WM_KEYDOWN`; returns false for keys no page wants, so Windows handles them.
    pub fn key_pressed(&mut self, virtual_key: u16, is_control_down: bool) -> bool {
        let Some(owner) = self.keyboard.owner else { return false };
        let Some(key) = key_input(virtual_key, is_control_down) else { return false };
        self.deliver_key(owner, key);
        self.flush();
        true
    }

    /// Handles `WM_CHAR`, joining surrogate pairs and skipping control characters.
    pub fn character_typed(&mut self, unit: u16) {
        let Some(owner) = self.keyboard.owner else { return };
        let units: Vec<u16> = match (self.keyboard.pending_high_surrogate.take(), unit) {
            (_, 0xD800..=0xDBFF) => {
                self.keyboard.pending_high_surrogate = Some(unit);
                return;
            }
            (Some(high), 0xDC00..=0xDFFF) => vec![high, unit],
            _ => vec![unit],
        };
        let typed: String =
            char::decode_utf16(units).filter_map(Result::ok).filter(|character| !character.is_control()).collect();
        if typed.is_empty() {
            return;
        }
        self.deliver_key(owner, KeyInput::Text(typed));
        self.flush();
    }

    pub(super) fn deliver_key(&mut self, owner: usize, key: KeyInput) {
        self.with_page(owner, |page, context| page.on_key(key, context));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pasted_text_lands_on_one_line_without_control_characters() {
        assert_eq!(single_line("Buy milk\r\nand\teggs\u{7}"), "Buy milk and eggs");
        assert_eq!(single_line("   "), "");
    }
    #[test]
    fn only_editing_keys_reach_a_text_field() {
        use windows::Win32::UI::Input::KeyboardAndMouse::{VK_BACK, VK_ESCAPE, VK_F5, VK_RETURN};
        assert_eq!(key_input(VK_BACK.0, false), Some(KeyInput::Backspace));
        assert_eq!(key_input(VK_RETURN.0, false), Some(KeyInput::Enter));
        assert_eq!(key_input(VK_ESCAPE.0, false), Some(KeyInput::Escape));
        assert_eq!(key_input(VK_F5.0, false), None);
        assert_eq!(key_input(0x56, false), None);
    }
}
