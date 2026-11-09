use super::{
    layout::{self, MESSAGE_BADGE_DIAMETER, MESSAGE_ICON_SIZE},
    model::{Device, Prepared, Tone},
    ring::{self, Fill},
};
use crate::ui::palette::{LIGHT_GRAY, MINT_GREEN, SURFACE_BADGE, SURFACE_BORDER, TEXT_DIM, TEXT_MUTED};
use crate::ui::type_styles::{MEDIUM_15, REGULAR_10_CENTER, REGULAR_12};
use crate::{
    gfx::{Align, Canvas, Color, FontStyle, Point, Rect, UI_FONT, Weight},
    ui::Icon,
};

const ICON_SIZE: f32 = 22.0;
const BOLT_SIZE: f32 = 10.0;
const BOLT_WIDTH: f32 = 8.0;
const BOLT_GAP: f32 = 1.0;

const PERCENT: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Medium, Align::Center).tabular();
const PERCENT_LEADING: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Medium, Align::Leading).tabular();
const IDLE_GREEN: Color = Color::rgb(0.184, 0.620, 0.267);
const CHARGING_GREEN: Color = Color::rgb(0.188, 0.820, 0.345);
const LOW_RED: Color = Color::rgb(1.0, 0.271, 0.227);
const OFF_ICON: Color = Color::rgb(0.333, 0.333, 0.373);
const BLUETOOTH_OFF: Color = Color::rgb(0.541, 0.541, 0.588);

pub(super) fn tone_color(tone: Tone) -> Color {
    match tone {
        Tone::Idle => IDLE_GREEN,
        Tone::Low => LOW_RED,
        Tone::Charging => CHARGING_GREEN,
    }
}

pub(super) fn draw_tiles(canvas: &Canvas, prepared: &Prepared, bounds: Rect, opacity: f32) {
    let shown = &prepared.shown;
    let count = shown.devices.len();
    for (index, (device, percent_text)) in shown.devices.iter().zip(&prepared.percent_texts).enumerate() {
        draw_tile(canvas, device, percent_text, layout::cell(bounds, index, count, canvas.scale()), opacity);
    }
}

fn draw_tile(canvas: &Canvas, device: &Device, percent_text: &str, cell: Rect, opacity: f32) {
    let scale = canvas.scale();
    let tone = device.tone();
    let fill = match device.ring_fraction() {
        Some(fraction) => Fill::Level { fraction, color: tone_color(tone) },
        None if device.is_connected => Fill::Solid,
        None => Fill::Dashed,
    };
    ring::draw(canvas, layout::ring_center(cell, scale), &fill, opacity);
    let icon_color = if device.is_connected { LIGHT_GRAY } else { OFF_ICON };
    device.kind.icon().draw(canvas, layout::icon_center(cell, scale), ICON_SIZE * scale, icon_color, opacity);
    draw_percent(canvas, device, percent_text, cell, opacity);
    canvas.text(&REGULAR_10_CENTER, &device.name, layout::name_line(cell, scale), TEXT_MUTED.faded(opacity));
}

fn draw_percent(canvas: &Canvas, device: &Device, text: &str, cell: Rect, opacity: f32) {
    let scale = canvas.scale();
    let line = layout::percent_line(cell, scale);
    let color = match (device.is_connected, device.battery_percent) {
        (false, _) => TEXT_DIM,
        (true, None) => Color::WHITE,
        (true, Some(_)) if device.tone() == Tone::Low => LOW_RED,
        (true, Some(_)) => Color::WHITE,
    };
    if device.tone() != Tone::Charging || !device.is_connected {
        canvas.text(&PERCENT, text, line, color.faded(opacity));
        return;
    }
    // The bolt follows the number, and the pair is centered as one.
    let text_width = canvas.measure(&PERCENT_LEADING, text) * scale;
    let total = text_width + (BOLT_GAP + BOLT_WIDTH) * scale;
    let start = cell.center_x() - total / 2.0;
    canvas.text(
        &PERCENT_LEADING,
        text,
        Rect { left: start, right: start + text_width + scale, ..line },
        color.faded(opacity),
    );
    let bolt = Point::new(start + text_width + (BOLT_GAP + BOLT_WIDTH / 2.0) * scale, line.top + line.height() / 2.0);
    Icon::Bolt.draw(canvas, bolt, BOLT_SIZE * scale, CHARGING_GREEN, opacity);
}

/// The whole-page message: a Bluetooth mark in a round badge at the left, a title and one line of detail beside it.
pub(super) fn draw_message(
    canvas: &Canvas,
    bounds: Rect,
    is_bluetooth_off: bool,
    title: &str,
    detail: &str,
    opacity: f32,
) {
    let scale = canvas.scale();
    let text_width = canvas.measure(&MEDIUM_15, title).max(canvas.measure(&REGULAR_12, detail)).ceil();
    let (icon, title_line, detail_line) = layout::message_lines(bounds, scale, text_width);
    let radius = MESSAGE_BADGE_DIAMETER / 2.0 * scale;
    canvas.fill_circle(icon, radius, SURFACE_BORDER.faded(opacity));
    canvas.fill_circle(icon, radius - scale, SURFACE_BADGE.faded(opacity));
    let tint = if is_bluetooth_off { BLUETOOTH_OFF } else { MINT_GREEN };
    Icon::Bluetooth.draw(canvas, icon, MESSAGE_ICON_SIZE * scale, tint, opacity);
    canvas.text(&MEDIUM_15, title, title_line, Color::WHITE.faded(opacity));
    canvas.text(&REGULAR_12, detail, detail_line, TEXT_MUTED.faded(opacity));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::Fonts;

    #[test]
    fn a_long_receiver_name_is_cut_with_an_ellipsis_inside_the_name_width() {
        let fonts = Fonts::new(1.0).unwrap();
        // The text style does not wrap, so DirectWrite trims what passes the line's right edge with an ellipsis.
        const { assert!(!REGULAR_10_CENTER.wrap) };
        let long_name = "Logitech Wireless Mouse M330 Silent Plus";
        assert!(fonts.measure(&REGULAR_10_CENTER, long_name) > 76.0, "the sample really is wider than the tile's name");
        for scale in [1.0, 1.5, 2.0] {
            let cell = layout::cell(Rect::new(0.0, 0.0, 440.0 * scale, 86.0 * scale), 0, 5, scale);
            assert!(layout::name_line(cell, scale).width() <= 76.0 * scale + 0.01);
        }
    }
}
