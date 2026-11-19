//! The About page: the app card, the links and the files the notch keeps.

mod licenses;
mod paths;
pub(super) mod widgets;

use crate::i18n::tr;
use std::{path::Path, rc::Rc};

use gpui::{AnyElement, App, Window, div, prelude::*, px};
use gpui_component::{IconName, WindowExt as _, notification::Notification};

use crate::{
    layout::{Group, Item, Page},
    links::{RELEASES_URL, REPOSITORY_URL},
};
use paths::{DataPaths, Opening};
use widgets::Handler;

const STORE_PRODUCT_ID: &str = "9NGTV96SZDS7";

/// What a row's button does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    /// Open an address in the default browser.
    Link(&'static str),
    Licenses,
    SettingsFile,
    DataFolder,
}

/// One row: title and description at the left, a pill button at the right.
struct Row {
    title: &'static str,
    description: &'static str,
    button: &'static str,
    action: Action,
    /// Drawn before the button's text.
    icon: Option<IconName>,
}

fn link_rows() -> Vec<Row> {
    vec![
        Row {
            title: tr!("about.whats_new"),
            description: tr!("about.whats_new_note"),
            button: tr!("about.open"),
            action: Action::Link(RELEASES_URL),
            icon: Some(IconName::ExternalLink),
        },
        Row {
            title: tr!("about.source"),
            description: tr!("about.source_note"),
            button: tr!("about.github"),
            action: Action::Link(REPOSITORY_URL),
            icon: Some(IconName::GitHub),
        },
        Row {
            title: tr!("about.licenses"),
            description: tr!("about.licenses_note"),
            button: tr!("about.view"),
            action: Action::Licenses,
            icon: None,
        },
    ]
}

fn file_rows() -> Vec<Row> {
    vec![
        Row {
            title: tr!("about.settings_file"),
            description: tr!("about.settings_file_note"),
            button: tr!("about.open"),
            action: Action::SettingsFile,
            icon: Some(IconName::ExternalLink),
        },
        Row {
            title: tr!("about.data_folder"),
            description: tr!("about.data_folder_note"),
            button: tr!("about.open"),
            action: Action::DataFolder,
            icon: Some(IconName::ExternalLink),
        },
    ]
}

/// Opens what `opening` names, or says in the window that nothing is saved yet.
fn perform(opening: Opening, window: &mut Window, cx: &mut App) {
    match opening {
        Opening::File(path) | Opening::Folder(path) => cx.open_with_system(&path),
        Opening::Nothing => window.push_notification(Notification::warning(tr!("about.nothing_saved")), cx),
    }
}

/// Picks what to open from the data paths and a check for what exists.
type ChooseOpening = fn(&DataPaths, fn(&Path) -> bool) -> Opening;

fn open_data(choose: ChooseOpening, window: &mut Window, cx: &mut App) {
    let opening = DataPaths::from_environment().map_or(Opening::Nothing, |paths| choose(&paths, Path::exists));
    perform(opening, window, cx);
}

fn handler_for(action: Action) -> Handler {
    match action {
        Action::Link(url) => widgets::open_url_handler(url),
        Action::Licenses => Rc::new(licenses::open_licenses),
        Action::SettingsFile => Rc::new(|window, cx| open_data(paths::settings_opening, window, cx)),
        Action::DataFolder => Rc::new(|window, cx| open_data(paths::folder_opening, window, cx)),
    }
}

/// The Microsoft Store address of the app, or nothing while there is no product id.
fn store_address(product_id: &str) -> Option<String> {
    (!product_id.is_empty()).then(|| format!("ms-windows-store://pdp/?productid={product_id}"))
}

/// Opens the app's Store page; none while `STORE_PRODUCT_ID` is empty, so the link stays hidden.
fn open_store() -> Option<Handler> {
    let address = store_address(STORE_PRODUCT_ID)?;
    Some(Rc::new(move |_window, cx| cx.open_url(&address)))
}

fn items(rows: Vec<Row>) -> Vec<Item> {
    rows.into_iter()
        .map(|row| {
            let Row { title, description, button, action, icon } = row;
            let control = move |_window: &mut Window, cx: &mut App| -> AnyElement {
                widgets::pill(title, button, icon.clone(), handler_for(action), cx)
            };
            Item::new(title, Rc::new(control)).description(description)
        })
        .collect()
}

pub fn page() -> Page {
    let card = Item::picture_row(|_window, cx| {
        div().px(px(2.0)).py(px(4.0)).child(widgets::app_card(open_store(), cx)).into_any_element()
    });
    Page::plain(
        tr!("about.title"),
        vec![Group::new(vec![card]), Group::new(items(link_rows())), Group::new(items(file_rows()))],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn titles(rows: &[Row]) -> Vec<&'static str> {
        rows.iter().map(|row| row.title).collect()
    }

    #[test]
    fn the_link_rows_are_whats_new_source_code_and_licenses() {
        assert_eq!(titles(&link_rows()), ["What's new", "Source code", "Licenses"]);
    }

    #[test]
    fn the_file_rows_are_the_settings_file_and_the_data_folder() {
        assert_eq!(titles(&file_rows()), ["Settings file", "Data folder"]);
    }

    #[test]
    fn whats_new_opens_the_release_notes_and_the_source_button_opens_the_repository() {
        let rows = link_rows();
        assert_eq!(rows[0].action, Action::Link("https://fold.encure.id/releases"));
        assert_eq!(rows[1].action, Action::Link("https://github.com/hibatillah/foldline"));
    }

    #[test]
    fn the_source_button_carries_the_github_icon() {
        assert!(matches!(link_rows()[1].icon, Some(IconName::GitHub)));
    }

    #[test]
    fn an_empty_product_id_gives_no_store_address() {
        assert_eq!(store_address(""), None);
    }

    #[test]
    fn a_product_id_gives_the_store_page_address() {
        assert_eq!(store_address("9ABC123DEF45").as_deref(), Some("ms-windows-store://pdp/?productid=9ABC123DEF45"));
    }

    #[test]
    fn the_settings_file_row_names_the_real_file() {
        assert!(file_rows()[0].description.starts_with("settings.txt"));
    }
}
