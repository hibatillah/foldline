//! The sidebar: the app's mark and name, then the pages in labelled groups, sitting straight on the window frame.

use crate::i18n::tr;
use std::rc::Rc;

use gpui::{AnyElement, App, ClickEvent, IntoElement, SharedString, Window, WindowControlArea, div, prelude::*, px};
use gpui_component::{ActiveTheme as _, Icon, h_flex, v_flex};

use crate::{layout::Page, palette};

/// Called with the index of the entry that was clicked.
pub type SelectHandler = Rc<dyn Fn(usize, &mut App)>;

const ITEM_HEIGHT: f32 = 32.0;
const GROUP_LABEL_HEIGHT: f32 = 24.0;
const GROUP_GAP: f32 = 14.0;
const ICON_SIZE: f32 = 14.0;
/// Space between an item's edge and its icon, and between the icon and the label.
const ITEM_PADDING: f32 = 10.0;
/// The gutter at each side of the items and below them; it is part of the drag area.
const SIDEBAR_PADDING: f32 = 8.0;
/// The brand's inset from the sidebar's edge.
const BRAND_PADDING_X: f32 = 10.0;
/// The top strip that holds the brand; dragging it moves the window.
pub const BRAND_HEIGHT: f32 = 58.0;
/// Extra space above the brand; the tile's centered gap to the window top (12 px) looks smaller than its 18 px gap to the left edge.
const BRAND_TOP_PADDING: f32 = 6.0;

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

    pub fn label(self) -> &'static str {
        match self {
            NavGroup::Notch => "Notch",
            NavGroup::PageSettings => tr!("sidebar.page_settings"),
            NavGroup::Footer => "",
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
    let foreground = cx.theme().foreground;
    let color = if selected { foreground } else { foreground.opacity(0.78) };
    let fill = palette::wash(cx);
    h_flex()
        .id(gpui::ElementId::Name(id.into()))
        .h(px(ITEM_HEIGHT))
        .px(px(ITEM_PADDING))
        .gap(px(ITEM_PADDING))
        .items_center()
        .rounded(px(8.0))
        .cursor_pointer()
        .text_sm()
        .text_color(color)
        .when(selected, |row| row.bg(fill))
        .when(!selected, |row| row.hover(move |style| style.bg(fill.opacity(0.6)).text_color(foreground)))
        .on_click(on_click)
        .child(icon.size(px(ICON_SIZE)).text_color(palette::accent(cx)))
        .child(label)
        .into_any_element()
}

/// A group's label; it is not clickable, so it moves the window like the empty sidebar.
fn group_label(label: &'static str, cx: &App) -> impl IntoElement {
    h_flex()
        .id(gpui::ElementId::Name(format!("sidebar-group-{label}").into()))
        .window_control_area(WindowControlArea::Drag)
        .h(px(GROUP_LABEL_HEIGHT))
        .px(px(ITEM_PADDING))
        .items_center()
        .text_xs()
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
        .child(drag_area(format!("{id}-left")).flex_none().w(px(SIDEBAR_PADDING)))
        .child(items.flex_1().min_w_0())
        .child(drag_area(format!("{id}-right")).flex_none().w(px(SIDEBAR_PADDING)))
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
        v_flex().gap_0p5().child(group_label(group.label(), cx)).children(indexes.into_iter().map(nav_item))
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
                .h(px(BRAND_HEIGHT))
                .px(px(SIDEBAR_PADDING + BRAND_PADDING_X))
                .pt(px(BRAND_TOP_PADDING))
                .items_center()
                .child(brand),
        )
        .child(guttered("sidebar-gutter-groups", v_flex().gap(px(GROUP_GAP)).children(groups)))
        // The empty space below the items moves the window.
        .child(drag_area("sidebar-empty").flex_1().min_h_0().w_full())
        .child(guttered("sidebar-gutter-footer", v_flex().gap_1().children(footer)))
        .child(drag_area("sidebar-bottom").flex_none().w_full().h(px(SIDEBAR_PADDING)))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(group: NavGroup, title: &'static str) -> NavEntry {
        NavEntry::new(group, crate::icons::GENERAL, Page::new(title, "", Vec::new()))
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
        const TOP_GAP: f32 = BRAND_TOP_PADDING + (BRAND_HEIGHT - BRAND_TOP_PADDING - crate::LOGO_SIZE) / 2.0;
        const LEFT_GAP: f32 = SIDEBAR_PADDING + BRAND_PADDING_X;
        const { assert!((LEFT_GAP - TOP_GAP).abs() <= 1.0) };
    }

    #[test]
    fn the_group_labels_are_notch_and_page_settings() {
        assert_eq!(NavGroup::Notch.label(), "Notch");
        assert_eq!(NavGroup::PageSettings.label(), "Page settings");
    }

    #[test]
    fn the_real_sections_group_the_page_specific_ones_apart() {
        let entries = crate::sections::all();
        let titles = |group: NavGroup| {
            entries.iter().filter(|entry| entry.group == group).map(|entry| entry.page.title).collect::<Vec<_>>()
        };
        assert_eq!(titles(NavGroup::Notch), ["General", "Pages", "Behavior", "Advanced"]);
        assert_eq!(titles(NavGroup::PageSettings), ["Calendar", "Shalat", "System stats", "Weather"]);
    }
}
