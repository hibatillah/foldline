//! The Dismiss and Open buttons a banner shows while the pointer is on it, right-aligned under its text; a banner
//! that can be answered adds Reply, which then is the main one.

use super::PointerEvent;
use super::svg_icon::{Stroke, draw_strokes};
use super::type_styles::MEDIUM_13;
use crate::gfx::{Canvas, Color, Fonts, Point, Rect, clamp_lines};

/// The words on the buttons; the caller translates them.
#[derive(Clone, Copy, Debug)]
pub struct BannerLabels<'a> {
    pub dismiss: &'a str,
    pub reply: &'a str,
    pub open: &'a str,
}

/// The space the buttons add under a banner's text, in DIP: the gap above them and their height.
pub const BANNER_BUTTONS_HEIGHT: f32 = TOP_GAP + HEIGHT;

const TOP_GAP: f32 = 10.0;
const HEIGHT: f32 = 28.0;
/// The gap above the buttons and the height of one, for a banner that puts something else in their row.
pub const BANNER_BUTTONS_GAP_ABOVE: f32 = TOP_GAP;
pub const BANNER_BUTTON_HEIGHT: f32 = HEIGHT;
const GAP: f32 = 8.0;
const PADDING: f32 = 14.0;
const OPEN_RIGHT_PADDING: f32 = 12.0;
const ICON_SIZE: f32 = 12.0;
const ICON_GAP: f32 = 6.0;
/// Keeps Dismiss and the longest Open inside a 340 DIP notification card's 20 DIP padding.
const MAX_OPEN_LABEL_WIDTH: f32 = 160.0;

const DISMISS_FILL: Color = Color::WHITE.with_alpha(0.14);
const DISMISS_FILL_HOVERED: Color = Color::WHITE.with_alpha(0.22);
const OPEN_FILL_HOVERED: Color = Color::gray(0.86);
/// An arrow pointing up and right, on the 24 unit grid.
const OPEN_ARROW: &str = "M7 17l10-10M8 7h9v9";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BannerButton {
    Dismiss,
    Open,
    Reply,
}

/// The two buttons, measured once when the banner is shown; positions are worked out from the banner's area.
#[derive(Debug, Default)]
pub struct BannerButtons {
    open_label: String,
    /// Read when the buttons are made, so a language change shows on the next banner.
    dismiss_label: String,
    reply_label: String,
    dismiss_width: f32,
    open_width: f32,
    /// The width of the Reply button; none when the banner cannot be answered.
    reply_width: Option<f32>,
    hovered: Option<BannerButton>,
    pressed: Option<BannerButton>,
}

impl BannerButtons {
    /// `labels.open` names the app and is cut with an ellipsis so a long name cannot push Dismiss off the banner;
    /// `labels.reply` is unused.
    pub fn new(fonts: &Fonts, labels: BannerLabels) -> Self {
        let open_label = clamp_lines(labels.open, 1, MAX_OPEN_LABEL_WIDTH, |text| fonts.measure(&MEDIUM_13, text))
            .pop()
            .unwrap_or_default();
        let dismiss_label = labels.dismiss.to_owned();
        let dismiss_width = 2.0 * PADDING + fonts.measure(&MEDIUM_13, &dismiss_label);
        let open_width = PADDING + fonts.measure(&MEDIUM_13, &open_label) + ICON_GAP + ICON_SIZE + OPEN_RIGHT_PADDING;
        Self {
            open_label,
            dismiss_label,
            reply_label: String::new(),
            dismiss_width,
            open_width,
            reply_width: None,
            hovered: None,
            pressed: None,
        }
    }

    /// Dismiss, a plain Open and Reply as the main button.
    pub fn with_reply(fonts: &Fonts, labels: BannerLabels) -> Self {
        let width = |label: &str| 2.0 * PADDING + fonts.measure(&MEDIUM_13, label);
        // Open's label beside Reply, where the app's name no longer fits.
        let (dismiss_label, reply_label, open_label) =
            (labels.dismiss.to_owned(), labels.reply.to_owned(), labels.open.to_owned());
        Self {
            dismiss_width: width(&dismiss_label),
            open_width: width(&open_label),
            reply_width: Some(width(&reply_label)),
            open_label,
            dismiss_label,
            reply_label,
            hovered: None,
            pressed: None,
        }
    }

    #[cfg(test)]
    pub fn dismiss_label(&self) -> &str {
        &self.dismiss_label
    }

    #[cfg(test)]
    pub fn reply_label(&self) -> &str {
        &self.reply_label
    }

    #[cfg(test)]
    pub fn open_label(&self) -> &str {
        &self.open_label
    }

    pub fn has_reply(&self) -> bool {
        self.reply_width.is_some()
    }

    /// Where each button sits, left to right; `right` is the row's right edge and `top` the top of the gap above it, in
    /// device pixels.
    pub fn rects(&self, right: f32, top: f32, scale: f32) -> impl Iterator<Item = (BannerButton, Rect)> {
        let button_top = top + TOP_GAP * scale;
        let bottom = button_top + HEIGHT * scale;
        let (main, open_right) = match self.reply_width {
            Some(width) => (BannerButton::Reply, right - (width + GAP) * scale),
            None => (BannerButton::Open, right),
        };
        let main_width = self.reply_width.unwrap_or(self.open_width);
        let main_rect = Rect::new(right - main_width * scale, button_top, right, bottom);
        let open_left = open_right - self.open_width * scale;
        let dismiss_right =
            if self.reply_width.is_some() { open_left - GAP * scale } else { main_rect.left - GAP * scale };
        let dismiss = Rect::new(dismiss_right - self.dismiss_width * scale, button_top, dismiss_right, bottom);
        let open = Rect::new(open_left, button_top, open_right, bottom);
        [
            Some((BannerButton::Dismiss, dismiss)),
            self.reply_width.map(|_| (BannerButton::Open, open)),
            Some((main, main_rect)),
        ]
        .into_iter()
        .flatten()
    }

    pub fn button_at(&self, point: Point, right: f32, top: f32, scale: f32) -> Option<BannerButton> {
        self.rects(right, top, scale).find(|(_, rect)| rect.contains(point)).map(|(button, _)| button)
    }

    /// Follows the pointer; returns the button that was clicked (pressed and released on it) and whether the look
    /// changed, so the caller repaints.
    pub fn on_pointer(
        &mut self,
        event: PointerEvent,
        right: f32,
        top: f32,
        scale: f32,
    ) -> (Option<BannerButton>, bool) {
        let before = self.hovered;
        let mut clicked = None;
        match event {
            PointerEvent::Moved(point) => self.hovered = self.button_at(point, right, top, scale),
            PointerEvent::Pressed(point) => {
                self.hovered = self.button_at(point, right, top, scale);
                self.pressed = self.hovered;
            }
            PointerEvent::Released(point) => {
                self.hovered = self.button_at(point, right, top, scale);
                clicked = self.pressed.take().filter(|&pressed| Some(pressed) == self.hovered);
            }
            PointerEvent::Left | PointerEvent::CaptureLost => {
                self.hovered = None;
                self.pressed = None;
            }
            PointerEvent::SecondaryReleased(_) | PointerEvent::Scrolled { .. } => {}
        }
        (clicked, before != self.hovered)
    }

    pub fn draw(&self, canvas: &Canvas, right: f32, top: f32, scale: f32, opacity: f32) {
        for (button, rect) in self.rects(right, top, scale) {
            let is_hovered = self.hovered == Some(button);
            let radius = rect.height() / 2.0;
            let label_left = rect.left + PADDING * scale;
            match button {
                BannerButton::Dismiss => {
                    self.draw_secondary(canvas, rect, &self.dismiss_label, is_hovered, scale, opacity)
                }
                BannerButton::Open if self.has_reply() => {
                    self.draw_secondary(canvas, rect, &self.open_label, is_hovered, scale, opacity);
                }
                BannerButton::Reply => {
                    let fill = if is_hovered { OPEN_FILL_HOVERED } else { Color::WHITE };
                    canvas.fill_rounded_rect(rect, radius, fill.faded(opacity));
                    canvas.text(
                        &MEDIUM_13,
                        &self.reply_label,
                        Rect { left: label_left, ..rect },
                        Color::BLACK.faded(opacity),
                    );
                }
                BannerButton::Open => {
                    let fill = if is_hovered { OPEN_FILL_HOVERED } else { Color::WHITE };
                    canvas.fill_rounded_rect(rect, radius, fill.faded(opacity));
                    canvas.text(
                        &MEDIUM_13,
                        &self.open_label,
                        Rect { left: label_left, ..rect },
                        Color::BLACK.faded(opacity),
                    );
                    let icon_center =
                        Point::new(rect.right - (OPEN_RIGHT_PADDING + ICON_SIZE / 2.0) * scale, rect.center_y());
                    // The 12 px design arrow is drawn on the 24 unit grid at twice its line width.
                    draw_strokes(
                        canvas,
                        &[Stroke::new(OPEN_ARROW, Color::BLACK)],
                        icon_center,
                        ICON_SIZE * scale,
                        3.0,
                        opacity,
                    );
                }
            }
        }
    }

    fn draw_secondary(&self, canvas: &Canvas, rect: Rect, label: &str, is_hovered: bool, scale: f32, opacity: f32) {
        let fill = if is_hovered { DISMISS_FILL_HOVERED } else { DISMISS_FILL };
        canvas.fill_rounded_rect(rect, rect.height() / 2.0, fill.faded(opacity));
        let label_area = Rect { left: rect.left + PADDING * scale, ..rect };
        canvas.text(&MEDIUM_13, label, label_area, Color::WHITE.faded(opacity));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LABELS: BannerLabels = BannerLabels { dismiss: "Dismiss", reply: "Reply", open: "Open" };

    fn buttons() -> BannerButtons {
        BannerButtons::new(&Fonts::new(1.0).unwrap(), BannerLabels { open: "Open Discord", ..LABELS })
    }

    #[test]
    fn the_open_button_ends_at_the_right_edge_and_dismiss_sits_left_of_it() {
        let [(dismiss, dismiss_rect), (open, open_rect)] = buttons().rects(380.0, 60.0, 1.0).collect::<Vec<_>>()[..]
        else {
            panic!("two buttons")
        };
        assert_eq!((dismiss, open), (BannerButton::Dismiss, BannerButton::Open));
        assert_eq!(open_rect.right, 380.0);
        assert_eq!(dismiss_rect.right, open_rect.left - GAP);
        assert_eq!(open_rect.top, 60.0 + TOP_GAP);
        assert_eq!(open_rect.height(), HEIGHT);
    }

    #[test]
    fn a_press_and_release_on_the_same_button_is_a_click() {
        let mut buttons = buttons();
        let open = buttons.rects(380.0, 60.0, 1.0).last().unwrap().1;
        let point = Point::new(open.center_x(), open.center_y());
        buttons.on_pointer(PointerEvent::Pressed(point), 380.0, 60.0, 1.0);
        let (clicked, _) = buttons.on_pointer(PointerEvent::Released(point), 380.0, 60.0, 1.0);
        assert_eq!(clicked, Some(BannerButton::Open));
    }

    #[test]
    fn releasing_on_another_button_than_the_pressed_one_clicks_nothing() {
        let mut buttons = buttons();
        let rects: Vec<_> = buttons.rects(380.0, 60.0, 1.0).collect();
        let (dismiss, open) = (rects[0].1, rects[1].1);
        buttons.on_pointer(PointerEvent::Pressed(Point::new(dismiss.center_x(), dismiss.center_y())), 380.0, 60.0, 1.0);
        let released = PointerEvent::Released(Point::new(open.center_x(), open.center_y()));
        assert_eq!(buttons.on_pointer(released, 380.0, 60.0, 1.0).0, None);
    }

    #[test]
    fn a_click_on_the_text_of_the_banner_is_no_button() {
        assert_eq!(buttons().button_at(Point::new(100.0, 20.0), 380.0, 60.0, 1.0), None);
    }

    #[test]
    fn moving_onto_a_button_asks_for_a_repaint_and_staying_on_it_does_not() {
        let mut buttons = buttons();
        let dismiss = buttons.rects(380.0, 60.0, 1.0).next().unwrap().1;
        let point = Point::new(dismiss.center_x(), dismiss.center_y());
        assert!(buttons.on_pointer(PointerEvent::Moved(point), 380.0, 60.0, 1.0).1);
        assert!(!buttons.on_pointer(PointerEvent::Moved(point), 380.0, 60.0, 1.0).1);
    }

    #[test]
    fn a_long_app_name_keeps_the_open_button_narrower_than_its_cap() {
        let open = format!("Open {}", "Very long app name ".repeat(6));
        let buttons = BannerButtons::new(&Fonts::new(1.0).unwrap(), BannerLabels { open: &open, ..LABELS });
        assert!(buttons.open_label.ends_with('\u{2026}'));
        assert!(buttons.open_width <= PADDING + MAX_OPEN_LABEL_WIDTH + ICON_GAP + ICON_SIZE + OPEN_RIGHT_PADDING);
    }

    #[test]
    fn with_a_reply_the_reply_button_is_rightmost_and_open_and_dismiss_sit_left_of_it() {
        let buttons = BannerButtons::with_reply(&Fonts::new(1.0).unwrap(), LABELS);
        let rects: Vec<_> = buttons.rects(380.0, 60.0, 1.0).collect();
        let kinds: Vec<_> = rects.iter().map(|(button, _)| *button).collect();
        assert_eq!(kinds, [BannerButton::Dismiss, BannerButton::Open, BannerButton::Reply]);
        assert_eq!(rects[2].1.right, 380.0);
        assert_eq!(rects[1].1.right, rects[2].1.left - GAP);
        assert_eq!(rects[0].1.right, rects[1].1.left - GAP);
    }

    #[test]
    fn the_buttons_show_the_labels_the_caller_passes() {
        let labels = BannerLabels { dismiss: "Tutup", reply: "Balas", open: "Buka" };
        let buttons = BannerButtons::with_reply(&Fonts::new(1.0).unwrap(), labels);
        assert_eq!((buttons.dismiss_label(), buttons.reply_label(), buttons.open_label()), ("Tutup", "Balas", "Buka"));
    }
}
