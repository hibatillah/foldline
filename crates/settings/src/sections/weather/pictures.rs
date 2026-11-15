//! The Weather page's drawings: the hourly rain bars with the moment the message is sent, and the pill that shows it.

use gpui::{AnyElement, App, SharedString, Window, div, prelude::*, px, relative, rgba};
use gpui_component::{ActiveTheme as _, Icon, h_flex, v_flex};

use crate::{i18n::tr, icons, illustration, picture::color};

const SCENE_HEIGHT: f32 = 78.0;
const BAR_AREA_HEIGHT: f32 = 36.0;
const LABEL_GRAY: u32 = 0x9a9aa6;
/// Rain strength in twelve steps of five minutes; the alert fires where the seventh begins, 30 minutes out.
const RAIN_STEPS: [f32; 12] = [0.12, 0.16, 0.22, 0.3, 0.42, 0.7, 0.9, 1.0, 0.8, 0.55, 0.38, 0.26];
const ALERT_STEP: usize = 6;
const PILL_TIME: u32 = 0x4c9aff;

/// The step of the bars at which the message is sent, as a fraction of the chart's width.
fn alert_fraction() -> f32 {
    ALERT_STEP as f32 / RAIN_STEPS.len() as f32
}

fn small_label(text: &'static str) -> gpui::Div {
    div().text_size(px(9.5)).text_color(color(LABEL_GRAY)).whitespace_nowrap().child(text)
}

fn rain_bars(cx: &App) -> AnyElement {
    let accent = crate::palette::accent(cx);
    let bars = RAIN_STEPS.iter().enumerate().map(|(step, strength)| {
        let before_alert = step < ALERT_STEP;
        div().flex_1().h(px(BAR_AREA_HEIGHT * strength)).rounded_t(px(2.0)).bg(if before_alert {
            rgba(0x4c9aff40).into()
        } else {
            accent
        })
    });
    let chart = div()
        .absolute()
        .left(px(12.0))
        .right(px(12.0))
        .top(px(22.0))
        .h(px(BAR_AREA_HEIGHT))
        .child(h_flex().size_full().items_end().gap(px(4.0)).children(bars))
        .child(
            div().absolute().top(px(-8.0)).bottom_0().left(relative(alert_fraction())).w(px(1.0)).bg(rgba(0xffffff80)),
        );
    let sent = div()
        .absolute()
        .top(px(5.0))
        .left(relative(alert_fraction()))
        .ml(px(-2.0))
        .px(px(5.0))
        .py(px(1.0))
        .rounded(px(4.0))
        .bg(rgba(0xffffff26))
        .text_size(px(9.5))
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(color(0xffffff))
        .whitespace_nowrap()
        .child(tr!("weather.alert_sent"));
    let labels = div()
        .absolute()
        .left(px(12.0))
        .right(px(12.0))
        .bottom(px(3.0))
        .h(px(12.0))
        .child(div().absolute().left_0().child(small_label(tr!("weather.axis_now"))))
        .child(div().absolute().left(relative(alert_fraction())).child(small_label(tr!("weather.axis_half_hour"))))
        .child(div().absolute().right_0().child(small_label(tr!("weather.axis_hour"))));
    illustration::scene(SCENE_HEIGHT).child(chart).child(sent).child(labels).into_any_element()
}

/// "Rain in 30 min" and the time it starts, in the pill over a drawn window.
fn rain_pill() -> AnyElement {
    let pill = h_flex()
        .gap(px(7.0))
        .items_center()
        .child(Icon::default().path(icons::WEATHER).size(px(12.0)).text_color(color(0xffffff)))
        .child(div().text_size(px(12.0)).child(SharedString::from(tr!("weather.alert_pill", minutes = 30))))
        .child(div().text_size(px(12.0)).text_color(color(PILL_TIME)).child("17:00"));
    illustration::scene(SCENE_HEIGHT)
        .child(illustration::notch(pill, 26.0, 12.0, 14.0))
        .child(illustration::standing_window(44.0))
        .into_any_element()
}

pub fn rain_alert(_window: &mut Window, cx: &mut App) -> AnyElement {
    let muted = cx.theme().muted_foreground;
    h_flex()
        .w_full()
        .items_start()
        .gap(px(12.0))
        .child(
            v_flex()
                .flex_grow()
                .flex_basis(relative(0.0))
                .min_w_0()
                .gap(px(6.0))
                .child(rain_bars(cx))
                .child(illustration::caption(tr!("weather.alert_caption_bars"), muted)),
        )
        .child(
            v_flex()
                .flex_grow()
                .flex_basis(relative(0.0))
                .min_w_0()
                .gap(px(6.0))
                .child(rain_pill())
                .child(illustration::caption(tr!("weather.alert_caption_pill"), muted)),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_alert_marker_sits_at_half_an_hour_of_the_hour_shown() {
        let minutes = alert_fraction() * RAIN_STEPS.len() as f32 * 5.0;
        assert!((minutes - 30.0).abs() < 0.001, "{minutes}");
    }

    #[test]
    fn the_rain_alert_wording_is_in_both_languages() {
        use store::language::{Language, with_override};
        assert_eq!(tr!("weather.alert_pill", minutes = 30), "Rain in 30 min");
        assert_eq!(
            with_override(Language::Indonesian, || tr!("weather.alert_pill", minutes = 30)),
            "Hujan 30 mnt lagi"
        );
    }
}
