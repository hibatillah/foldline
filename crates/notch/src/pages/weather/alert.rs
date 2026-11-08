//! The rain alert: a short message in the pill, then a banner that stays while the pointer is on it and then offers
//! Dismiss and Open Weather.

use crate::{
    activity::{FLASH_TIMER, Flash, Presence},
    gfx::{Align, Canvas, Color, FontStyle, Point, Rect, UI_FONT, Weight, clamp_lines},
    page::{Context, PointerEvent, TimerId},
    ui::{self, BANNER_BUTTONS_HEIGHT, BannerButton, BannerButtons, Icon, Stroke, draw_strokes},
};

pub(super) const STAGE_TIMER: TimerId = TimerId(3);
const MESSAGE_MILLISECONDS: u32 = 3000;
const BANNER_MILLISECONDS: u32 = 9000;
/// After the pointer leaves the banner, so a pointer that slips off the edge on the way to a button does not lose it.
const LINGER_MILLISECONDS: u32 = 1500;
const OPEN_LABEL: &str = "Open Weather";

const WIDTH: f32 = 400.0;
const PADDING: f32 = 20.0;
const TOP_PADDING: f32 = 12.0;
const BOTTOM_PADDING: f32 = 16.0;
const TILE_SIZE: f32 = 30.0;
const TILE_RADIUS: f32 = 8.0;
const TILE_GAP: f32 = 12.0;
const TILE_ICON_SIZE: f32 = 18.0;
const HEADING_HEIGHT: f32 = 18.0;
const HEADING_GAP: f32 = 2.0;
const BODY_LINE_HEIGHT: f32 = 17.0;
const BODY_LINES: usize = 2;
const TITLE_GAP: f32 = 8.0;

const HEADING: FontStyle = FontStyle::new(UI_FONT, 14.0, Weight::Medium, Align::Leading);
const APP_NAME: FontStyle = FontStyle::new(UI_FONT, 14.0, Weight::Regular, Align::Leading);
const BODY: FontStyle = FontStyle::new(UI_FONT, 13.0, Weight::Regular, Align::Leading);

const BLUE: Color = Color::rgb(0.455, 0.753, 0.988);
const TILE: Color = Color::rgb(0.11, 0.494, 0.839);
const APP_NAME_COLOR: Color = Color::rgb(0.557, 0.557, 0.6);
const BODY_COLOR: Color = Color::rgb(0.678, 0.678, 0.722);
const RAIN_CLOUD: &str = "M4 14.9A7 7 0 1 1 15.7 8h1.8a4.5 4.5 0 0 1 2.5 8.2";
const RAIN_DROPS: &str = "M16 14v6M8 14v6M12 16v6";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Message,
    Banner,
}

pub(super) struct Alert {
    flash: Flash,
    stage: Stage,
    is_pointer_over: bool,
    title: String,
    time: String,
    message_width: f32,
    lines: Vec<String>,
    buttons: BannerButtons,
}

pub(super) fn banner_height(line_count: usize) -> f32 {
    TOP_PADDING + HEADING_HEIGHT + HEADING_GAP + line_count as f32 * BODY_LINE_HEIGHT + BOTTOM_PADDING
}

/// The right edge of the button row and the top of the gap above it, in device pixels.
fn buttons_origin(area: Rect, scale: f32, line_count: usize) -> (f32, f32) {
    let text_bottom = TOP_PADDING + HEADING_HEIGHT + HEADING_GAP + line_count as f32 * BODY_LINE_HEIGHT;
    (area.right - PADDING * scale, area.top + text_bottom * scale)
}

impl Alert {
    #[cfg(test)]
    pub(super) fn sample_banner(fonts: &crate::gfx::Fonts) -> Self {
        let mut alert = Self::new();
        alert.title = "Rain in 30 min".to_owned();
        alert.time = "17:00".to_owned();
        let body = body_text("Thunderstorms", "17:00", "Pekanbaru, ID");
        alert.lines = clamp_lines(&body, BODY_LINES, Self::text_width(), |text| fonts.measure(&BODY, text));
        alert.buttons = BannerButtons::new(fonts, OPEN_LABEL);
        alert.stage = Stage::Banner;
        alert.flash = Flash::visible();
        alert
    }

    #[cfg(test)]
    pub(super) fn pointed_at(mut self) -> Self {
        self.is_pointer_over = true;
        self
    }

    pub(super) fn new() -> Self {
        Self {
            flash: Flash::default(),
            stage: Stage::Message,
            is_pointer_over: false,
            title: String::new(),
            time: String::new(),
            message_width: 0.0,
            lines: Vec::new(),
            buttons: BannerButtons::default(),
        }
    }

    /// Starts over with the message stage; `body` is cut to two lines for the banner.
    pub(super) fn show(&mut self, context: &mut Context, title: String, time: String, body: &str) {
        let fonts = context.fonts();
        self.message_width = ui::message_width(fonts, &title, Some(&time));
        self.lines = clamp_lines(body, BODY_LINES, Self::text_width(), |text| fonts.measure(&BODY, text));
        self.buttons = BannerButtons::new(fonts, OPEN_LABEL);
        self.title = title;
        self.time = time;
        self.stage = Stage::Message;
        self.flash.show(context, MESSAGE_MILLISECONDS + BANNER_MILLISECONDS);
        context.set_timer(STAGE_TIMER, MESSAGE_MILLISECONDS);
    }

    fn text_width() -> f32 {
        WIDTH - 2.0 * PADDING - TILE_SIZE - TILE_GAP
    }

    pub(super) fn presence(&self) -> Option<Presence> {
        self.flash.is_visible().then(|| match self.stage {
            Stage::Message => Presence::Transient { width: self.message_width },
            Stage::Banner => {
                let buttons = if self.is_pointer_over { BANNER_BUTTONS_HEIGHT } else { 0.0 };
                Presence::Banner { width: WIDTH, height: banner_height(self.lines.len()) + buttons }
            }
        })
    }

    pub(super) fn on_timer(&mut self, timer: TimerId, context: &mut Context) {
        if timer == STAGE_TIMER {
            context.kill_timer(STAGE_TIMER);
            if self.flash.is_visible() {
                self.stage = Stage::Banner;
                self.flash.show(context, BANNER_MILLISECONDS);
            }
        } else if timer == FLASH_TIMER {
            self.is_pointer_over = false;
            self.flash.on_timer(timer, context);
            context.kill_timer(STAGE_TIMER);
        }
    }

    /// Leaving the notch is leaving the banner, also when the notch had opened meanwhile and the banner got no `Left`.
    pub(super) fn on_hover_changed(&mut self, is_hovering: bool, context: &mut Context) {
        if !is_hovering {
            self.pointer_left(context);
        }
    }

    /// The pointer on the banner keeps it up and shows the buttons; leaving lets it go after a moment.
    pub(super) fn on_pointer(&mut self, event: PointerEvent, area: Rect, scale: f32, context: &mut Context) {
        if self.stage != Stage::Banner || !self.flash.is_visible() {
            return;
        }
        match event {
            PointerEvent::Moved(_) | PointerEvent::Pressed(_) if !self.is_pointer_over => {
                self.is_pointer_over = true;
                context.kill_timer(FLASH_TIMER);
                context.request_repaint();
            }
            PointerEvent::Left | PointerEvent::CaptureLost => self.pointer_left(context),
            _ => {}
        }
        let (right, top) = buttons_origin(area, scale, self.lines.len());
        let (clicked, needs_repaint) = self.buttons.on_pointer(event, right, top, scale);
        if needs_repaint {
            context.request_repaint();
        }
        if let Some(button) = clicked {
            self.is_pointer_over = false;
            self.flash.on_timer(FLASH_TIMER, context);
            if button == BannerButton::Open {
                context.request_open();
            }
        }
    }

    fn pointer_left(&mut self, context: &mut Context) {
        if !std::mem::take(&mut self.is_pointer_over) {
            return;
        }
        if self.stage == Stage::Banner && self.flash.is_visible() {
            self.flash.show(context, LINGER_MILLISECONDS);
        }
    }

    pub(super) fn draw(&self, canvas: &Canvas, area: Rect, scale: f32, opacity: f32) {
        match self.stage {
            Stage::Message => ui::draw_message(canvas, area, opacity, Icon::Cloud, BLUE, &self.title, Some(&self.time)),
            Stage::Banner => self.draw_banner(canvas, area, scale, opacity),
        }
    }

    fn draw_banner(&self, canvas: &Canvas, area: Rect, scale: f32, opacity: f32) {
        let tile = Rect::new(
            area.left + PADDING * scale,
            area.top + TOP_PADDING * scale,
            area.left + (PADDING + TILE_SIZE) * scale,
            area.top + (TOP_PADDING + TILE_SIZE) * scale,
        );
        canvas.fill_rounded_rect(tile, TILE_RADIUS * scale, TILE.faded(opacity));
        let layers = [Stroke::new(RAIN_CLOUD, Color::WHITE), Stroke::new(RAIN_DROPS, Color::WHITE)];
        draw_strokes(
            canvas,
            &layers,
            Point::new(tile.center_x(), tile.center_y()),
            TILE_ICON_SIZE * scale,
            2.0,
            opacity,
        );
        let left = tile.right + TILE_GAP * scale;
        let right = area.right - PADDING * scale;
        let heading = Rect::new(left, tile.top, right, tile.top + HEADING_HEIGHT * scale);
        canvas.text(&HEADING, &self.title, heading, Color::WHITE.faded(opacity));
        let title_width = canvas.measure(&HEADING, &self.title) * scale;
        let app_left = left + title_width + TITLE_GAP * scale;
        canvas.text(&APP_NAME, "Weather", Rect { left: app_left, ..heading }, APP_NAME_COLOR.faded(opacity));
        for (index, line) in self.lines.iter().enumerate() {
            let top = heading.bottom + (HEADING_GAP + index as f32 * BODY_LINE_HEIGHT) * scale;
            let rect = Rect::new(left, top, right, top + BODY_LINE_HEIGHT * scale);
            canvas.text(&BODY, line, rect, BODY_COLOR.faded(opacity));
        }
        if self.is_pointer_over {
            let (right, top) = buttons_origin(area, scale, self.lines.len());
            self.buttons.draw(canvas, right, top, scale, opacity);
        }
    }
}

/// What the banner says: the sky, when it comes and where, and a hint.
pub(super) fn body_text(sky: &str, time: &str, place: &str) -> String {
    format!("{sky} around {time} in {place}. Bring an umbrella if you are heading out.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_banner_grows_by_one_body_line_each() {
        assert_eq!(banner_height(2) - banner_height(1), BODY_LINE_HEIGHT);
        assert_eq!(banner_height(2), 82.0);
    }

    #[test]
    fn the_body_names_the_sky_the_time_and_the_place() {
        assert_eq!(
            body_text("Thunderstorms", "17:00", "Pekanbaru, ID"),
            "Thunderstorms around 17:00 in Pekanbaru, ID. Bring an umbrella if you are heading out."
        );
    }

    #[test]
    fn nothing_shows_before_an_alert_arrives() {
        assert_eq!(Alert::new().presence(), None);
    }

    #[test]
    fn the_pointer_on_the_banner_adds_room_for_the_buttons_under_the_same_lines() {
        let fonts = crate::gfx::Fonts::new(1.0).unwrap();
        let Some(Presence::Banner { height: calm, .. }) = Alert::sample_banner(&fonts).presence() else { panic!() };
        let Some(Presence::Banner { height, .. }) = Alert::sample_banner(&fonts).pointed_at().presence() else {
            panic!()
        };
        assert_eq!(height - calm, BANNER_BUTTONS_HEIGHT);
    }

    #[test]
    fn the_buttons_end_inside_the_banner_at_its_padding() {
        let area = Rect::new(0.0, 0.0, WIDTH, banner_height(2) + BANNER_BUTTONS_HEIGHT);
        let (right, top) = buttons_origin(area, 1.0, 2);
        assert_eq!(right, WIDTH - PADDING);
        assert_eq!(top + BANNER_BUTTONS_HEIGHT, area.bottom - BOTTOM_PADDING);
    }
}
