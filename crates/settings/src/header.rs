//! The content pane's header: the breadcrumb on the left, the window buttons at the right end. The strip is the
//! window's drag area. The buttons are drawn here, not by the component library's title bar, to keep the header's height and colors.

use crate::i18n::tr;
use gpui::{App, ElementId, IntoElement, Window, WindowControlArea, div, prelude::*, px};
use gpui_component::{ActiveTheme as _, Icon, IconName, Sizable as _, h_flex};

use crate::palette;

const HEIGHT: f32 = 44.0;
const BUTTON_WIDTH: f32 = 36.0;
/// The breadcrumb's inset from the card's left edge, and the space around its chevron.
const TRAIL_PADDING_LEFT: f32 = 22.0;
const TRAIL_GAP: f32 = 10.0;
const CHEVRON_SIZE: f32 = 13.0;

/// The breadcrumb as the two names it joins: "Settings" and the page's title.
pub fn breadcrumb(page_title: &'static str) -> [&'static str; 2] {
    [tr!("header.settings"), page_title]
}

/// The breadcrumb as one line of text, with a "›" between the names.
#[cfg(test)]
pub fn breadcrumb_text(page_title: &'static str) -> String {
    breadcrumb(page_title).join(" › ")
}

fn control(id: &'static str, icon: IconName, area: WindowControlArea, is_close: bool, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    let (hover_fill, hover_text) =
        if is_close { (theme.danger, theme.danger_foreground) } else { (palette::wash(cx), theme.foreground) };
    div()
        .id(ElementId::Name(id.into()))
        .flex()
        .flex_none()
        .w(px(BUTTON_WIDTH))
        .h_full()
        .items_center()
        .justify_center()
        .text_color(theme.foreground)
        .window_control_area(area)
        .hover(move |style| style.bg(hover_fill).text_color(hover_text))
        .child(Icon::new(icon).small())
}

pub fn render(page_title: &'static str, window: &Window, cx: &App) -> impl IntoElement {
    let [root, page] = breadcrumb(page_title);
    let trail = h_flex()
        .id("header-trail")
        .window_control_area(WindowControlArea::Drag)
        .flex_1()
        .h_full()
        .pl(px(TRAIL_PADDING_LEFT))
        .gap(px(TRAIL_GAP))
        .items_center()
        .text_base()
        .child(div().text_color(palette::muted(cx)).child(root))
        .child(Icon::new(IconName::ChevronRight).size(px(CHEVRON_SIZE)).text_color(palette::muted(cx)))
        .child(div().font_weight(gpui::FontWeight::MEDIUM).text_color(palette::accent(cx)).child(page));
    let maximize = if window.is_maximized() { IconName::WindowRestore } else { IconName::WindowMaximize };
    h_flex()
        .h(px(HEIGHT))
        .flex_none()
        .bg(palette::header(cx))
        .border_b_1()
        .border_color(palette::row_divider(cx))
        .child(trail)
        .child(control("minimize", IconName::WindowMinimize, WindowControlArea::Min, false, cx))
        .child(control("maximize", maximize, WindowControlArea::Max, false, cx))
        .child(control("close", IconName::WindowClose, WindowControlArea::Close, true, cx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_breadcrumb_reads_settings_then_the_page() {
        assert_eq!(breadcrumb_text("General"), "Settings › General");
        assert_eq!(breadcrumb_text("Page settings"), "Settings › Page settings");
    }

    #[test]
    fn every_section_has_its_own_breadcrumb() {
        let texts = crate::sections::all().iter().map(|entry| breadcrumb_text(entry.page.title)).collect::<Vec<_>>();
        assert_eq!(
            texts,
            [
                "Settings › General",
                "Settings › Pages",
                "Settings › Behavior",
                "Settings › Advanced",
                "Settings › Calendar",
                "Settings › Timer",
                "Settings › Shalat",
                "Settings › System stats",
                "Settings › Weather",
                "Settings › About",
            ]
        );
    }
}
