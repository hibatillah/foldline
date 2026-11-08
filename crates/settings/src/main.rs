#![windows_subsystem = "windows"]

mod brand;
mod card;
mod field;
mod header;
mod icons;
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
mod version;

use std::rc::Rc;

use gpui::{
    App, Application, Bounds, Context, SharedString, Window, WindowBounds, WindowOptions, div, img, prelude::*, px,
    size,
};
use gpui_component::{ActiveTheme as _, Root, Theme, TitleBar, h_flex, v_flex};

use sidebar::{NavEntry, SelectHandler};

const SIDEBAR_WIDTH: f32 = 200.0;
const MINIMUM_WINDOW_SIZE: (f32, f32) = (720.0, 480.0);
const DEFAULT_WINDOW_SIZE: (f32, f32) = (1040.0, 660.0);
/// Space kept free around the window when the screen is smaller than the default size (taskbar and edges).
const SCREEN_MARGIN: f32 = 80.0;
const LOGO_SIZE: f32 = 28.0;
// The notch's own font files (SIL Open Font License, assets/fonts/Geist-LICENSE.txt).
/// The one scale knob: gpui-component's Root sets the window's rem size to this, so every rem-based size (text, `px_*`, `gap_*`, controls) follows it. 16 is the library default.
const UI_FONT_SIZE: f32 = 14.7;
const GEIST_FONTS: [&[u8]; 2] = [
    include_bytes!("../../../assets/fonts/Geist-Regular.ttf"),
    include_bytes!("../../../assets/fonts/Geist-Medium.ttf"),
];
const FONT_FAMILY: &str = "Geist";
/// The frame that shows around the content card: above it, to its right and below it.
const FRAME_PADDING: f32 = 8.0;
const CARD_RADIUS: f32 = 8.0;

struct SettingsWindow {
    selected_page_index: usize,
    pages: Vec<NavEntry>,
}

impl Render for SettingsWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        let version_label = format!("v{}", version::displayed_version());
        let brand = h_flex()
            .gap_3()
            .items_center()
            .child(img(brand::tile_image()).size(px(LOGO_SIZE)).flex_none())
            .child(
                h_flex()
                    .gap_1()
                    .items_baseline()
                    .min_w_0()
                    .child(div().flex_none().text_base().font_weight(gpui::FontWeight::SEMIBOLD).child("Notch"))
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
        let content =
            div().id("content").flex_1().min_h_0().overflow_y_scroll().px_6().py_6().child(selected.render(window, cx));
        let card = v_flex()
            .size_full()
            .rounded(px(CARD_RADIUS))
            .border_1()
            .border_color(palette::card_border(cx))
            .bg(palette::card(cx))
            .overflow_hidden()
            .child(header::render(selected.title, window, cx))
            .child(content);

        v_flex()
            .relative()
            .size_full()
            .bg(palette::frame_fill(cx))
            .text_color(cx.theme().foreground)
            .child(
                h_flex().size_full().child(sidebar).child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .h_full()
                        .pt(px(FRAME_PADDING))
                        .pr(px(FRAME_PADDING))
                        .pb(px(FRAME_PADDING))
                        .child(card),
                ),
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
        titlebar.title = Some(SharedString::from("Notch Settings"));
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
            let view = cx.new(|_| SettingsWindow { selected_page_index: 0, pages: sections::all() });
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
