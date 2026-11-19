//! The Pages page: a picture of the page dots in the notch, the numbered page list, and the hints about the digit keys
//! and switching a page off.

mod list;
mod order;
mod preview;

use std::rc::Rc;

use gpui::{AnyElement, App, SharedString, div, prelude::*, px};
use gpui_component::{ActiveTheme as _, IconName, v_flex};
use store::Settings;

use super::about::widgets::{open_url_handler, pill};
use crate::{
    i18n::tr,
    icons, info_box,
    layout::{Group, Item, Page, block_frame, section_title},
    links::{REPOSITORY_URL, repository_label},
};
use order::{DISABLED_KEY, KNOWN_KEY, KeyHint, ORDER_KEY, current_order, disabled_pages, key_hints, parse_known};

const SECTION_TITLE_GAP: f32 = 7.0;
const NOTE_LINE_HEIGHT: f32 = 17.0;

/// The pages in their order, each with its state and key; empty until the notch has reported its pages.
fn listed_pages() -> Vec<list::ListedPage> {
    let settings = Settings::load();
    let known = parse_known(settings.get_string(KNOWN_KEY));
    let order = current_order(&known, settings.get_string(ORDER_KEY));
    let disabled = disabled_pages(&known, settings.get_string(DISABLED_KEY));
    let skipped: Vec<String> = order.iter().filter(|id| disabled.contains(id)).cloned().collect();
    let hints = key_hints(&order, &skipped);
    order
        .into_iter()
        .zip(hints)
        .map(|(id, hint)| {
            let label = known.iter().find(|page| page.id == id).map(|page| page.label.clone()).unwrap_or_default();
            list::ListedPage { enabled: !disabled.contains(&id), label: label.into(), hint, id }
        })
        .collect()
}

/// "9 pages · 8 on · drag to reorder".
fn count_note(pages: &[list::ListedPage]) -> SharedString {
    let on = pages.iter().filter(|page| page.enabled).count();
    tr!("pages.count_note", total = pages.len(), on = on).into()
}

fn preview_section(cx: &App) -> AnyElement {
    let shown: Vec<String> =
        listed_pages().into_iter().filter(|page| page.hint != KeyHint::Skipped).map(|page| page.label.into()).collect();
    let muted = cx.theme().muted_foreground;
    block_frame(cx)
        .child(div().px(px(14.0)).py(px(12.0)).child(preview::strip(&shown, cx)))
        .child(
            div()
                .px(px(14.0))
                .pb(px(12.0))
                .text_sm()
                .line_height(px(NOTE_LINE_HEIGHT))
                .text_color(muted)
                .child(tr!("pages.preview_note")),
        )
        .into_any_element()
}

fn list_section(cx: &App) -> AnyElement {
    let pages = listed_pages();
    if pages.is_empty() {
        return block_frame(cx)
            .p(px(14.0))
            .text_sm()
            .text_color(cx.theme().muted_foreground)
            .child(tr!("pages.not_reported"))
            .into_any_element();
    }
    v_flex()
        .gap(px(SECTION_TITLE_GAP))
        .child(section_title(tr!("pages.group_pages"), Some(count_note(&pages)), cx))
        .child(block_frame(cx).children(list::rows(&pages, cx)))
        .into_any_element()
}

/// "Want to add your own page?": a link to the repository, where a page can be added.
fn contribute_group() -> Group {
    let repository_button = |_window: &mut gpui::Window, cx: &mut App| {
        pill("pages-repository", repository_label(), Some(IconName::GitHub), open_url_handler(REPOSITORY_URL), cx)
    };
    Group::titled(
        tr!("pages.contribute_title"),
        vec![
            Item::new(tr!("pages.contribute_row"), Rc::new(repository_button))
                .description(tr!("pages.contribute_note")),
        ],
    )
}

pub fn page() -> Page {
    Page::plain(
        tr!("pages.title"),
        vec![
            Group::titled(
                tr!("pages.group_preview"),
                vec![Item::custom(|_window, cx| preview_section(cx)).outside_block()],
            ),
            Group::loose(vec![Item::custom(|_window, cx| {
                info_box::info_box(icons::KEYBOARD, tr!("pages.keys_hint"), cx)
            })]),
            Group::loose(vec![
                Item::custom(|_window, cx| list_section(cx)),
                Item::custom(|_window, cx| info_box::info_box(icons::ABOUT, tr!("pages.off_hint"), cx)),
            ]),
            contribute_group(),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn listed(enabled: bool) -> list::ListedPage {
        list::ListedPage { id: "media".to_owned(), label: "Media".into(), enabled, hint: KeyHint::Skipped }
    }

    #[test]
    fn the_page_opens_with_the_preview_and_the_key_hint_comes_second() {
        let page = page();
        assert_eq!(page.group_titles().first().copied(), Some("How it looks in Foldline"));
        assert_eq!(page.group_titles().len(), 2, "the hint and the list groups carry no title");
    }

    #[test]
    fn the_page_ends_with_the_section_about_adding_your_own_page() {
        assert_eq!(page().group_titles().last().copied(), Some("Want to add your own page?"));
    }

    #[test]
    fn the_contribute_texts_exist_in_both_languages() {
        use store::language::{Language, with_override};
        let indonesian = with_override(Language::Indonesian, || tr!("pages.contribute_title"));
        assert_eq!(indonesian, "Ingin menambahkan halaman Anda sendiri?");
        assert!(tr!("pages.contribute_note").contains("pull request"));
    }

    #[test]
    fn the_count_note_says_how_many_pages_there_are_and_how_many_are_on() {
        assert_eq!(count_note(&[listed(true), listed(false), listed(true)]), "3 pages · 2 on · drag to reorder");
    }
}
