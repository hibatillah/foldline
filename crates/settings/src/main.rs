#![windows_subsystem = "windows"]

rust_i18n::i18n!("locales", fallback = "en");

mod brand;
mod card;
mod chips;
mod field;
mod header;
mod i18n;
mod icons;
mod illustration;
mod info_box;
mod layout;
mod location;
mod notify;
mod palette;
mod picture;
#[path = "../../../scripts/release_version.rs"]
mod release_version;
mod sections;
mod sidebar;
mod single_instance;
mod startup;
mod stepper;
mod switch;
mod tag;
mod version;

use std::rc::Rc;

use gpui::{
    App, Application, Bounds, Context, SharedString, Window, WindowBounds, WindowOptions, div, img, prelude::*, px,
    size,
};
use gpui_component::{ActiveTheme as _, Root, Theme, TitleBar, h_flex, v_flex};

use sidebar::{NavEntry, SelectHandler};

const SIDEBAR_WIDTH: f32 = 216.0;
const MINIMUM_WINDOW_SIZE: (f32, f32) = (720.0, 480.0);
const DEFAULT_WINDOW_SIZE: (f32, f32) = (1040.0, 660.0);
/// Space kept free around the window when the screen is smaller than the default size (taskbar and edges).
const SCREEN_MARGIN: f32 = 80.0;
const LOGO_SIZE: f32 = 28.0;
/// The one scale knob: gpui-component's Root sets the window's rem size to this, so every rem-based size (text, `px_*`, `gap_*`, controls) follows it. 16 is the library default.
const UI_FONT_SIZE: f32 = 14.5;
// The notch's own font files (SIL Open Font License, assets/fonts/Geist-LICENSE.txt).
const GEIST_FONTS: [&[u8]; 2] = [
    include_bytes!("../../../assets/fonts/Geist-Regular.ttf"),
    include_bytes!("../../../assets/fonts/Geist-Medium.ttf"),
];
const FONT_FAMILY: &str = "Geist";
/// The space between the pane's header and the page, at the page's sides and below it.
const CONTENT_PADDING_TOP: f32 = 16.0;
const CONTENT_PADDING_X: f32 = 22.0;
const CONTENT_PADDING_BOTTOM: f32 = 20.0;
/// Between the tile, "Foldline" and the version.
const BRAND_GAP: f32 = 8.0;

struct SettingsWindow {
    selected_page_index: usize,
    pages: Vec<NavEntry>,
    /// The language `pages` was built in; the pages are built again when it changes.
    built_for: store::language::Language,
}

impl Render for SettingsWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if store::language::current() != self.built_for {
            self.built_for = store::language::current();
            self.pages = sections::all();
        }
        let view = cx.entity();
        let version_label = format!("v{}", version::displayed_version());
        let brand = h_flex()
            .gap(px(BRAND_GAP))
            .items_center()
            .child(img(brand::tile_image()).size(px(LOGO_SIZE)).flex_none())
            .child(
                h_flex()
                    .gap(px(BRAND_GAP))
                    .items_baseline()
                    .min_w_0()
                    .child(div().flex_none().text_lg().font_weight(gpui::FontWeight::SEMIBOLD).child("Foldline"))
                    .child(
                        div()
                            .min_w_0()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .text_xs()
                            .text_color(palette::muted(cx))
                            .child(version_label),
                    ),
            )
            .into_any_element();
        let on_select: SelectHandler = Rc::new(move |page_index, cx| {
            view.update(cx, |this, cx| {
                this.selected_page_index = page_index;
                cx.notify();
            });
        });
        let sidebar = sidebar::render(brand, &self.pages, self.selected_page_index, on_select, cx);
        let selected = &self.pages[self.selected_page_index].page;
        let content = div()
            // One scroll position per page, so a page opens at its top.
            .id(("content", self.selected_page_index))
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .pt(px(CONTENT_PADDING_TOP))
            .px(px(CONTENT_PADDING_X))
            .pb(px(CONTENT_PADDING_BOTTOM))
            .child(selected.render(window, cx));
        let pane = v_flex()
            .size_full()
            .border_l_1()
            .border_color(palette::card_border(cx))
            .bg(palette::card(cx))
            .overflow_hidden()
            .child(header::render(selected.title, window, cx))
            .child(content);

        v_flex()
            .relative()
            .size_full()
            .text_color(cx.theme().foreground)
            .child(
                h_flex()
                    .size_full()
                    .child(div().h_full().flex_none().bg(palette::frame_fill(cx)).child(sidebar))
                    .child(div().flex_1().min_w_0().h_full().child(pane)),
            )
            .children(Root::render_notification_layer(window, cx))
    }
}

/// The opening size: the default, cut down to fit the screen with a margin, never below the minimum size.
fn opening_size(screen: Option<(f32, f32)>) -> (f32, f32) {
    let Some((screen_width, screen_height)) = screen else { return DEFAULT_WINDOW_SIZE };
    let fit = |wanted: f32, available: f32, minimum: f32| wanted.min(available - SCREEN_MARGIN).max(minimum);
    (
        fit(DEFAULT_WINDOW_SIZE.0, screen_width, MINIMUM_WINDOW_SIZE.0),
        fit(DEFAULT_WINDOW_SIZE.1, screen_height, MINIMUM_WINDOW_SIZE.1),
    )
}

fn main() {
    let Some(_instance_guard) = single_instance::acquire() else { return };

    Application::new().with_assets(icons::WindowAssets).run(|cx: &mut App| {
        gpui_component::init(cx);
        rust_i18n::set_locale(store::language::apply_setting(&store::Settings::load()).code());
        if let Err(error) =
            cx.text_system().add_fonts(GEIST_FONTS.iter().map(|bytes| std::borrow::Cow::Borrowed(*bytes)).collect())
        {
            eprintln!("could not load Geist, using the system font: {error}");
        }
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let work_area = cx.primary_display().map(|display| display.bounds().size);
        let (width, height) = opening_size(work_area.map(|area| (f32::from(area.width), f32::from(area.height))));
        let bounds = Bounds::centered(None, size(px(width), px(height)), cx);
        let mut titlebar = TitleBar::title_bar_options();
        titlebar.title = Some(SharedString::from("Foldline Settings"));
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(titlebar),
            window_background: palette::window_background(),
            window_min_size: Some(size(px(MINIMUM_WINDOW_SIZE.0), px(MINIMUM_WINDOW_SIZE.1))),
            ..Default::default()
        };
        let opened = cx.open_window(options, |window, cx| {
            Theme::sync_system_appearance(Some(window), cx);
            palette::apply(cx);
            let view = cx.new(|_| SettingsWindow {
                selected_page_index: 0,
                pages: sections::all(),
                built_for: store::language::current(),
            });
            let view_to_refresh = view.clone();
            window
                .observe_window_appearance(move |window, cx| {
                    Theme::sync_system_appearance(Some(window), cx);
                    palette::apply(cx);
                    view_to_refresh.update(cx, |_, cx| cx.notify());
                })
                .detach();
            cx.new(|cx| Root::new(view, window, cx))
        });
        if let Err(error) = opened {
            eprintln!("could not open the settings window: {error}");
            cx.quit();
            return;
        }
        cx.activate(true);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_window_opens_at_the_default_size_on_a_large_screen_and_shrinks_on_a_small_one() {
        assert_eq!(opening_size(Some((2560.0, 1440.0))), DEFAULT_WINDOW_SIZE);
        assert_eq!(opening_size(Some((1280.0, 720.0))), (1040.0, 640.0));
        assert_eq!(opening_size(Some((800.0, 600.0))), (720.0, 520.0));
        assert_eq!(opening_size(Some((640.0, 400.0))), MINIMUM_WINDOW_SIZE);
        assert_eq!(opening_size(None), DEFAULT_WINDOW_SIZE);
    }
}
