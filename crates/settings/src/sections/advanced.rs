//! The Advanced page: the debug log.

use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

use gpui::{AnyElement, App, Window, div, prelude::*, px};
use gpui_component::{ActiveTheme as _, h_flex, v_flex};
use store::Settings;

use super::about::widgets::{Handler, pill};
use crate::i18n::tr;
use crate::{
    field::{self, Kind},
    layout::{Group, Item, Page},
};

const DEBUG_LOG_KEY: &str = "advanced.debug_log";
const PATH_FIELD_WIDTH: f32 = 230.0;
/// The name the save dialog offers for a new log file.
const SUGGESTED_LOG_NAME: &str = "foldline-debug.log";
const SAMPLE_FONT: &str = "Consolas";
const SAMPLE_TEXT_SIZE: f32 = 12.0;
const SAMPLE_LINE_HEIGHT: f32 = 19.0;
/// Three lines as `log::debug` in the notch writes them: milliseconds since 1970, then the message.
const SAMPLE_LINES: [(&str, &str); 3] = [
    ("1763107498120 ms", "drop target ready"),
    ("1763107498410 ms", "frame timer started"),
    ("1763107512006 ms", "frame timer stopped"),
];

/// Where the save dialog opens: the folder of the log already set, else the notch's data folder, else the user folder.
fn browse_directory(stored: Option<&str>, data_folder: Option<PathBuf>, home: Option<PathBuf>) -> PathBuf {
    stored
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .and_then(|path| Path::new(path).parent().map(Path::to_path_buf))
        .filter(|folder| !folder.as_os_str().is_empty())
        .or(data_folder)
        .or(home)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Asks Windows' save dialog for a log file and stores the path picked; a cancel stores nothing.
fn browse(_window: &mut Window, cx: &mut App) {
    let stored = Settings::load().get_string(DEBUG_LOG_KEY).map(str::to_owned);
    let data_folder = store::data_directory();
    let home = std::env::var_os("USERPROFILE").map(PathBuf::from);
    let directory = browse_directory(stored.as_deref(), data_folder, home);
    let picked = cx.prompt_for_new_path(&directory, Some(SUGGESTED_LOG_NAME));
    cx.spawn(async move |cx| {
        if let Ok(Ok(Some(path))) = picked.await {
            let _ = cx.update(|cx| field::write(DEBUG_LOG_KEY, &path.to_string_lossy(), cx));
        }
    })
    .detach();
}

fn log_file_control(window: &mut Window, cx: &mut App) -> AnyElement {
    let field =
        field::input(DEBUG_LOG_KEY, Kind::Path { width: PATH_FIELD_WIDTH }, tr!("advanced.debug_log_placeholder"));
    let handler: Handler = Rc::new(browse);
    h_flex()
        .gap(px(8.0))
        .items_center()
        .child(field(window, cx))
        .child(pill("browse-log", tr!("advanced.browse"), None, handler, cx))
        .into_any_element()
}

/// Three sample lines of the log, then what an empty path means.
fn log_sample(cx: &App) -> AnyElement {
    let muted = cx.theme().muted_foreground;
    let lines = SAMPLE_LINES.iter().map(|(time, event)| {
        h_flex()
            .gap(px(14.0))
            .child(div().text_color(muted.opacity(0.7)).child(*time))
            .child(div().text_color(cx.theme().foreground.opacity(0.75)).child(*event))
    });
    v_flex()
        .gap(px(7.0))
        .child(
            v_flex()
                .px(px(12.0))
                .py(px(10.0))
                .rounded(px(9.0))
                .border_1()
                .border_color(crate::palette::row_divider(cx))
                .bg(crate::palette::input_background(cx))
                .font_family(SAMPLE_FONT)
                .text_size(px(SAMPLE_TEXT_SIZE))
                .line_height(px(SAMPLE_LINE_HEIGHT))
                .children(lines),
        )
        .child(div().text_size(px(12.0)).text_color(muted).child(tr!("advanced.debug_log_sample_note")))
        .into_any_element()
}

pub fn page() -> Page {
    Page::plain(
        tr!("advanced.title"),
        vec![Group::titled(
            tr!("advanced.group_troubleshooting"),
            vec![
                Item::new(tr!("advanced.debug_log"), Rc::new(log_file_control))
                    .tag(tr!("advanced.after_restart"))
                    .description(tr!("advanced.debug_log_note")),
                Item::picture_row(|_window, cx| log_sample(cx)),
            ],
        )],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_dialog_opens_in_the_folder_of_the_log_already_set() {
        let folder = browse_directory(Some(r"D:\logs\notch.log"), Some(PathBuf::from(r"C:\data")), None);
        assert_eq!(folder, PathBuf::from(r"D:\logs"));
    }

    #[test]
    fn without_a_log_the_dialog_opens_in_the_data_folder_then_the_user_folder() {
        let data = PathBuf::from(r"C:\Users\me\AppData\Roaming\Foldline");
        let home = PathBuf::from(r"C:\Users\me");
        assert_eq!(browse_directory(None, Some(data.clone()), Some(home.clone())), data);
        assert_eq!(browse_directory(Some("  "), None, Some(home.clone())), home);
        assert_eq!(browse_directory(Some("notch.log"), Some(data.clone()), Some(home)), data);
    }

    #[test]
    fn the_page_holds_only_troubleshooting() {
        let settings_page = page();
        let titles = settings_page.group_titles();
        assert_eq!(titles, ["Troubleshooting"]);
    }
}
