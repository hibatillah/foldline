#![windows_subsystem = "windows"]

mod card;
mod field;
mod layout;
mod location;
mod notify;
mod picture;
#[cfg(test)]
mod pre_release;
mod sections;
mod single_instance;
mod startup;
mod version;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use gpui::{
    App, Application, Bounds, Context, Image, ImageFormat, SharedString, Window, WindowBackgroundAppearance,
    WindowBounds, WindowOptions, div, img, prelude::*, px, rgba, size, transparent_black,
};
use gpui_component::{
    ActiveTheme as _, IconName, Root, Theme, TitleBar, WindowExt as _, h_flex,
    notification::Notification,
    sidebar::{Sidebar, SidebarMenu, SidebarMenuItem},
    v_flex,
};

use layout::Page;

const SIDEBAR_WIDTH: f32 = 200.0;
const MINIMUM_WINDOW_SIZE: (f32, f32) = (720.0, 480.0);
const DEFAULT_WINDOW_SIZE: (f32, f32) = (1140.0, 720.0);
/// Space between two sidebar items; the component's default is 8.
const SIDEBAR_ITEM_GAP: f32 = 5.0;
/// Installed beside `notch-settings.exe`; the sidebar's bottom link opens it.
const LICENSES_FILE: &str = "THIRD-PARTY-LICENSES.txt";
/// How many folders above the program's own the licenses file is looked for (`run/` and `target/.../release` sit below the repository root).
const LICENSES_SEARCH_LEVELS: usize = 6;
const LICENSES_LABEL: &str = "Licenses";
const LOGO_SIZE: f32 = 22.0;
// The notch's own font files (SIL Open Font License, assets/fonts/Geist-LICENSE.txt).
const GEIST_FONTS: [&[u8]; 2] = [
    include_bytes!("../../../assets/fonts/Geist-Regular.ttf"),
    include_bytes!("../../../assets/fonts/Geist-Medium.ttf"),
];
const FONT_FAMILY: &str = "Geist";
// A fixed strip at the top: the sidebar and the content start below it so nothing sits under the window buttons.
const HEADER_HEIGHT: f32 = 34.0;

/// The three layers of the window plus the greys that sit on them, all opaque.
///
/// The content is the darkest layer; the sidebar and the cards in the content share one lighter gray; an input and a
/// selected option sit one step above the card.
struct Palette {
    sidebar: u32,
    content: u32,
    card: u32,
    input: u32,
    border: u32,
    divider: u32,
    option: u32,
    option_selected: u32,
    option_border: u32,
    title: u32,
    // Text in the content and in the sidebar differ because the sidebar is the darker surface.
    muted: u32,
    sidebar_muted: u32,
}

fn palette(cx: &App) -> Palette {
    if cx.theme().is_dark() {
        Palette {
            sidebar: 0x1c1c1cff,
            content: 0x0e0e0eff,
            card: 0x1c1c1cff,
            input: 0x262626ff,
            border: 0x2c2c2cff,
            divider: 0x262626ff,
            option: 0x141414ff,
            option_selected: 0x2a2a2aff,
            option_border: 0x2c2c2cff,
            title: 0xffffffff,
            muted: 0xa1a1a1ff,
            sidebar_muted: 0x9a9a9aff,
        }
    } else {
        Palette {
            sidebar: 0xffffffff,
            content: 0xe8e8e8ff,
            card: 0xffffffff,
            input: 0xf3f3f3ff,
            border: 0xd9d9d9ff,
            divider: 0xebebebff,
            option: 0xf6f6f6ff,
            option_selected: 0xe9e9e9ff,
            option_border: 0xdcdcdcff,
            title: 0x0a0a0aff,
            muted: 0x666666ff,
            sidebar_muted: 0x777777ff,
        }
    }
}

/// The field's own background, lighter than the card it sits in.
pub fn input_background(cx: &App) -> gpui::Hsla {
    rgba(palette(cx).input).into()
}

/// The line between two rows of a card.
pub fn row_divider(cx: &App) -> gpui::Hsla {
    rgba(palette(cx).divider).into()
}

/// The fill of an option card: lighter when selected, darker than the card around it otherwise.
pub fn option_fill(selected: bool, cx: &App) -> gpui::Hsla {
    let colors = palette(cx);
    rgba(if selected { colors.option_selected } else { colors.option }).into()
}

/// The outline of an option card: bright when selected, quiet otherwise.
pub fn option_border(selected: bool, cx: &App) -> gpui::Hsla {
    if selected { cx.theme().foreground.opacity(0.85) } else { rgba(palette(cx).option_border).into() }
}

/// Muted text in the sidebar, a different grey from the content's because the surface is darker.
pub fn sidebar_muted(cx: &App) -> gpui::Hsla {
    rgba(palette(cx).sidebar_muted).into()
}

/// Section titles: full foreground, white in the dark theme.
pub fn section_title(cx: &App) -> gpui::Hsla {
    cx.theme().foreground
}

/// Sidebar, content and cards get their own greys; the title bar is clear so the content layer runs to the top edge.
fn apply_palette(cx: &mut App) {
    let colors = palette(cx);
    let theme = Theme::global_mut(cx);
    theme.font_family = FONT_FAMILY.into();
    theme.sidebar = rgba(colors.sidebar).into();
    theme.sidebar_border = transparent_black();
    theme.background = rgba(colors.content).into();
    theme.foreground = rgba(colors.title).into();
    theme.group_box = rgba(colors.card).into();
    theme.accordion = rgba(colors.card).into();
    theme.accordion_hover = rgba(colors.input).into();
    theme.border = rgba(colors.border).into();
    theme.muted_foreground = rgba(colors.muted).into();
    theme.title_bar = transparent_black();
    theme.title_bar_border = transparent_black();
}

struct SettingsWindow {
    selected_page_index: usize,
    pages: Vec<Page>,
    logo: Arc<Image>,
}

impl Render for SettingsWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        let menu = SidebarMenu::new().gap(px(SIDEBAR_ITEM_GAP)).children(self.pages.iter().enumerate().map(
            |(page_index, page)| {
                let view = view.clone();
                SidebarMenuItem::new(page.title).active(page_index == self.selected_page_index).on_click(
                    move |_event, _window, cx| {
                        view.update(cx, |this, cx| {
                            this.selected_page_index = page_index;
                            cx.notify();
                        });
                    },
                )
            },
        ));
        let version_label = format!("v{}", version::displayed_version());
        let brand =
            h_flex().gap_2().items_center().child(img(self.logo.clone()).size(px(LOGO_SIZE)).flex_none()).child(
                h_flex()
                    .gap_1()
                    .items_baseline()
                    .min_w_0()
                    .child(div().flex_none().font_weight(gpui::FontWeight::SEMIBOLD).child("Notch"))
                    .child(
                        div()
                            .min_w_0()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .text_xs()
                            .text_color(sidebar_muted(cx))
                            .child(version_label),
                    ),
            );
        let licenses = div().w_full().child(
            SidebarMenu::new().child(
                SidebarMenuItem::new(LICENSES_LABEL)
                    .icon(IconName::File)
                    .on_click(|_event, window, cx| open_licenses(window, cx)),
            ),
        );
        let sidebar = Sidebar::left().w(px(SIDEBAR_WIDTH)).collapsed(false).header(brand).child(menu).footer(licenses);
        let content = div()
            .id("content")
            .flex_1()
            .min_w_0()
            .h_full()
            .overflow_y_scroll()
            .px_6()
            .pb_6()
            .pt(px(HEADER_HEIGHT + 8.0))
            .child(self.pages[self.selected_page_index].render(window, cx));

        v_flex()
            .relative()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(h_flex().size_full().child(sidebar).child(content))
            .child(div().absolute().top_0().left_0().right_0().child(TitleBar::new()))
            .children(Root::render_notification_layer(window, cx))
    }
}

/// Looks for the licenses file in `start_folder`, then in each parent folder, up to `LICENSES_SEARCH_LEVELS` levels up.
fn find_licenses_file(start_folder: &Path, file_exists: impl Fn(&Path) -> bool) -> Option<PathBuf> {
    start_folder
        .ancestors()
        .take(LICENSES_SEARCH_LEVELS + 1)
        .map(|folder| folder.join(LICENSES_FILE))
        .find(|candidate| file_exists(candidate))
}

/// Opens the license notices, or tells the user in the window when the file cannot be found.
fn open_licenses(window: &mut Window, cx: &mut App) {
    let found =
        std::env::current_exe().ok().and_then(|exe| find_licenses_file(exe.parent()?, |candidate| candidate.is_file()));
    match found {
        Some(path) => cx.open_with_system(&path),
        None => window.push_notification(Notification::warning(format!("{LICENSES_FILE} was not found.")), cx),
    }
}

fn main() {
    let Some(_instance_guard) = single_instance::acquire() else { return };

    Application::new().with_assets(gpui_component_assets::Assets).run(|cx: &mut App| {
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

        let bounds = Bounds::centered(None, size(px(DEFAULT_WINDOW_SIZE.0), px(DEFAULT_WINDOW_SIZE.1)), cx);
        let mut titlebar = TitleBar::title_bar_options();
        titlebar.title = Some(SharedString::from("Notch Settings"));
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(titlebar),
            window_background: WindowBackgroundAppearance::Opaque,
            window_min_size: Some(size(px(MINIMUM_WINDOW_SIZE.0), px(MINIMUM_WINDOW_SIZE.1))),
            ..Default::default()
        };
        let opened = cx.open_window(options, |window, cx| {
            Theme::sync_system_appearance(Some(window), cx);
            apply_palette(cx);
            window
                .observe_window_appearance(|window, cx| {
                    Theme::sync_system_appearance(Some(window), cx);
                    apply_palette(cx);
                })
                .detach();
            let logo = Arc::new(Image::from_bytes(
                ImageFormat::Png,
                include_bytes!("../../../assets/branding/logo.png").to_vec(),
            ));
            let view = cx.new(|_| SettingsWindow { selected_page_index: 0, pages: sections::all(), logo });
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
    fn the_licenses_file_next_to_the_program_wins_over_one_further_up() {
        let near = Path::new("repo/run").join(LICENSES_FILE);
        let far = Path::new("repo").join(LICENSES_FILE);
        let found = find_licenses_file(Path::new("repo/run"), |path| path == near || path == far);
        assert_eq!(found, Some(near));
    }

    #[test]
    fn the_search_walks_up_to_the_repository_root_from_the_build_folder() {
        let root_file = Path::new("repo").join(LICENSES_FILE);
        let found =
            find_licenses_file(Path::new("repo/target/x86_64-pc-windows-msvc/release"), |path| path == root_file);
        assert_eq!(found, Some(root_file));
    }

    #[test]
    fn the_search_stops_after_the_level_limit_and_reports_nothing() {
        let too_far = Path::new("a").join(LICENSES_FILE);
        let start = Path::new("a/b/c/d/e/f/g/h");
        assert_eq!(find_licenses_file(start, |path| path == too_far), None);
        assert_eq!(find_licenses_file(start, |_| false), None);
    }
}
