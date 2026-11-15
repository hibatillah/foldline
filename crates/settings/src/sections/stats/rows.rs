//! The device lists of the System stats page: one row per disk or graphics card, with a tick that shows it on the page.

use gpui::{AnyElement, App, ElementId, SharedString, Window, div, prelude::*, px};
use gpui_component::{ActiveTheme as _, Disableable as _, checkbox::Checkbox, h_flex, v_flex};
use store::Settings;

use super::{
    Availability, Kind, availability,
    disk_info::{self, DiskInfo},
    list_text, selected, toggled,
};
use crate::{field, layout::block_frame};

const ROW_PADDING_X: f32 = 14.0;
const ROW_PADDING_Y: f32 = 9.0;
const ROW_MIN_HEIGHT: f32 = 50.0;
const BAR_WIDTH: f32 = 92.0;
const BAR_HEIGHT: f32 = 5.0;
const PERCENT_WIDTH: f32 = 36.0;
/// A device that cannot be ticked because the page is full is dimmed this far.
const UNAVAILABLE_OPACITY: f32 = 0.45;

fn muted_message(text: &'static str, cx: &App) -> AnyElement {
    block_frame(cx)
        .px(px(ROW_PADDING_X))
        .py(px(ROW_PADDING_Y + 3.0))
        .text_sm()
        .text_color(cx.theme().muted_foreground)
        .child(text)
        .into_any_element()
}

fn usage_bar(used_percent: u32, cx: &App) -> AnyElement {
    h_flex()
        .flex_none()
        .gap(px(12.0))
        .items_center()
        .child(div().w(px(BAR_WIDTH)).h(px(BAR_HEIGHT)).rounded_full().bg(cx.theme().foreground.opacity(0.12)).child(
            div().h_full().w(gpui::relative(used_percent as f32 / 100.0)).rounded_full().bg(crate::palette::accent(cx)),
        ))
        .child(
            div()
                .w(px(PERCENT_WIDTH))
                .text_right()
                .text_sm()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(crate::palette::heading(cx))
                .child(SharedString::from(format!("{used_percent}%"))),
        )
        .into_any_element()
}

/// "Local disk · 512 GB".
fn disk_subtitle(info: DiskInfo) -> String {
    format!("{} · {}", info.kind.label(), disk_info::size_text(info.total_bytes))
}

fn row(
    kind: Kind,
    name: &str,
    checked: bool,
    available: bool,
    last: bool,
    info: Option<DiskInfo>,
    cx: &App,
) -> AnyElement {
    let device = name.to_owned();
    let tick = Checkbox::new(ElementId::Name(SharedString::from(format!("{}-{name}", kind.id_prefix))))
        .checked(checked)
        .disabled(!available)
        .on_click(move |on: &bool, _window, cx| {
            let settings = Settings::load();
            let known = match availability(&settings, kind) {
                Availability::Devices(known) => known,
                _ => return,
            };
            if let Some(names) = toggled(&known, settings.get_string(kind.key), kind, &device, *on) {
                field::write(kind.key, &list_text(&names, kind), cx);
            }
        });
    let text = v_flex()
        .flex_1()
        .min_w_0()
        .gap(px(1.0))
        .child(div().text_base().font_weight(gpui::FontWeight::MEDIUM).child(SharedString::from(name.to_owned())))
        .children(info.map(|info| div().text_sm().text_color(cx.theme().muted_foreground).child(disk_subtitle(info))));
    h_flex()
        .w_full()
        .min_h(px(ROW_MIN_HEIGHT))
        .px(px(ROW_PADDING_X))
        .py(px(ROW_PADDING_Y))
        .gap(px(12.0))
        .items_center()
        .when(!last, |row| row.border_b_1().border_color(crate::palette::row_divider(cx)))
        .when(!available, |row| row.opacity(UNAVAILABLE_OPACITY))
        .child(tick)
        .child(text)
        .children(info.map(|info| usage_bar(info.used_percent, cx)))
        .into_any_element()
}

/// The block of rows for one kind of device, or a muted line while the notch has not listed any.
pub fn list(kind: Kind, _window: &mut Window, cx: &mut App) -> AnyElement {
    let settings = Settings::load();
    let known = match availability(&settings, kind) {
        Availability::Unlisted => return muted_message((kind.unlisted)(), cx),
        Availability::NoneFound => return muted_message((kind.none_found)(), cx),
        Availability::Devices(known) => known,
    };
    let chosen = selected(&known, settings.get_string(kind.key), kind);
    let full = chosen.len() >= kind.limit;
    let count = known.len();
    let rows = known.iter().enumerate().map(|(index, name)| {
        let checked = chosen.contains(name);
        let info = if kind.separator == ',' { disk_info::read(name) } else { None };
        row(kind, name, checked, checked || !full, index + 1 == count, info, cx)
    });
    block_frame(cx).children(rows).into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::tr;

    #[test]
    fn a_disk_subtitle_names_the_kind_and_the_size() {
        let info = DiskInfo { kind: disk_info::DriveKind::Local, total_bytes: 512_000_000_000, used_percent: 40 };
        assert_eq!(disk_subtitle(info), "Local disk · 512 GB");
    }

    #[test]
    fn the_pick_note_counts_the_ticked_devices() {
        assert_eq!(tr!("stats.pick_note", limit = 3, chosen = 2), "Pick up to 3 · 2 of 3");
    }
}
