//! The About page: the app card, the links and the files the notch keeps.

mod licenses;
mod paths;
pub(super) mod widgets;

use crate::i18n::tr;
use std::{path::Path, rc::Rc};

use gpui::{AnyElement, App, Window, div, prelude::*, px};
use gpui_component::{IconName, WindowExt as _, notification::Notification};

use crate::layout::{Group, Item, Page};
use paths::{DataPaths, Opening};
use widgets::Handler;

/// The release notes of this version. Empty until the site exists; while it is empty the button is inert.
const WHATS_NEW_URL: &str = "";

const STORE_PRODUCT_ID: &str = "9NGTV96SZDS7";

/// What a row's button does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    /// Open an address in the default browser; an empty one does nothing.
    Link(&'static str),
    Licenses,
    SettingsFile,
    DataFolder,
}

impl Action {
    fn is_inert(self) -> bool {
        matches!(self, Action::Link(url) if url.is_empty())
    }
}

/// One row: title and description at the left, a pill button at the right.
struct Row {
    title: &'static str,
    description: &'static str,
    button: &'static str,
    action: Action,
    external: bool,
}

fn link_rows() -> Vec<Row> {
    vec![
        Row {
            title: tr!("about.whats_new"),
            description: tr!("about.whats_new_note"),
            button: tr!("about.open"),
            action: Action::Link(WHATS_NEW_URL),
            external: true,
        },
        Row {
            title: tr!("about.licenses"),
            description: tr!("about.licenses_note"),
            button: tr!("about.view"),
            action: Action::Licenses,
            external: false,
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
            external: true,
        },
        Row {
            title: tr!("about.data_folder"),
            description: tr!("about.data_folder_note"),
            button: tr!("about.open"),
            action: Action::DataFolder,
            external: true,
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

fn handler_for(action: Action) -> Option<Handler> {
    if action.is_inert() {
        return None;
    }
    let handler: Handler = match action {
        Action::Link(url) => Rc::new(move |_window, cx| cx.open_url(url)),
        Action::Licenses => Rc::new(licenses::open_licenses),
        Action::SettingsFile => Rc::new(|window, cx| open_data(paths::settings_opening, window, cx)),
        Action::DataFolder => Rc::new(|window, cx| open_data(paths::folder_opening, window, cx)),
    };
    Some(handler)
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
            let Row { title, description, button, action, external } = row;
            let control = move |_window: &mut Window, cx: &mut App| -> AnyElement {
                let icon = external.then_some(IconName::ExternalLink);
                widgets::pill(title, button, icon, handler_for(action), cx)
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
        vec![
            Group::new(vec![card]),
            Group::new(items(link_rows())),
            Group::titled(tr!("about.group_files"), items(file_rows())).icon(crate::icons::FOLDER),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn titles(rows: &[Row]) -> Vec<&'static str> {
        rows.iter().map(|row| row.title).collect()
    }

    #[test]
    fn the_link_rows_are_whats_new_and_licenses() {
        assert_eq!(titles(&link_rows()), ["What's new", "Licenses"]);
    }

    #[test]
    fn the_file_rows_are_the_settings_file_and_the_data_folder() {
        assert_eq!(titles(&file_rows()), ["Settings file", "Data folder"]);
    }

    #[test]
    fn an_empty_whats_new_address_makes_its_button_inert() {
        let whats_new = &link_rows()[0];
        assert!(whats_new.action.is_inert());
        assert!(handler_for(whats_new.action).is_none());
    }

    #[test]
    fn the_other_rows_have_a_working_button() {
        for row in link_rows().iter().skip(1).chain(file_rows().iter()) {
            assert!(!row.action.is_inert(), "{}", row.title);
            assert!(handler_for(row.action).is_some(), "{}", row.title);
        }
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
