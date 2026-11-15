//! The window's button and the About page's app card.

use crate::i18n::tr;
use std::rc::Rc;

use gpui::{AnyElement, App, ElementId, Window, div, img, prelude::*, px};
use gpui_component::{ActiveTheme as _, Icon, IconName, h_flex, tooltip::Tooltip, v_flex};

use crate::{brand, palette, version};

/// What a button does when clicked; a button without one is inert.
pub type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

const TILE_SIZE: f32 = 54.0;
const BUTTON_HEIGHT: f32 = 30.0;
const BUTTON_RADIUS: f32 = 8.0;
const BUTTON_PADDING_X: f32 = 12.0;
const BUTTON_GAP: f32 = 7.0;
const BUTTON_TEXT_SIZE: f32 = 13.0;
const BUTTON_ICON_SIZE: f32 = 13.0;
/// An inert button is dimmed this far, with "Coming soon" to its left.
const INERT_OPACITY: f32 = 0.4;
const COMING_SOON_SIZE: f32 = 13.5;
const UPDATES_NOTE_SIZE: f32 = 13.5;
const NAME_SIZE: f32 = 17.0;
const VERSION_SIZE: f32 = 13.5;

fn coming_soon() -> &'static str {
    tr!("about.coming_soon")
}

/// A rounded button a step lighter than its block, with a thin border and an optional icon. Without a handler it is
/// dimmed and "Coming soon" stands to its left.
pub fn pill(id: &str, label: &'static str, icon: Option<IconName>, handler: Option<Handler>, cx: &App) -> AnyElement {
    let theme = cx.theme();
    let hover_fill = palette::button_border(cx);
    let button = h_flex()
        .id(ElementId::Name(format!("about-{id}").into()))
        .flex_none()
        .h(px(BUTTON_HEIGHT))
        .px(px(BUTTON_PADDING_X))
        .gap(px(BUTTON_GAP))
        .items_center()
        .rounded(px(BUTTON_RADIUS))
        .border_1()
        .border_color(palette::button_border(cx))
        .bg(palette::button_fill(cx))
        .text_size(px(BUTTON_TEXT_SIZE))
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(theme.foreground)
        .whitespace_nowrap()
        .children(icon.map(|name| Icon::new(name).size(px(BUTTON_ICON_SIZE))))
        .child(label);
    match handler {
        Some(handler) => button
            .cursor_pointer()
            .hover(move |style| style.bg(hover_fill))
            .on_click(move |_event, window, cx| handler(window, cx))
            .into_any_element(),
        None => h_flex()
            .flex_none()
            .gap_3()
            .items_center()
            .child(div().text_size(px(COMING_SOON_SIZE)).text_color(palette::muted(cx)).child(coming_soon()))
            .child(button.opacity(INERT_OPACITY).tooltip(|window, cx| Tooltip::new(coming_soon()).build(window, cx)))
            .into_any_element(),
    }
}

/// The app card: the tile, the name and version at the left, where updates come from at the right. The store link is
/// left out while it has no handler.
pub fn app_card(open_store: Option<Handler>, cx: &App) -> AnyElement {
    let theme = cx.theme();
    let store_link = open_store
        .map(|handler| pill("open-store", tr!("about.open_in_store"), Some(IconName::ExternalLink), Some(handler), cx));
    let updates = h_flex()
        .flex_none()
        .gap_3()
        .items_center()
        .child(
            div()
                .text_size(px(UPDATES_NOTE_SIZE))
                .text_color(palette::muted(cx))
                .child(tr!("about.updates_from_store")),
        )
        .children(store_link);
    h_flex()
        .w_full()
        .items_center()
        .justify_between()
        .gap(px(14.0))
        .child(
            h_flex()
                .gap(px(14.0))
                .items_center()
                .child(img(brand::tile_image()).size(px(TILE_SIZE)).flex_none())
                .child(
                    v_flex()
                        .child(div().text_size(px(NAME_SIZE)).font_weight(gpui::FontWeight::SEMIBOLD).child("Notch"))
                        .child(
                            div()
                                .text_size(px(VERSION_SIZE))
                                .text_color(theme.muted_foreground)
                                .child(tr!("about.version", version = version::displayed_version())),
                        ),
                ),
        )
        .child(updates)
        .into_any_element()
}
