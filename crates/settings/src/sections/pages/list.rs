//! The page list: one numbered row per page with its drag handle, icon, name, description and switch. Drag a row to
//! reorder it; switch it to turn the page on or off.

use gpui::{AnyElement, App, Context, IntoElement, Render, SharedString, Window, div, prelude::*, px};
use gpui_component::{ActiveTheme as _, Icon, h_flex, v_flex};
use store::Settings;

use super::order::{
    DISABLED_KEY, KNOWN_KEY, KeyHint, ORDER_KEY, current_order, disabled_pages, order_text, parse_known, reorder,
    switch_page,
};
use crate::{
    field,
    i18n::tr,
    icons,
    sections::apply,
    tag::{Tone, tag},
};

const ROW_MIN_HEIGHT: f32 = 50.0;
const ROW_PADDING_X: f32 = 14.0;
const ROW_PADDING_Y: f32 = 8.0;
const ROW_GAP: f32 = 12.0;
/// Between the number, the handle, the icon and the text.
const LEAD_GAP: f32 = 10.0;
const NUMBER_SIZE: f32 = 22.0;
const NUMBER_TEXT_SIZE: f32 = 12.0;
const ICON_TILE: f32 = 30.0;
const ICON_SIZE: f32 = 16.0;
const DESCRIPTION_LINE_HEIGHT: f32 = 17.0;
/// A page that is off is drawn at this opacity, its switch excepted.
const OFF_OPACITY: f32 = 0.5;

struct DraggedPage {
    index: usize,
    label: SharedString,
}

struct DragPreview(SharedString);

impl Render for DragPreview {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px_3()
            .py_1()
            .rounded_md()
            .bg(crate::palette::button_fill(cx))
            .text_color(cx.theme().foreground)
            .child(self.0.clone())
    }
}

/// What the list knows about one page of the notch.
struct PageRow {
    id: &'static str,
    icon: &'static str,
    description: fn() -> &'static str,
}

/// One row per page id; a page the notch has that is missing here is listed without a description and with the Pages icon.
const PAGE_ROWS: &[PageRow] = &[
    PageRow { id: "media", icon: icons::PLAY, description: || tr!("pages.desc_media") },
    PageRow { id: "shelf", icon: icons::INBOX, description: || tr!("pages.desc_shelf") },
    PageRow { id: "calendar", icon: icons::CALENDAR, description: || tr!("pages.desc_calendar") },
    PageRow { id: "timer", icon: icons::TIMER, description: || tr!("pages.desc_timer") },
    PageRow { id: "todo", icon: icons::TODO, description: || tr!("pages.desc_todo") },
    PageRow { id: "shalat", icon: icons::SHALAT, description: || tr!("pages.desc_shalat") },
    PageRow { id: "stats", icon: icons::STATS, description: || tr!("pages.desc_stats") },
    PageRow { id: "weather", icon: icons::WEATHER, description: || tr!("pages.desc_weather") },
    PageRow { id: "devices", icon: icons::BLUETOOTH, description: || tr!("pages.desc_devices") },
    PageRow { id: "clipboard", icon: icons::CLIPBOARD, description: || tr!("pages.desc_clipboard") },
];

fn page_row_of(id: &str) -> Option<&'static PageRow> {
    PAGE_ROWS.iter().find(|row| row.id == id)
}

pub fn page_description(id: &str) -> Option<&'static str> {
    page_row_of(id).map(|row| (row.description)())
}

/// The icon of a page; one that has no row gets the Pages icon.
pub fn page_icon(id: &str) -> &'static str {
    page_row_of(id).map_or(icons::PAGES, |row| row.icon)
}

/// Two columns of dots, the usual sign that a row can be dragged.
fn grip(cx: &App) -> AnyElement {
    let color = cx.theme().muted_foreground.opacity(0.7);
    let column =
        move || v_flex().gap(px(3.0)).children((0..3).map(move |_| div().size(px(3.0)).rounded_full().bg(color)));
    h_flex().flex_none().gap(px(3.0)).child(column()).child(column()).into_any_element()
}

/// The square with the digit key of the page: blue for a page shown, a dashed "–" for one the notch skips.
fn number_badge(hint: KeyHint, cx: &App) -> AnyElement {
    let accent = crate::palette::accent(cx);
    let text = match hint {
        KeyHint::Digit(digit) | KeyHint::DigitAndZero(digit) => digit.to_string(),
        KeyHint::Zero => "0".to_owned(),
        KeyHint::Skipped => "–".to_owned(),
    };
    let badge = div()
        .flex_none()
        .size(px(NUMBER_SIZE))
        .rounded(px(6.0))
        .border_1()
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(NUMBER_TEXT_SIZE))
        .child(text);
    match hint {
        KeyHint::Skipped => badge.border_dashed().border_color(cx.theme().foreground.opacity(0.14)),
        _ => badge.bg(accent.opacity(0.18)).border_color(accent.opacity(0.35)).text_color(accent),
    }
    .into_any_element()
}

/// One row of the list; `index` is its place in the order.
fn page_row(index: usize, id: &str, label: SharedString, enabled: bool, hint: KeyHint, cx: &App) -> AnyElement {
    let switch_id = id.to_owned();
    let switch = field::bare_switch(("page-switch", index), enabled, move |enabled, _window, cx| {
        let mut settings = Settings::load();
        let known = parse_known(settings.get_string(KNOWN_KEY));
        let order = current_order(&known, settings.get_string(ORDER_KEY));
        let mut disabled = disabled_pages(&known, settings.get_string(DISABLED_KEY));
        if switch_page(&order, &mut disabled, &switch_id, enabled) {
            apply(&mut settings, DISABLED_KEY, &order_text(&order, &disabled).1);
        }
        cx.refresh_windows();
    });
    let title = h_flex()
        .gap(px(8.0))
        .items_center()
        .child(div().text_base().font_weight(gpui::FontWeight::MEDIUM).child(label.clone()))
        .children(matches!(hint, KeyHint::DigitAndZero(_)).then(|| tag(tr!("pages.also_key_zero"), Tone::Accent, cx)));
    let text = v_flex().flex_1().min_w_0().child(title).children(page_description(id).map(|description| {
        div()
            .text_sm()
            .line_height(px(DESCRIPTION_LINE_HEIGHT))
            .text_color(cx.theme().muted_foreground)
            .child(description)
    }));
    let icon_tile = div()
        .flex_none()
        .size(px(ICON_TILE))
        .rounded(px(8.0))
        .bg(cx.theme().foreground.opacity(0.08))
        .flex()
        .items_center()
        .justify_center()
        .child(Icon::default().path(page_icon(id)).size(px(ICON_SIZE)).text_color(cx.theme().foreground.opacity(0.8)));
    let lead = h_flex()
        .flex_1()
        .min_w_0()
        .gap(px(LEAD_GAP))
        .items_center()
        .when(!enabled, |lead| lead.opacity(OFF_OPACITY))
        .child(number_badge(hint, cx))
        .child(grip(cx))
        .child(icon_tile)
        .child(text);
    let drop_highlight = crate::palette::accent(cx).opacity(0.12);
    let dragged = DraggedPage { index, label };
    h_flex()
        .id(("page-row", index))
        .min_h(px(ROW_MIN_HEIGHT))
        .px(px(ROW_PADDING_X))
        .py(px(ROW_PADDING_Y))
        .gap(px(ROW_GAP))
        .items_center()
        .when(index > 0, |row| row.border_t_1().border_color(crate::palette::row_divider(cx)))
        .cursor_grab()
        .on_drag(dragged, |dragged, _position, _window, cx| cx.new(|_| DragPreview(dragged.label.clone())))
        .drag_over::<DraggedPage>(move |style, _dragged, _window, _cx| style.bg(drop_highlight))
        .on_drop(move |dragged: &DraggedPage, _window, cx| {
            let mut settings = Settings::load();
            let known = parse_known(settings.get_string(KNOWN_KEY));
            let mut order = current_order(&known, settings.get_string(ORDER_KEY));
            reorder(&mut order, dragged.index, index);
            apply(&mut settings, ORDER_KEY, &order_text(&order, &[]).0);
            cx.refresh_windows();
        })
        .child(lead)
        .child(switch)
        .into_any_element()
}

/// One page as the list draws it.
pub struct ListedPage {
    pub id: String,
    pub label: SharedString,
    pub enabled: bool,
    pub hint: KeyHint,
}

pub fn rows(pages: &[ListedPage], cx: &App) -> Vec<AnyElement> {
    pages
        .iter()
        .enumerate()
        .map(|(index, page)| page_row(index, &page.id, page.label.clone(), page.enabled, page.hint, cx))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clipboard_page_has_a_description() {
        let description = page_description("clipboard").expect("clipboard description");
        assert!(description.starts_with("History of the last 7 copies"));
    }

    #[test]
    fn every_page_of_the_notch_has_a_row_with_a_description() {
        for id in ["media", "shelf", "calendar", "timer", "todo", "shalat", "stats", "weather", "devices", "clipboard"]
        {
            assert!(page_description(id).is_some_and(|text| !text.is_empty()), "{id} has no description");
        }
    }

    #[test]
    fn no_two_rows_share_an_id_or_an_icon() {
        let mut ids: Vec<&str> = PAGE_ROWS.iter().map(|row| row.id).collect();
        let mut row_icons: Vec<&str> = PAGE_ROWS.iter().map(|row| row.icon).collect();
        ids.sort_unstable();
        ids.dedup();
        row_icons.sort_unstable();
        row_icons.dedup();
        assert_eq!(ids.len(), PAGE_ROWS.len());
        assert_eq!(row_icons.len(), PAGE_ROWS.len());
    }

    #[test]
    fn a_row_reads_its_description_in_the_current_language() {
        use store::language::{Language, with_override};
        let english = page_description("todo").expect("todo description");
        let indonesian = with_override(Language::Indonesian, || page_description("todo")).expect("todo description");
        assert_ne!(english, indonesian);
    }

    #[test]
    fn a_page_without_a_row_has_no_description_and_the_pages_icon() {
        assert_eq!(page_description("someday"), None);
        assert_eq!(page_icon("someday"), icons::PAGES);
    }

    #[test]
    fn a_row_gives_its_page_the_icon_it_names() {
        assert_eq!(page_icon("clipboard"), icons::CLIPBOARD);
        assert_eq!(page_icon("devices"), icons::BLUETOOTH);
    }
}
