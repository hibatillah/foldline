//! The chosen-screen item of the General page: a dropdown of the connected screens that writes `notch.monitor_id`.

use std::rc::Rc;

use gpui::{AppContext as _, Entity, SharedString, Subscription, div, prelude::*, px};
use gpui_component::IndexPath;
use gpui_component::select::{Select, SelectEvent, SelectItem, SelectState};
use store::{Settings, screens::Screen};

use super::general::chosen_is_selected;
use crate::{i18n::tr, layout::Item};

pub const MONITOR_ID_KEY: &str = "notch.monitor_id";
const DROPDOWN_WIDTH: f32 = 280.0;

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
    super::apply(settings, MONITOR_ID_KEY, picked_id);
    true
}

#[derive(Clone, PartialEq, Eq)]
struct ScreenOption {
    id: SharedString,
    label: SharedString,
}

impl SelectItem for ScreenOption {
    type Value = SharedString;

    fn title(&self) -> SharedString {
        self.label.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.id
    }
}

fn options_of(screens: &[Screen]) -> Vec<ScreenOption> {
    let (screen_word, main_word) = (tr!("general.screen_word"), tr!("general.screen_main"));
    screens
        .iter()
        .enumerate()
        .map(|(index, screen)| ScreenOption {
            id: screen.id.clone().into(),
            label: screen_label(index + 1, screen, screen_word, main_word).into(),
        })
        .collect()
}

struct ScreenBox {
    select: Entity<SelectState<Vec<ScreenOption>>>,
    options: Vec<ScreenOption>,
    _subscription: Subscription,
}

fn stored_chosen(screens: &[Screen]) -> ChosenScreen {
    resolve_chosen(Settings::load().get_string(MONITOR_ID_KEY), screens)
}

fn dropdown() -> crate::layout::Render {
    Rc::new(|window, cx| {
        let screens = store::screens::list();
        let options = options_of(&screens);
        let chosen = stored_chosen(&screens);
        let selected_position = match chosen {
            ChosenScreen::Connected(position) => Some(position),
            ChosenScreen::Unset | ChosenScreen::NotConnected => None,
        };
        let state = window.use_keyed_state("chosen-screen", cx, {
            let options = options.clone();
            move |window, cx| {
                let select = cx.new(|cx| SelectState::new(options.clone(), None, window, cx));
                let _subscription = cx.subscribe_in(
                    &select,
                    window,
                    |_box: &mut ScreenBox, _select, event: &SelectEvent<Vec<ScreenOption>>, _window, cx| {
                        let SelectEvent::Confirm(Some(id)) = event else { return };
                        if store_picked_screen(&mut Settings::load(), id) {
                            cx.refresh_windows();
                        }
                    },
                );
                ScreenBox { select, options, _subscription }
            }
        });
        // The list is read on this paint; a screen that came or went changes the items, never the stored id.
        state.update(cx, |screen_box, cx| {
            if screen_box.options != options {
                screen_box.options = options;
                let items = screen_box.options.clone();
                screen_box.select.update(cx, |select, cx| select.set_items(items, window, cx));
            }
            let wanted = selected_position.map(IndexPath::new);
            screen_box.select.update(cx, |select, cx| {
                if select.selected_index(cx) != wanted {
                    select.set_selected_index(wanted, window, cx);
                }
            });
        });
        let placeholder = match chosen {
            ChosenScreen::NotConnected => tr!("general.screen_not_connected"),
            ChosenScreen::Unset | ChosenScreen::Connected(_) => tr!("general.screen_choose"),
        };
        let select = state.read(cx).select.clone();
        div().w(px(DROPDOWN_WIDTH)).child(Select::new(&select).placeholder(placeholder)).into_any_element()
    })
}

/// The row under "Show the notch on"; left out of the page unless "A chosen screen" is selected.
pub fn item() -> Item {
    Item::new(tr!("general.screen_title"), dropdown()).visible_when(|| chosen_is_selected(&Settings::load())).note_when(
        || {
            matches!(stored_chosen(&store::screens::list()), ChosenScreen::NotConnected)
                .then(|| tr!("general.screen_not_connected_note"))
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen(id: &str, model: Option<&str>, is_primary: bool, width: i32, height: i32) -> Screen {
        Screen { id: id.to_owned(), model: model.map(str::to_owned), is_primary, width, height }
    }

    fn temporary_settings(name: &str, text: &str) -> Settings {
        let path = std::env::temp_dir().join(format!("notch-settings-{name}-{}.txt", std::process::id()));
        std::fs::write(&path, text).unwrap();
        Settings::at(Some(path))
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

    fn settings_path(name: &str) -> Option<std::path::PathBuf> {
        Some(std::env::temp_dir().join(format!("notch-settings-{name}-{}.txt", std::process::id())))
    }
}
