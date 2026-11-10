//! The rain alert: a 340 DIP banner for a few seconds (Dismiss and Open while the pointer is on it, which shows the weather page), then the
//! countdown badge in the pill's group until the rain starts.

use crate::i18n::tr;
use salah::{DateTime, Utc};

use super::badge::{self, Badge};
use crate::ui::palette::{CARD_APP_NAME_TEXT, CARD_BODY_TEXT};
use crate::ui::type_styles::{MEDIUM_14, REGULAR_13, REGULAR_14};
use crate::{
    activity::{FLASH_TIMER, Flash, Presence},
    gfx::{Canvas, Color, Point, Rect, clamp_lines},
    page::{Context, PointerEvent, TimerId},
    ui::{BANNER_BUTTONS_HEIGHT, BannerButton, BannerButtons, BannerLabels, Stroke, draw_strokes},
};

pub(super) const BADGE_TIMER: TimerId = TimerId(3);
const BANNER_MILLISECONDS: u32 = 9000;
/// After the pointer leaves the banner, so a pointer that slips off the edge on the way to a button does not lose it.
const LINGER_MILLISECONDS: u32 = 1500;

/// The width of a notification card.
pub(super) const WIDTH: f32 = 340.0;
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

const TILE: Color = Color::rgb(0.11, 0.494, 0.839);
const RAIN_CLOUD: &str = "M4 14.9A7 7 0 1 1 15.7 8h1.8a4.5 4.5 0 0 1 2.5 8.2";
const RAIN_DROPS: &str = "M16 14v6M8 14v6M12 16v6";

/// What the alert shows for the coming rain.
pub(super) struct Rain {
    pub(super) at: DateTime<Utc>,
    pub(super) title: String,
    pub(super) body: String,
}

pub(super) struct Alert {
    flash: Flash,
    is_pointer_over: bool,
    title: String,
    lines: Vec<String>,
    buttons: BannerButtons,
    badge: Option<Badge>,
}

pub(super) fn banner_height(line_count: usize) -> f32 {
    TOP_PADDING + HEADING_HEIGHT + HEADING_GAP + line_count as f32 * BODY_LINE_HEIGHT + BOTTOM_PADDING
}

/// The right edge of the button row and the top of the gap above it, in units.
fn buttons_origin(area: Rect, line_count: usize) -> (f32, f32) {
    let text_bottom = TOP_PADDING + HEADING_HEIGHT + HEADING_GAP + line_count as f32 * BODY_LINE_HEIGHT;
    (area.right - PADDING, area.top + text_bottom)
}

fn text_width() -> f32 {
    WIDTH - 2.0 * PADDING - TILE_SIZE - TILE_GAP
}

/// What the banner says: the sky, when it comes and where, and a hint.
pub(super) fn body_text(sky: &str, time: &str, place: &str) -> String {
    tr!("weather.alert_body", sky = sky, time = time, place = place).into_owned()
}

fn translated_buttons(fonts: &crate::gfx::Fonts) -> BannerButtons {
    let (dismiss, reply, open) = (tr!("banner.dismiss"), tr!("banner.reply"), tr!("banner.open"));
    BannerButtons::new(fonts, BannerLabels { dismiss: &dismiss, reply: &reply, open: &open })
}

impl Alert {
    pub(super) fn new() -> Self {
        Self {
            flash: Flash::default(),
            is_pointer_over: false,
            title: String::new(),
            lines: Vec::new(),
            buttons: BannerButtons::default(),
            badge: None,
        }
    }

    #[cfg(test)]
    pub(super) fn sample_banner(fonts: &crate::gfx::Fonts, rain_at: DateTime<Utc>) -> Self {
        let mut alert = Self::new();
        alert.title = "Rain in 30 min".to_owned();
        let body = body_text("Thunderstorms", "17:00", "Pekanbaru, ID");
        alert.lines = clamp_lines(&body, BODY_LINES, text_width(), |text| fonts.measure(&REGULAR_13, text));
        alert.buttons = translated_buttons(fonts);
        alert.badge = Some(Badge::new(fonts, rain_at, super::RAIN_ALERT_LEAD_MINUTES));
        alert.flash = Flash::visible();
        alert
    }

    /// The state after the banner went away and only the badge is left.
    #[cfg(test)]
    pub(super) fn sample_badge(fonts: &crate::gfx::Fonts, rain_at: DateTime<Utc>) -> Self {
        let mut alert = Self::sample_banner(fonts, rain_at);
        alert.flash = Flash::default();
        alert
    }

    #[cfg(test)]
    pub(super) fn pointed_at(mut self) -> Self {
        self.is_pointer_over = true;
        self
    }

    pub(super) fn remeasure_badge(&mut self, fonts: &crate::gfx::Fonts) {
        self.badge = self.badge.as_ref().map(|badge| badge.remeasured(fonts));
    }

    /// Starts over with the banner, cut to two lines; the badge counts down to the rain.
    pub(super) fn show(&mut self, context: &mut Context, rain: Rain, now: DateTime<Utc>) {
        let fonts = context.fonts();
        self.lines = clamp_lines(&rain.body, BODY_LINES, text_width(), |text| fonts.measure(&REGULAR_13, text));
        self.buttons = translated_buttons(fonts);
        self.badge = Some(Badge::new(fonts, rain.at, super::RAIN_ALERT_LEAD_MINUTES));
        self.title = rain.title;
        self.is_pointer_over = false;
        self.flash.show(context, BANNER_MILLISECONDS);
        context.set_timer(BADGE_TIMER, badge::next_tick_milliseconds(rain.at, now));
    }

    /// Takes the banner and the badge away.
    pub(super) fn clear(&mut self, context: &mut Context) {
        let was_showing = self.flash.is_visible() || self.badge.is_some();
        self.is_pointer_over = false;
        self.badge = None;
        self.flash.on_timer(FLASH_TIMER, context);
        context.kill_timer(BADGE_TIMER);
        if was_showing {
            context.request_repaint();
        }
    }

    fn live_badge(&self, now: DateTime<Utc>) -> Option<&Badge> {
        self.badge.as_ref().filter(|badge| !badge.is_over(now))
    }

    pub(super) fn presence(&self, now: DateTime<Utc>) -> Option<Presence> {
        if self.flash.is_visible() {
            let buttons = if self.is_pointer_over { BANNER_BUTTONS_HEIGHT } else { 0.0 };
            return Some(Presence::Banner { width: WIDTH, height: banner_height(self.lines.len()) + buttons });
        }
        self.live_badge(now).map(|badge| Presence::Badge { width: badge.width })
    }

    pub(super) fn has_badge(&self, now: DateTime<Utc>) -> bool {
        self.live_badge(now).is_some()
    }

    pub(super) fn short_badge_width(&self, now: DateTime<Utc>) -> Option<f32> {
        self.live_badge(now).map(|badge| badge.short_width)
    }

    pub(super) fn countdown_seconds(&self, now: DateTime<Utc>) -> Option<u64> {
        self.live_badge(now).map(|badge| (badge.rain_at - now).num_seconds().max(0) as u64)
    }

    pub(super) fn on_timer(&mut self, timer: TimerId, now: DateTime<Utc>, context: &mut Context) {
        if timer == BADGE_TIMER {
            match self.live_badge(now).map(|badge| badge.rain_at) {
                Some(rain_at) => context.set_timer(BADGE_TIMER, badge::next_tick_milliseconds(rain_at, now)),
                None => {
                    self.badge = None;
                    context.kill_timer(BADGE_TIMER);
                }
            }
            context.request_repaint();
        } else if timer == FLASH_TIMER {
            self.is_pointer_over = false;
            self.flash.on_timer(timer, context);
        }
    }

    /// Leaving the notch is leaving the banner, also when the notch had opened meanwhile and the banner got no `Left`.
    pub(super) fn on_hover_changed(&mut self, is_hovering: bool, context: &mut Context) {
        if !is_hovering {
            self.pointer_left(context);
        }
    }

    /// A press on the badge, once the banner is gone, opens the notch on the weather page.
    fn press_opens(&self, event: PointerEvent, now: DateTime<Utc>) -> bool {
        !self.flash.is_visible() && matches!(event, PointerEvent::Pressed(_)) && self.live_badge(now).is_some()
    }

    /// The pointer on the banner keeps it up and shows the buttons; leaving lets it go after a moment.
    pub(super) fn on_pointer(&mut self, event: PointerEvent, area: Rect, now: DateTime<Utc>, context: &mut Context) {
        if self.press_opens(event, now) {
            context.request_open();
        }
        if !self.flash.is_visible() {
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
        let (right, top) = buttons_origin(area, self.lines.len());
        let (clicked, needs_repaint) = self.buttons.on_pointer(event, right, top, 1.0);
        if needs_repaint {
            context.request_repaint();
        }
        match clicked {
            Some(BannerButton::Dismiss) => self.clear(context),
            Some(BannerButton::Open) => {
                context.request_open();
                self.is_pointer_over = false;
                self.flash.on_timer(FLASH_TIMER, context);
            }
            // This banner has no Reply button.
            Some(BannerButton::Reply) | None => {}
        }
    }

    fn pointer_left(&mut self, context: &mut Context) {
        if std::mem::take(&mut self.is_pointer_over) && self.flash.is_visible() {
            self.flash.show(context, LINGER_MILLISECONDS);
        }
    }

    pub(super) fn draw(&self, canvas: &Canvas, area: Rect, opacity: f32, now: DateTime<Utc>) {
        if self.flash.is_visible() {
            self.draw_banner(canvas, area, opacity);
        } else if let Some(badge) = self.live_badge(now) {
            badge.draw(canvas, area, opacity, now, false);
        }
    }

    pub(super) fn draw_short_badge(&self, canvas: &Canvas, area: Rect, opacity: f32, now: DateTime<Utc>) {
        if let Some(badge) = self.live_badge(now) {
            badge.draw(canvas, area, opacity, now, true);
        }
    }

    fn draw_banner(&self, canvas: &Canvas, area: Rect, opacity: f32) {
        let tile = Rect::new(
            area.left + PADDING,
            area.top + TOP_PADDING,
            area.left + PADDING + TILE_SIZE,
            area.top + TOP_PADDING + TILE_SIZE,
        );
        canvas.fill_rounded_rect(tile, TILE_RADIUS, TILE.faded(opacity));
        let layers = [Stroke::new(RAIN_CLOUD, Color::WHITE), Stroke::new(RAIN_DROPS, Color::WHITE)];
        draw_strokes(canvas, &layers, Point::new(tile.center_x(), tile.center_y()), TILE_ICON_SIZE, 2.0, opacity);
        let left = tile.right + TILE_GAP;
        let right = area.right - PADDING;
        let heading = Rect::new(left, tile.top, right, tile.top + HEADING_HEIGHT);
        canvas.text(&MEDIUM_14, &self.title, heading, Color::WHITE.faded(opacity));
        let title_width = canvas.measure(&MEDIUM_14, &self.title);
        let app_left = left + title_width + TITLE_GAP;
        canvas.text(&REGULAR_14, "Weather", Rect { left: app_left, ..heading }, CARD_APP_NAME_TEXT.faded(opacity));
        for (index, line) in self.lines.iter().enumerate() {
            let top = heading.bottom + HEADING_GAP + index as f32 * BODY_LINE_HEIGHT;
            let rect = Rect::new(left, top, right, top + BODY_LINE_HEIGHT);
            canvas.text(&REGULAR_13, line, rect, CARD_BODY_TEXT.faded(opacity));
        }
        if self.is_pointer_over {
            let (right, top) = buttons_origin(area, self.lines.len());
            self.buttons.draw(canvas, right, top, 1.0, opacity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::Fonts;

    fn rain_at() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-11-05T10:00:00Z").unwrap().with_timezone(&Utc)
    }

    fn before(minutes: i64) -> DateTime<Utc> {
        rain_at() - salah::Duration::minutes(minutes)
    }

    #[test]
    fn the_banner_grows_by_one_body_line_each() {
        assert_eq!(banner_height(2) - banner_height(1), BODY_LINE_HEIGHT);
        assert_eq!(banner_height(2), 82.0);
    }

    #[test]
    fn the_body_names_the_sky_the_time_and_the_place() {
        assert_eq!(
            body_text("Thunderstorms", "17:00", "Pekanbaru, ID"),
            "Thunderstorms around 17:00 in Pekanbaru, ID. Bring an umbrella."
        );
    }

    #[test]
    fn nothing_shows_before_an_alert_arrives() {
        assert_eq!(Alert::new().presence(before(30)), None);
    }

    #[test]
    fn the_sample_body_fits_the_narrower_banner_without_an_ellipsis() {
        let fonts = Fonts::new(1.0).unwrap();
        let body = body_text("Thunderstorms", "17:00", "Pekanbaru, ID");
        let lines = clamp_lines(&body, BODY_LINES, text_width(), |text| fonts.measure(&REGULAR_13, text));
        assert!(lines.iter().all(|line| !line.ends_with('\u{2026}')), "{lines:?}");
    }

    #[test]
    fn the_pointer_on_the_banner_adds_room_for_the_buttons_under_the_same_lines() {
        let fonts = Fonts::new(1.0).unwrap();
        let calm = Alert::sample_banner(&fonts, rain_at()).presence(before(30));
        let pointed = Alert::sample_banner(&fonts, rain_at()).pointed_at().presence(before(30));
        let (Some(Presence::Banner { width, height: calm }), Some(Presence::Banner { height, .. })) = (calm, pointed)
        else {
            panic!("both are banners")
        };
        assert_eq!(width, 340.0);
        assert_eq!(height - calm, BANNER_BUTTONS_HEIGHT);
    }

    #[test]
    fn after_the_banner_the_countdown_stays_as_a_badge_until_the_rain_starts() {
        let fonts = Fonts::new(1.0).unwrap();
        let alert = Alert::sample_badge(&fonts, rain_at());
        assert!(matches!(alert.presence(before(25)), Some(Presence::Badge { .. })));
        assert_eq!(alert.countdown_seconds(before(25)), Some(25 * 60));
        assert_eq!(alert.presence(rain_at()), None);
        assert_eq!(alert.countdown_seconds(rain_at()), None);
    }

    #[test]
    fn a_press_on_the_badge_opens_the_notch_but_a_move_or_the_banner_does_not() {
        let fonts = Fonts::new(1.0).unwrap();
        let press = PointerEvent::Pressed(Point::new(1.0, 1.0));
        let badge = Alert::sample_badge(&fonts, rain_at());
        assert!(badge.press_opens(press, before(25)));
        assert!(!badge.press_opens(PointerEvent::Moved(Point::new(1.0, 1.0)), before(25)));
        assert!(!badge.press_opens(press, rain_at()), "the badge is gone once the rain starts");
        let banner = Alert::sample_banner(&fonts, rain_at());
        assert!(!banner.press_opens(press, before(25)), "the banner has its own buttons");
        assert!(!Alert::new().press_opens(press, before(25)));
    }

    #[test]
    fn the_buttons_end_inside_the_banner_at_its_padding() {
        let area = Rect::new(0.0, 0.0, WIDTH, banner_height(2) + BANNER_BUTTONS_HEIGHT);
        let (right, top) = buttons_origin(area, 2);
        assert_eq!(right, WIDTH - PADDING);
        assert_eq!(top + BANNER_BUTTONS_HEIGHT, area.bottom - BOTTOM_PADDING);
    }

    #[test]
    fn the_alert_body_reads_in_indonesian() {
        store::language::with_override(store::language::Language::Indonesian, || {
            assert_eq!(
                body_text("Badai petir", "17:00", "Pekanbaru, ID"),
                "Badai petir sekitar pukul 17:00 di Pekanbaru, ID. Bawa payung."
            );
        });
    }

    #[test]
    fn a_language_change_measures_the_rain_badge_again() {
        let fonts = crate::gfx::Fonts::new(1.0).unwrap();
        let mut alert = Alert::sample_banner(&fonts, rain_at());
        let english = alert.badge.as_ref().unwrap().width;
        store::language::with_override(store::language::Language::Indonesian, || {
            alert.remeasure_badge(&fonts);
            assert!(alert.badge.as_ref().unwrap().width > english);
        });
    }
}
