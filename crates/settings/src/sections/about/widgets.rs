//! The window's button and the About page's app card.

use crate::i18n::tr;
use std::rc::Rc;

use gpui::{AnyElement, App, ElementId, Window, div, img, prelude::*, px};
use gpui_component::{ActiveTheme as _, Icon, IconName, h_flex, v_flex};

use crate::{brand, palette, version};

/// What a button does when clicked.
pub type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

const TILE_SIZE: f32 = 54.0;
const BUTTON_HEIGHT: f32 = 30.0;
const BUTTON_RADIUS: f32 = 8.0;
const BUTTON_PADDING_X: f32 = 12.0;
const BUTTON_GAP: f32 = 7.0;
const BUTTON_TEXT_SIZE: f32 = 13.0;
const BUTTON_ICON_SIZE: f32 = 13.0;
const UPDATES_NOTE_SIZE: f32 = 13.5;
const NAME_SIZE: f32 = 17.0;
const VERSION_SIZE: f32 = 13.5;

/// A handler that opens an address in the default browser.
pub fn open_url_handler(url: &'static str) -> Handler {
    Rc::new(move |_window, cx| cx.open_url(url))
}

/// A rounded button a step lighter than its block, with a thin border and an optional icon.
pub fn pill(id: &str, label: &'static str, icon: Option<IconName>, handler: Handler, cx: &App) -> AnyElement {
    let theme = cx.theme();
    let hover_fill = palette::button_border(cx);
    h_flex()
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
        .child(label)
        .cursor_pointer()
        .hover(move |style| style.bg(hover_fill))
        .on_click(move |_event, window, cx| handler(window, cx))
        .into_any_element()
}

/// The app card: the tile, the name and version at the left, where updates come from at the right. The store link is
/// left out while it has no handler.
pub fn app_card(open_store: Option<Handler>, cx: &App) -> AnyElement {
    let theme = cx.theme();
    let store_link = open_store
        .map(|handler| pill("open-store", tr!("about.open_in_store"), Some(IconName::ExternalLink), handler, cx));
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
                        .child(div().text_size(px(NAME_SIZE)).font_weight(gpui::FontWeight::SEMIBOLD).child("Foldline"))
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
