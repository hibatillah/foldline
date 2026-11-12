//! Digit keys that jump to a page while the open notch is hovered: 1 to 9 for the first nine pages in the settings
//! order, 0 for the last. They are hotkeys held only for that time, because the notch has no keyboard focus then.

use windows::Win32::UI::Input::KeyboardAndMouse::{MOD_NOREPEAT, RegisterHotKey, UnregisterHotKey};

use super::Notch;
use crate::log;

/// Hotkey ids of the digits 0 to 9 start here.
pub(super) const HOTKEY_BASE_ID: i32 = 0x4E00;
const DIGIT_COUNT: usize = 10;
const VIRTUAL_KEY_ZERO: u32 = 0x30;

/// The digits registered right now.
#[derive(Default)]
pub(super) struct PageHotkeys {
    registered: [bool; DIGIT_COUNT],
}

/// Where in the page order `digit` leads for `page_count` pages: 1 to 9 the same place, 0 the last; `None` for a digit
/// with no page.
pub(super) fn target_position(digit: usize, page_count: usize) -> Option<usize> {
    match digit {
        1..=9 => (digit <= page_count).then_some(digit - 1),
        0 => page_count.checked_sub(1),
        _ => None,
    }
}

fn wanted_digits(page_count: usize) -> [bool; DIGIT_COUNT] {
    std::array::from_fn(|digit| target_position(digit, page_count).is_some())
}

impl Notch {
    /// Holds the digit hotkeys while the pointer is on the open notch and no page is typing; gives them back otherwise.
    pub(super) fn sync_page_hotkeys(&mut self) {
        let is_wanted =
            !self.is_paused() && self.open.is_open && self.frame().is_open() && self.keyboard.owner.is_none();
        let wanted = if is_wanted { wanted_digits(self.navigable.len()) } else { [false; DIGIT_COUNT] };
        for (digit, &is_wanted) in wanted.iter().enumerate() {
            let is_registered = self.page_hotkeys.registered[digit];
            if is_wanted == is_registered {
                continue;
            }
            let id = HOTKEY_BASE_ID + digit as i32;
            if is_wanted {
                let key = VIRTUAL_KEY_ZERO + digit as u32;
                match unsafe { RegisterHotKey(Some(self.window), id, MOD_NOREPEAT, key) } {
                    Ok(()) => self.page_hotkeys.registered[digit] = true,
                    Err(error) => log::debug(&format!("could not register the {digit} key: {error}")),
                }
            } else {
                if let Err(error) = unsafe { UnregisterHotKey(Some(self.window), id) } {
                    log::debug(&format!("could not release the {digit} key: {error}"));
                }
                self.page_hotkeys.registered[digit] = false;
            }
        }
    }

    /// Handles `WM_HOTKEY` for a digit.
    pub fn hotkey_pressed(&mut self, id: i32) {
        let digit = (id - HOTKEY_BASE_ID) as usize;
        let Some(&page) =
            target_position(digit, self.navigable.len()).and_then(|position| self.navigable.get(position))
        else {
            return;
        };
        self.go_to_page(page);
        self.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digits_one_to_nine_lead_to_the_pages_in_order() {
        assert_eq!(target_position(1, 4), Some(0));
        assert_eq!(target_position(4, 4), Some(3));
        assert_eq!(target_position(5, 4), None);
    }

    #[test]
    fn zero_leads_to_the_last_page_and_pages_past_the_ninth_have_no_key() {
        assert_eq!(target_position(0, 4), Some(3));
        assert_eq!(target_position(0, 12), Some(11));
        assert_eq!(target_position(9, 12), Some(8));
    }

    #[test]
    fn no_page_means_no_key() {
        assert_eq!(wanted_digits(0), [false; DIGIT_COUNT]);
        assert!(wanted_digits(2)[1] && wanted_digits(2)[2] && wanted_digits(2)[0] && !wanted_digits(2)[3]);
    }
}
