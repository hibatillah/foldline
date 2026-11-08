//! Notification text and privacy-mode presentation.

use std::{sync::Arc, time::Instant};

use crate::gfx::{Color, ImageData};

pub const PRIVATE_TEXT: &str = "New message";

#[derive(Clone)]
pub struct Item {
    pub id: u32,
    pub app: String,
    pub app_id: String,
    pub icon: Option<Arc<ImageData>>,
    pub title: String,
    pub body: String,
    /// When the listener read it, so the debug log can tell how long the banner took after that.
    pub read_at: Instant,
}

const PALETTE: [Color; 8] = [
    Color::rgb(0.345, 0.396, 0.949),
    Color::rgb(0.169, 0.541, 0.243),
    Color::rgb(0.788, 0.165, 0.165),
    Color::rgb(0.110, 0.494, 0.839),
    Color::rgb(0.910, 0.349, 0.047),
    Color::rgb(0.404, 0.267, 0.851),
    Color::rgb(0.047, 0.522, 0.600),
    Color::rgb(0.761, 0.145, 0.361),
];

pub fn app_color(app: &str) -> Color {
    let hash = app.bytes().fold(5381u32, |hash, byte| hash.wrapping_mul(33).wrapping_add(u32::from(byte)));
    PALETTE[hash as usize % PALETTE.len()]
}

pub fn app_letter(app: &str) -> String {
    app.chars()
        .find(|character| character.is_alphanumeric())
        .map(|character| character.to_uppercase().collect())
        .unwrap_or_else(|| "?".to_owned())
}

/// The heading: the sender in white, then the app name in grey. A sender that is empty or only whitespace is missing,
/// and the app name takes its place in white with nothing after it.
#[derive(Debug, PartialEq, Eq)]
pub struct Heading {
    pub lead: String,
    pub app: Option<String>,
}

pub fn heading(item: &Item) -> Heading {
    let sender = item.title.trim();
    if sender.is_empty() {
        Heading { lead: item.app.clone(), app: None }
    } else {
        Heading { lead: sender.to_owned(), app: Some(item.app.clone()) }
    }
}

pub fn body_text(item: &Item, shows_text: bool) -> String {
    if !shows_text { PRIVATE_TEXT.to_owned() } else { item.body.split_whitespace().collect::<Vec<_>>().join(" ") }
}

/// What the primary button says; it names the app even in privacy mode.
pub fn open_label(item: &Item) -> String {
    format!("Open {}", item.app)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: u32, title: &str, body: &str) -> Item {
        Item {
            id,
            app: "Discord".to_owned(),
            app_id: String::new(),
            icon: None,
            title: title.to_owned(),
            body: body.to_owned(),
            read_at: Instant::now(),
        }
    }

    #[test]
    fn an_app_always_gets_the_same_color_and_a_letter() {
        assert_eq!(app_color("Discord"), app_color("Discord"));
        assert_eq!(app_letter("discord"), "D");
        assert_eq!(app_letter("  \u{1f600} mail"), "M");
        assert_eq!(app_letter(""), "?");
    }

    #[test]
    fn the_heading_names_the_sender_then_the_app() {
        assert_eq!(
            heading(&item(1, "Rafi", "hi")),
            Heading { lead: "Rafi".to_owned(), app: Some("Discord".to_owned()) }
        );
    }

    #[test]
    fn a_missing_sender_leaves_the_app_name_alone_in_the_lead() {
        let expected = Heading { lead: "Discord".to_owned(), app: None };
        assert_eq!(heading(&item(1, "", "hi")), expected);
        assert_eq!(heading(&item(1, "  \n ", "hi")), expected);
    }

    #[test]
    fn private_mode_leaves_the_text_out() {
        let message = item(1, "Rafi", "Deploy is green.");
        assert_eq!(body_text(&message, false), PRIVATE_TEXT);
        assert_eq!(body_text(&message, true), "Deploy is green.");
    }

    #[test]
    fn a_long_message_is_written_on_one_line_with_single_spaces() {
        assert_eq!(body_text(&item(1, "a", "one   two\nthree"), true), "one two three");
    }

    #[test]
    fn the_open_button_names_the_app() {
        assert_eq!(open_label(&item(1, "Rafi", "hi")), "Open Discord");
    }
}
