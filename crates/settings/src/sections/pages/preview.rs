//! "How it looks in Foldline": the open notch's header at the first, a middle and the last page, with one dot per
//! page shown and the long dot on the page you are on.

use gpui::{AnyElement, App, SharedString, div, prelude::*, px, rgba};
use gpui_component::{ActiveTheme as _, h_flex};

use crate::{
    i18n::tr,
    illustration,
    picture::{BATTERY_GREEN, color, ring},
};

const PICTURE_HEIGHT: f32 = 54.0;
const HEADER_INSET: f32 = 10.0;
const HEADER_HEIGHT: f32 = 28.0;
const HEADER_CORNER: f32 = 12.0;
const DOT_HEIGHT: f32 = 4.0;
const DOT_WIDTH: f32 = 4.0;
const CURRENT_DOT_WIDTH: f32 = 13.0;
const DOT_GAP: f32 = 3.0;
const CLOCK_SIZE: f32 = 11.0;
const IDLE_DOT: u32 = 0x5a5a62;

/// Which of the shown pages to draw: the first, the third (or the middle one when there are fewer), and the last.
pub fn pictured_positions(shown_count: usize) -> Vec<usize> {
    let Some(last) = shown_count.checked_sub(1) else { return Vec::new() };
    let middle = if shown_count >= 5 { 2 } else { last / 2 };
    let mut positions = vec![0, middle, last];
    positions.dedup();
    positions
}

fn header(current: usize, shown_count: usize) -> AnyElement {
    let dots = h_flex().gap(px(DOT_GAP)).items_center().children((0..shown_count).map(|position| {
        let is_current = position == current;
        div()
            .h(px(DOT_HEIGHT))
            .w(px(if is_current { CURRENT_DOT_WIDTH } else { DOT_WIDTH }))
            .rounded(px(DOT_HEIGHT / 2.0))
            .bg(color(if is_current { 0xffffff } else { IDLE_DOT }))
    }));
    let clock = h_flex()
        .gap(px(6.0))
        .items_center()
        .text_size(px(CLOCK_SIZE))
        .text_color(color(0xffffff))
        .child(ring(BATTERY_GREEN))
        .child("15:05");
    illustration::scene(PICTURE_HEIGHT)
        .child(
            h_flex()
                .absolute()
                .left(px(HEADER_INSET))
                .right(px(HEADER_INSET))
                .top_0()
                .h(px(HEADER_HEIGHT))
                .px(px(12.0))
                .justify_between()
                .items_center()
                .rounded_b(px(HEADER_CORNER))
                .bg(color(0x000000))
                .child(dots)
                .child(clock),
        )
        .child(
            div()
                .absolute()
                .left(px(HEADER_INSET))
                .right(px(HEADER_INSET))
                .bottom_0()
                .h(px(20.0))
                .rounded_t(px(6.0))
                .bg(rgba(0xffffff0d)),
        )
        .into_any_element()
}

fn caption(position: usize, shown_count: usize, label: &str) -> SharedString {
    if position + 1 == shown_count && position > 0 {
        tr!("pages.preview_last", page = label).into()
    } else if position == 0 {
        tr!("pages.preview_first", page = label).into()
    } else {
        tr!("pages.preview_page", number = position + 1, page = label).into()
    }
}

/// The strip for the labels of the pages shown, in order; nothing while no page is shown.
pub fn strip(shown_labels: &[String], cx: &App) -> AnyElement {
    let shown_count = shown_labels.len();
    let pictures = pictured_positions(shown_count)
        .into_iter()
        .map(|position| (header(position, shown_count), caption(position, shown_count, &shown_labels[position])))
        .collect();
    illustration::strip(pictures, cx.theme().muted_foreground)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn many_pages_show_the_first_the_third_and_the_last() {
        assert_eq!(pictured_positions(9), [0, 2, 8]);
    }

    #[test]
    fn three_pages_show_each_of_them() {
        assert_eq!(pictured_positions(3), [0, 1, 2]);
    }

    #[test]
    fn one_or_two_pages_are_not_drawn_twice() {
        assert_eq!(pictured_positions(1), [0]);
        assert_eq!(pictured_positions(2), [0, 1]);
        assert!(pictured_positions(0).is_empty());
    }

    #[test]
    fn the_last_caption_names_key_zero_and_the_first_key_one() {
        assert_eq!(caption(0, 9, "Media"), "First page (key 1): Media");
        assert_eq!(caption(2, 9, "Calendar"), "Page 3 (key 3): Calendar");
        assert_eq!(caption(8, 9, "Connected devices"), "Last page (key 0): Connected devices");
    }
}
