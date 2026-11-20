//! The sidebar: the app's mark and name, then the pages (the page settings under a label), sitting straight on the window frame.

use crate::i18n::tr;
use std::rc::Rc;

use gpui::{AnyElement, App, ClickEvent, IntoElement, SharedString, Window, WindowControlArea, div, prelude::*, px};
use gpui_component::{ActiveTheme as _, Icon, h_flex, v_flex};

use crate::{layout::Page, palette};

/// Called with the index of the entry that was clicked.
pub type SelectHandler = Rc<dyn Fn(usize, &mut App)>;

const ITEM_HEIGHT: f32 = 36.0;
/// Between `text_sm` (12.7) and the base size (14.5).
const ITEM_TEXT_SIZE: f32 = 14.0;
const ITEM_RADIUS: f32 = 8.0;
/// Between two items of a group.
const ITEM_GAP: f32 = 1.0;
const ICON_SIZE: f32 = 17.0;
/// Space between an item's edge and its icon.
const ITEM_PADDING: f32 = 10.0;
const ICON_GAP: f32 = 9.0;
/// Space above a group's label, and between the label and its first item.
const GROUP_LABEL_ABOVE: f32 = 12.0;
const GROUP_LABEL_BELOW: f32 = 5.0;
/// The sidebar's padding: above the brand, at each side of the items and below the footer; all of it is drag area.
const SIDEBAR_PADDING_TOP: f32 = 16.0;
const SIDEBAR_PADDING_X: f32 = 10.0;
const SIDEBAR_PADDING_BOTTOM: f32 = 12.0;
/// The brand's inset from the gutter, and the space between the brand and the first item.
const BRAND_PADDING_X: f32 = 8.0;
const BRAND_PADDING_BOTTOM: f32 = 36.0;

/// The sidebar's groups, in the order they are shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavGroup {
    /// The notch as a whole.
    Notch,
    /// Settings that belong to one page of the notch (Calendar, Shalat).
    PageSettings,
    /// Items pinned to the bottom of the sidebar without a label (About); not part of `ORDER`.
    Footer,
}

impl NavGroup {
    pub const ORDER: [NavGroup; 2] = [NavGroup::Notch, NavGroup::PageSettings];

    /// The label above the group; the notch's own pages and the footer have none.
    pub fn label(self) -> Option<&'static str> {
        match self {
            NavGroup::PageSettings => Some(tr!("sidebar.page_settings")),
            NavGroup::Notch | NavGroup::Footer => None,
        }
    }
}

/// One sidebar item: the page it opens, its icon and the group it sits in.
pub struct NavEntry {
    pub group: NavGroup,
    pub icon: &'static str,
    pub page: Page,
}

impl NavEntry {
    pub fn new(group: NavGroup, icon: &'static str, page: Page) -> Self {
        Self { group, icon, page }
    }
}

/// The entries' indexes per group, groups in `NavGroup::ORDER`, entries in the order given; a group without entries is left out.
pub fn grouped(entries: &[NavEntry]) -> Vec<(NavGroup, Vec<usize>)> {
    NavGroup::ORDER
        .into_iter()
        .map(|group| {
            let indexes = entries
                .iter()
                .enumerate()
                .filter(|(_, entry)| entry.group == group)
                .map(|(index, _)| index)
                .collect::<Vec<_>>();
            (group, indexes)
        })
        .filter(|(_, indexes)| !indexes.is_empty())
        .collect()
}

fn item(
    id: impl Into<SharedString>,
    icon: Icon,
    label: &'static str,
    selected: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    cx: &App,
) -> AnyElement {
    let fill = palette::wash(cx);
    h_flex()
        .id(gpui::ElementId::Name(id.into()))
        .h(px(ITEM_HEIGHT))
        .px(px(ITEM_PADDING))
        .gap(px(ICON_GAP))
        .items_center()
        .rounded(px(ITEM_RADIUS))
        .cursor_pointer()
        .text_size(px(ITEM_TEXT_SIZE))
        .text_color(cx.theme().foreground)
        .when(selected, |row| row.bg(fill))
        .when(!selected, |row| row.hover(move |style| style.bg(fill.opacity(0.6))))
        .on_click(on_click)
        .child(icon.size(px(ICON_SIZE)).text_color(palette::accent(cx)))
        .child(label)
        .into_any_element()
}

/// A group's label with the space above and below it; it is not clickable, so it moves the window like the empty sidebar.
fn group_label(label: &'static str, cx: &App) -> impl IntoElement {
    h_flex()
        .id(gpui::ElementId::Name(format!("sidebar-group-{label}").into()))
        .window_control_area(WindowControlArea::Drag)
        .pt(px(GROUP_LABEL_ABOVE))
        .pb(px(GROUP_LABEL_BELOW))
        .px(px(ITEM_PADDING))
        .items_center()
        .text_sm()
        .text_color(palette::muted(cx))
        .child(label)
}

/// A strip of the sidebar that moves the window when dragged; items are never inside one, so their clicks stay clicks.
fn drag_area(id: impl Into<SharedString>) -> gpui::Stateful<gpui::Div> {
    div().id(gpui::ElementId::Name(id.into())).window_control_area(WindowControlArea::Drag)
}

/// Items between two drag gutters, so the sidebar's side padding moves the window too.
fn guttered(id: &'static str, items: gpui::Div) -> gpui::Div {
    h_flex()
        .child(drag_area(format!("{id}-left")).flex_none().w(px(SIDEBAR_PADDING_X)))
        .child(items.flex_1().min_w_0())
        .child(drag_area(format!("{id}-right")).flex_none().w(px(SIDEBAR_PADDING_X)))
}

/// The whole sidebar column. `on_select` gets the index of the clicked entry; footer entries sit at the bottom.
pub fn render(
    brand: AnyElement,
    entries: &[NavEntry],
    selected: usize,
    on_select: SelectHandler,
    cx: &App,
) -> AnyElement {
    let nav_item = |index: usize| {
        let entry = &entries[index];
        let on_select = on_select.clone();
        item(
            format!("nav-{}", entry.page.title),
            Icon::default().path(entry.icon),
            entry.page.title,
            index == selected,
            move |_event, _window, cx| on_select(index, cx),
            cx,
        )
    };
    let groups = grouped(entries).into_iter().map(|(group, indexes)| {
        v_flex()
            .children(group.label().map(|label| group_label(label, cx)))
            .child(v_flex().gap(px(ITEM_GAP)).children(indexes.into_iter().map(nav_item)))
    });
    let footer = entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.group == NavGroup::Footer)
        .map(|(index, _)| nav_item(index));
    v_flex()
        .h_full()
        .flex_none()
        .w(px(crate::SIDEBAR_WIDTH))
        .child(
            h_flex()
                .id("sidebar-brand")
                .window_control_area(WindowControlArea::Drag)
                .flex_none()
                .px(px(SIDEBAR_PADDING_X + BRAND_PADDING_X))
                .pt(px(SIDEBAR_PADDING_TOP))
                .pb(px(BRAND_PADDING_BOTTOM))
                .items_center()
                .child(brand),
        )
        .child(guttered("sidebar-gutter-groups", v_flex().children(groups)))
        // The empty space below the items moves the window.
        .child(drag_area("sidebar-empty").flex_1().min_h_0().w_full())
        .child(guttered("sidebar-gutter-footer", v_flex().gap(px(ITEM_GAP)).children(footer)))
        .child(drag_area("sidebar-bottom").flex_none().w_full().h(px(SIDEBAR_PADDING_BOTTOM)))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(group: NavGroup, title: &'static str) -> NavEntry {
        NavEntry::new(group, crate::icons::GENERAL, Page::plain(title, Vec::new()))
    }

    #[test]
    fn page_settings_sit_in_their_own_group_after_the_notch_group() {
        let entries = [
            entry(NavGroup::Notch, "General"),
            entry(NavGroup::PageSettings, "Calendar"),
            entry(NavGroup::Notch, "Advanced"),
        ];
        assert_eq!(grouped(&entries), vec![(NavGroup::Notch, vec![0, 2]), (NavGroup::PageSettings, vec![1])]);
    }

    #[test]
    fn an_empty_group_is_left_out() {
        let entries = [entry(NavGroup::PageSettings, "Shalat")];
        assert_eq!(grouped(&entries), vec![(NavGroup::PageSettings, vec![0])]);
    }

    #[test]
    fn the_tile_sits_about_as_far_from_the_top_as_from_the_left_edge() {
        const TOP_GAP: f32 = SIDEBAR_PADDING_TOP;
        const LEFT_GAP: f32 = SIDEBAR_PADDING_X + BRAND_PADDING_X;
        const { assert!((LEFT_GAP - TOP_GAP).abs() <= 2.0) };
    }

    #[test]
    fn the_brand_and_the_first_item_keep_a_gap_on_the_spacing_scale() {
        const { assert!(BRAND_PADDING_BOTTOM >= 32.0 && BRAND_PADDING_BOTTOM <= 40.0) };
    }

    #[test]
    fn an_item_stays_a_comfortable_click_target_and_holds_its_icon() {
        const { assert!(ITEM_HEIGHT >= 32.0) };
        const { assert!(ICON_SIZE + 8.0 <= ITEM_HEIGHT) };
    }

    #[test]
    fn the_item_text_sits_between_the_small_text_and_the_base_size() {
        const SMALL_TEXT: f32 = crate::UI_FONT_SIZE * 0.875;
        const { assert!(ITEM_TEXT_SIZE > SMALL_TEXT && ITEM_TEXT_SIZE < crate::UI_FONT_SIZE) };
        const { assert!(ITEM_TEXT_SIZE + 8.0 <= ITEM_HEIGHT) };
    }

    #[test]
    fn only_the_page_settings_group_has_a_label() {
        assert_eq!(NavGroup::Notch.label(), None);
        assert_eq!(NavGroup::Footer.label(), None);
        assert_eq!(NavGroup::PageSettings.label(), Some("Page settings"));
    }

    #[test]
    fn the_real_sections_group_the_page_specific_ones_apart() {
        let entries = crate::sections::all();
        let group_of = |title: &str| entries.iter().find(|entry| entry.page.title == title).map(|entry| entry.group);
        for title in ["General", "Pages", "Behavior", "Advanced"] {
            assert_eq!(group_of(title), Some(NavGroup::Notch), "{title}");
        }
        for title in ["Calendar", "Timer", "Shalat", "System stats", "Weather"] {
            assert_eq!(group_of(title), Some(NavGroup::PageSettings), "{title}");
        }
    }

    #[test]
    fn advanced_sits_in_the_notch_group() {
        let entries = crate::sections::all();
        let advanced = entries.iter().find(|entry| entry.icon == crate::icons::ADVANCED).expect("advanced entry");
        assert_eq!(advanced.group, NavGroup::Notch);
        let notch_group =
            grouped(&entries).into_iter().find(|(group, _)| *group == NavGroup::Notch).expect("notch group");
        let last_in_notch_group = notch_group.1.last().copied().expect("notch entries");
        assert_eq!(entries[last_in_notch_group].icon, crate::icons::ADVANCED);
    }

    #[test]
    fn shalat_is_listed_under_page_settings() {
        let entries = crate::sections::all();
        let shalat = entries.iter().find(|entry| entry.icon == crate::icons::SHALAT).expect("shalat entry");
        assert_eq!(shalat.group, NavGroup::PageSettings);
        assert_eq!(shalat.page.title, "Shalat");
    }

    #[test]
    fn shalat_keeps_its_own_word_in_indonesian() {
        use store::language::{Language, with_override};
        let title = with_override(Language::Indonesian, || {
            crate::sections::all()
                .into_iter()
                .find(|entry| entry.icon == crate::icons::SHALAT)
                .map(|entry| entry.page.title)
        });
        assert_eq!(title, Some("Shalat"));
    }
}
