//! The About page's pieces: the pill button and the app card.

use std::rc::Rc;

use gpui::{AnyElement, App, ElementId, Window, div, img, prelude::*, px};
use gpui_component::{ActiveTheme as _, Icon, IconName, Sizable as _, h_flex, tooltip::Tooltip, v_flex};

use crate::{brand, palette, version};

/// What a button does when clicked; a button without one is inert.
pub type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

const TILE_SIZE: f32 = 48.0;
const PILL_HEIGHT: f32 = 32.0;
pub const COMING_SOON: &str = "Coming soon";

/// A dark rounded button with a thin border and an optional icon. Without a handler it is dimmed and "Coming soon"
/// stands to its left.
pub fn pill(id: &str, label: &'static str, icon: Option<IconName>, handler: Option<Handler>, cx: &App) -> AnyElement {
    let theme = cx.theme();
    let inert = handler.is_none();
    let hover_fill = palette::block_fill(cx);
    let button = h_flex()
        .id(ElementId::Name(format!("about-{id}").into()))
        .h(px(PILL_HEIGHT))
        .px_4()
        .gap_2()
        .items_center()
        .rounded(px(10.0))
        .border_1()
        .border_color(palette::input_border(cx))
        .bg(palette::input_background(cx))
        .text_sm()
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(if inert { palette::muted(cx) } else { theme.foreground })
        .children(icon.map(|name| Icon::new(name).small()))
        .child(label);
    match handler {
        Some(handler) => button
            .cursor_pointer()
            .hover(move |style| style.bg(hover_fill))
            .on_click(move |_event, window, cx| handler(window, cx))
            .into_any_element(),
        None => h_flex()
            .gap_3()
            .items_center()
            .child(div().text_sm().text_color(palette::muted(cx)).child(COMING_SOON))
            .child(button.opacity(0.6).tooltip(|window, cx| Tooltip::new(COMING_SOON).build(window, cx)))
            .into_any_element(),
    }
}

/// The app card: the tile, the name and version at the left, the update check at the right.
pub fn app_card(check_now: Option<Handler>, cx: &App) -> AnyElement {
    let theme = cx.theme();
    let check = pill("check-now", "Check now", None, check_now, cx);
    h_flex()
        .w_full()
        .items_center()
        .justify_between()
        .gap_4()
        .child(
            h_flex().gap_4().items_center().child(img(brand::tile_image()).size(px(TILE_SIZE)).flex_none()).child(
                v_flex()
                    .gap_0p5()
                    .child(div().text_base().font_weight(gpui::FontWeight::SEMIBOLD).child("Notch"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(format!("Version {}", version::displayed_version())),
                    ),
            ),
        )
        .child(check)
        .into_any_element()
}
