//! The chosen-screen picker of the General page: the connected screens drawn where they stand on the desktop, scaled
//! down; a click on one writes its id to `notch.monitor_id`.

use gpui::{AnyElement, App, SharedString, div, prelude::*, px, rgba};
use gpui_component::{ActiveTheme as _, tooltip::Tooltip, v_flex};
use store::{Settings, screens::Screen};

use super::monitor::chosen_is_selected;
use crate::{i18n::tr, illustration, layout::Item, picture::color};

pub const MONITOR_ID_KEY: &str = "notch.monitor_id";
/// The drawing's size; the screens are scaled to fit inside it with `MAP_PADDING` around them.
const MAP_HEIGHT: f32 = 132.0;
const MAP_PADDING: f32 = 14.0;
/// The drawing's width for the fit; the drawing itself fills its row and the screens stay centered.
const FIT_WIDTH: f32 = 420.0;
/// The space between two screens that touch on the desktop, so each reads as its own.
const SCREEN_GAP: f32 = 4.0;
const SCREEN_RADIUS: f32 = 5.0;
const NOTCH_WIDTH: f32 = 22.0;
const NOTCH_HEIGHT: f32 = 6.0;

/// "Screen 2, LG UltraGear, 1920x1080 (main)": the model only when Windows names one, "(main)" only on the primary.
fn screen_label(position: usize, screen: &Screen, screen_word: &str, main_word: &str) -> String {
    let mut parts = vec![format!("{screen_word} {position}")];
    parts.extend(screen.model.clone());
    parts.push(format!("{}x{}", screen.width, screen.height));
    let label = parts.join(", ");
    if screen.is_primary { format!("{label} ({main_word})") } else { label }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ChosenScreen {
    /// Nothing is stored yet.
    Unset,
    /// The stored id is at this position of the list.
    Connected(usize),
    /// An id is stored but no connected screen has it.
    NotConnected,
}

fn resolve_chosen(stored_id: Option<&str>, screens: &[Screen]) -> ChosenScreen {
    let Some(stored_id) = stored_id.filter(|id| !id.is_empty()) else { return ChosenScreen::Unset };
    screens.iter().position(|screen| screen.id == stored_id).map_or(ChosenScreen::NotConnected, ChosenScreen::Connected)
}

/// Writes the picked id when it differs from the stored one; returns whether it wrote.
fn store_picked_screen(settings: &mut Settings, picked_id: &str) -> bool {
    if settings.get_string(MONITOR_ID_KEY) == Some(picked_id) {
        return false;
    }
    super::super::apply(settings, MONITOR_ID_KEY, picked_id);
    true
}

fn stored_chosen(screens: &[Screen]) -> ChosenScreen {
    resolve_chosen(Settings::load().get_string(MONITOR_ID_KEY), screens)
}

/// A screen's place in the drawing, in pixels from the drawing's top-left corner.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Placed {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
}

/// Scales the screens' desktop rectangles down to fit `width` by `height` less the padding, keeping their proportions
/// and where they stand to each other, and centers the whole arrangement.
fn fit(screens: &[Screen], width: f32, height: f32) -> Vec<Placed> {
    let (Some(min_x), Some(min_y)) =
        (screens.iter().map(|screen| screen.left).min(), screens.iter().map(|screen| screen.top).min())
    else {
        return Vec::new();
    };
    let max_x = screens.iter().map(|screen| screen.left + screen.width).max().unwrap_or(min_x);
    let max_y = screens.iter().map(|screen| screen.top + screen.height).max().unwrap_or(min_y);
    let (desk_width, desk_height) = ((max_x - min_x).max(1) as f32, (max_y - min_y).max(1) as f32);
    let scale = ((width - 2.0 * MAP_PADDING) / desk_width).min((height - 2.0 * MAP_PADDING) / desk_height);
    let offset_x = (width - desk_width * scale) / 2.0;
    let offset_y = (height - desk_height * scale) / 2.0;
    screens
        .iter()
        .map(|screen| Placed {
            left: offset_x + (screen.left - min_x) as f32 * scale,
            top: offset_y + (screen.top - min_y) as f32 * scale,
            width: screen.width as f32 * scale,
            height: screen.height as f32 * scale,
        })
        .collect()
}

fn drawn_screen(index: usize, screen: &Screen, place: Placed, chosen: bool, cx: &App) -> AnyElement {
    let accent = crate::palette::accent(cx);
    let label = screen_label(index + 1, screen, tr!("general.screen_word"), tr!("general.screen_main"));
    let id = screen.id.clone();
    let (fill, border) =
        if chosen { (accent.opacity(0.18), accent) } else { (color(0x26304a), rgba(0xffffff40).into()) };
    let inset = SCREEN_GAP / 2.0;
    div()
        .id(SharedString::from(format!("screen-{}", screen.id)))
        .absolute()
        .left(px(place.left + inset))
        .top(px(place.top + inset))
        .w(px((place.width - SCREEN_GAP).max(8.0)))
        .h(px((place.height - SCREEN_GAP).max(8.0)))
        .rounded(px(SCREEN_RADIUS))
        .border(px(1.5))
        .border_color(border)
        .bg(fill)
        .cursor_pointer()
        .hover(move |style| style.border_color(accent))
        .tooltip(move |window, cx| Tooltip::new(label.clone()).build(window, cx))
        .on_click(move |_event, _window, cx| {
            if store_picked_screen(&mut Settings::load(), &id) {
                cx.refresh_windows();
            }
        })
        .when(chosen, |drawn| {
            drawn.child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .w_full()
                    .flex()
                    .justify_center()
                    .child(div().w(px(NOTCH_WIDTH)).h(px(NOTCH_HEIGHT)).rounded_b(px(3.0)).bg(color(0x000000))),
            )
        })
        .child(
            v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .child(div().text_sm().font_weight(gpui::FontWeight::SEMIBOLD).child(format!("{}", index + 1)))
                .child(
                    div()
                        .text_size(px(10.5))
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("{}x{}", screen.width, screen.height)),
                )
                .when(screen.is_primary, |text| {
                    text.child(div().text_size(px(10.5)).text_color(accent).child(tr!("general.screen_main_badge")))
                }),
        )
        .into_any_element()
}

/// The drawing of the connected screens, read when the page paints; the stored id is never overwritten by painting.
fn screen_map(cx: &App) -> AnyElement {
    let screens = store::screens::list();
    let chosen = stored_chosen(&screens);
    let places = fit(&screens, FIT_WIDTH, MAP_HEIGHT);
    let drawn = screens.iter().zip(places).enumerate().map(|(index, (screen, place))| {
        drawn_screen(index, screen, place, chosen == ChosenScreen::Connected(index), cx)
    });
    let map = div().relative().w(px(FIT_WIDTH)).h(px(MAP_HEIGHT)).children(drawn);
    illustration::scene(MAP_HEIGHT).flex().justify_center().child(map).into_any_element()
}

/// The title, the hint and the drawing; left out of the page unless "A chosen screen" is selected.
pub fn item() -> Item {
    Item::picture_row(|_window, cx| {
        let screens = store::screens::list();
        let hint = match stored_chosen(&screens) {
            ChosenScreen::NotConnected => tr!("general.screen_not_connected_note"),
            ChosenScreen::Unset => tr!("general.screen_choose"),
            ChosenScreen::Connected(_) => tr!("general.screen_pick_note"),
        };
        v_flex()
            .gap(px(8.0))
            .child(
                v_flex()
                    .gap(px(1.0))
                    .child(div().text_base().font_weight(gpui::FontWeight::MEDIUM).child(tr!("general.screen_title")))
                    .child(div().text_sm().text_color(cx.theme().muted_foreground).child(hint)),
            )
            .child(screen_map(cx))
            .into_any_element()
    })
    .visible_when(|| chosen_is_selected(&Settings::load()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen(id: &str, model: Option<&str>, is_primary: bool, width: i32, height: i32) -> Screen {
        Screen { id: id.to_owned(), model: model.map(str::to_owned), is_primary, left: 0, top: 0, width, height }
    }

    fn placed_screen(left: i32, top: i32, width: i32, height: i32) -> Screen {
        Screen { id: format!("{left}-{top}"), model: None, is_primary: left == 0 && top == 0, left, top, width, height }
    }

    fn temporary_settings(name: &str, text: &str) -> Settings {
        let path = std::env::temp_dir().join(format!("foldline-settings-{name}-{}.txt", std::process::id()));
        std::fs::write(&path, text).unwrap();
        Settings::at(Some(path))
    }

    fn settings_path(name: &str) -> Option<std::path::PathBuf> {
        Some(std::env::temp_dir().join(format!("foldline-settings-{name}-{}.txt", std::process::id())))
    }

    #[test]
    fn the_main_screen_is_labelled_with_its_number_size_and_main() {
        let label = screen_label(1, &screen("a", None, true, 1920, 1080), "Screen", "main");
        assert_eq!(label, "Screen 1, 1920x1080 (main)");
    }

    #[test]
    fn another_screen_has_no_main_mark() {
        let label = screen_label(2, &screen("b", None, false, 2560, 1440), "Screen", "main");
        assert_eq!(label, "Screen 2, 2560x1440");
    }

    #[test]
    fn a_screen_with_a_model_name_shows_it_after_the_number() {
        let label = screen_label(2, &screen("b", Some("LG UltraGear"), false, 2560, 1440), "Screen", "main");
        assert_eq!(label, "Screen 2, LG UltraGear, 2560x1440");
    }

    #[test]
    fn a_stored_id_in_the_list_resolves_to_its_position() {
        let screens = [screen("a", None, true, 1, 1), screen("b", None, false, 1, 1)];
        assert_eq!(resolve_chosen(Some("b"), &screens), ChosenScreen::Connected(1));
    }

    #[test]
    fn a_stored_id_missing_from_the_list_shows_not_connected() {
        let screens = [screen("a", None, true, 1, 1)];
        assert_eq!(resolve_chosen(Some("gone"), &screens), ChosenScreen::NotConnected);
    }

    #[test]
    fn no_stored_id_or_an_empty_one_is_unset() {
        let screens = [screen("a", None, true, 1, 1)];
        assert_eq!(resolve_chosen(None, &screens), ChosenScreen::Unset);
        assert_eq!(resolve_chosen(Some(""), &screens), ChosenScreen::Unset);
    }

    #[test]
    fn picking_a_screen_writes_its_id_to_the_monitor_id_key() {
        let mut settings = temporary_settings("pick", "notch.monitor=chosen\n");
        assert!(store_picked_screen(&mut settings, r"\\?\DISPLAY#GSM5B09#1"));
        assert_eq!(Settings::at(settings_path("pick")).get_string(MONITOR_ID_KEY), Some(r"\\?\DISPLAY#GSM5B09#1"));
        let _ = std::fs::remove_file(settings_path("pick").unwrap());
    }

    #[test]
    fn picking_the_screen_already_stored_writes_nothing() {
        let mut settings = temporary_settings("same", "notch.monitor_id=b\n");
        assert!(!store_picked_screen(&mut settings, "b"));
        let _ = std::fs::remove_file(settings_path("same").unwrap());
    }

    #[test]
    fn a_stored_id_missing_from_the_list_stays_in_the_file_after_resolving() {
        let settings = temporary_settings("keep", "notch.monitor_id=gone\n");
        let screens = [screen("a", None, true, 1, 1)];
        assert_eq!(resolve_chosen(settings.get_string(MONITOR_ID_KEY), &screens), ChosenScreen::NotConnected);
        assert_eq!(Settings::at(settings_path("keep")).get_string(MONITOR_ID_KEY), Some("gone"));
        let _ = std::fs::remove_file(settings_path("keep").unwrap());
    }

    #[test]
    fn one_screen_is_drawn_centered_and_as_large_as_the_padding_allows() {
        let places = fit(&[placed_screen(0, 0, 1920, 1080)], 420.0, 132.0);
        let place = places[0];
        assert!((place.height - (132.0 - 2.0 * MAP_PADDING)).abs() < 0.01, "{place:?}");
        assert!((place.left + place.width / 2.0 - 210.0).abs() < 0.01, "{place:?}");
        assert!((place.width / place.height - 1920.0 / 1080.0).abs() < 0.01, "{place:?}");
    }

    #[test]
    fn a_screen_left_of_the_main_one_is_drawn_left_of_it_at_the_same_scale() {
        let screens = [placed_screen(0, 0, 1920, 1080), placed_screen(-2560, -360, 2560, 1440)];
        let places = fit(&screens, 420.0, 132.0);
        let (main, left) = (places[0], places[1]);
        assert!((left.left + left.width - main.left).abs() < 0.01, "the screens touch: {places:?}");
        assert!(left.top < main.top, "the taller screen starts higher: {places:?}");
        assert!((left.width / main.width - 2560.0 / 1920.0).abs() < 0.01, "{places:?}");
    }

    #[test]
    fn no_screens_draw_nothing() {
        assert!(fit(&[], 420.0, 132.0).is_empty());
    }
}
