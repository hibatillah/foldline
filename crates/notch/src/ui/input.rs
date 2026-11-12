use crate::gfx::Point;

/// Pointer input, delivered to visible open pages in their own shifted frames. Positions are units from the page's top-left.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PointerEvent {
    Moved(Point),
    Pressed(Point),
    Released(Point),
    Left,
    CaptureLost,
    SecondaryReleased(Point),
    /// The wheel or a touchpad swipe over a page that `Page::claims_scroll`: positive moves toward later content.
    Scrolled {
        notches: f32,
    },
}

/// Typing delivered to the page or activity that holds the keyboard (`Context::take_keyboard`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyInput {
    /// Typed or pasted characters, without line breaks or other control characters.
    Text(String),
    Backspace,
    Delete,
    /// Ctrl+Backspace: the word before the caret.
    BackspaceWord,
    /// Ctrl+Delete: the word after the caret.
    DeleteWord,
    Left,
    Right,
    Home,
    End,
    Enter,
    Escape,
}
