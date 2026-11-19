//! The three pictures under Startup: the notch waiting in the tray, the pill hidden while idle, and the pill back when
//! the pointer reaches the top edge.

use gpui::{AnyElement, App, SharedString, div, prelude::*, px, rgba};
use gpui_component::ActiveTheme as _;

use crate::{i18n::tr, illustration, layout::Item, picture::color};

const PICTURE_HEIGHT: f32 = 76.0;
const TASKBAR_HEIGHT: f32 = 18.0;
const TRAY_ICON: f32 = 7.0;
const TRAY_FOLDLINE_ICON: f32 = 9.0;
/// The outline that marks where the hidden pill would be.
const HIDDEN_PILL_WIDTH: f32 = 62.0;
const HIDDEN_PILL_HEIGHT: f32 = 20.0;
const PILL_HEIGHT: f32 = 24.0;
const PILL_TEXT_SIZE: f32 = 11.5;

/// A desktop window and the taskbar, with the notch's blue icon last in the tray.
fn in_the_tray(cx: &App) -> AnyElement {
    let accent = crate::palette::accent(cx);
    let icon = |size: f32, fill: gpui::Hsla| div().size(px(size)).rounded(px(2.0)).bg(fill);
    illustration::scene(PICTURE_HEIGHT)
        .child(
            div()
                .absolute()
                .left(px(14.0))
                .right(px(14.0))
                .top(px(10.0))
                .h(px(48.0))
                .rounded(px(6.0))
                .bg(rgba(0xffffff0f))
                .child(illustration::text_bar(10.0, 10.0, 80.0, true))
                .child(illustration::text_bar(10.0, 20.0, 120.0, false))
                .child(illustration::text_bar(10.0, 30.0, 60.0, false)),
        )
        .child(
            div()
                .absolute()
                .left_0()
                .right_0()
                .bottom_0()
                .h(px(TASKBAR_HEIGHT))
                .bg(rgba(0x0000008c))
                .flex()
                .justify_end()
                .items_center()
                .gap(px(5.0))
                .pr(px(10.0))
                .child(icon(TRAY_ICON, color(0x777777)))
                .child(icon(TRAY_ICON, color(0x777777)))
                .child(icon(TRAY_FOLDLINE_ICON, accent)),
        )
        .into_any_element()
}

/// A window in front and a dashed outline where the pill would be.
fn hidden_while_idle() -> AnyElement {
    illustration::scene(PICTURE_HEIGHT)
        .child(illustration::standing_window(30.0))
        .child(
            div().absolute().top_0().left_0().w_full().flex().justify_center().child(
                div()
                    .w(px(HIDDEN_PILL_WIDTH))
                    .h(px(HIDDEN_PILL_HEIGHT))
                    .border_1()
                    .border_t_0()
                    .border_dashed()
                    .border_color(rgba(0xffffff4d))
                    .rounded_b(px(8.0)),
            ),
        )
        .into_any_element()
}

/// The pill back at the top with the clock, the pointer just under it.
fn back_at_the_top_edge() -> AnyElement {
    let clock = div().text_size(px(PILL_TEXT_SIZE)).child("15:05");
    let pointer =
        div().absolute().top(px(10.0)).left(gpui::relative(0.6)).child(
            gpui_component::Icon::default().path(crate::icons::POINTER).size(px(12.0)).text_color(color(0xffffff)),
        );
    illustration::scene(PICTURE_HEIGHT)
        .child(illustration::standing_window(30.0))
        .child(illustration::notch(clock, PILL_HEIGHT, 10.0, 14.0))
        .child(pointer)
        .into_any_element()
}

/// The strip in its own row of the Startup block.
pub fn item() -> Item {
    Item::picture_row(|_window, cx| {
        let pictures = vec![
            (in_the_tray(cx), SharedString::from(tr!("general.idle_picture_tray"))),
            (hidden_while_idle(), SharedString::from(tr!("general.idle_picture_hidden"))),
            (back_at_the_top_edge(), SharedString::from(tr!("general.idle_picture_back"))),
        ];
        illustration::strip(pictures, cx.theme().muted_foreground)
    })
}
