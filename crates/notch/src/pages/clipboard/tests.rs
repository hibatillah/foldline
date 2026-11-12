use std::time::Instant;

use super::item::Content;

use super::{
    sample::{COPIED, EMPTY, IMAGE_SELECTED, PASTED, PINNED, PINNED_FULL, REMOVED, SKIPPED, TEXT_SELECTED},
    *,
};
use crate::{
    gfx::Rect,
    page::{Page, RESTING_MIN_WIDTH},
};

fn frame() -> Frame {
    Frame {
        bounds: Rect::new(0.0, 0.0, CONTENT_WIDTH, CONTENT_HEIGHT),
        expansion: 1.0,
        expanded: Size::new(CONTENT_WIDTH, CONTENT_HEIGHT),
        lead_gap: 0.0,
    }
}

#[test]
fn the_page_is_opt_in_and_asks_for_the_designed_size() {
    let page = ClipboardPage::new();
    assert!(page.is_opt_in());
    assert_eq!(page.id(), "clipboard");
    let sizes = page.sizes();
    assert_eq!(sizes.expanded.width, 508.0);
    assert_eq!(sizes.resting_width, RESTING_MIN_WIDTH);
}

#[test]
fn an_empty_history_asks_for_the_default_height() {
    let page = ClipboardPage::sample(EMPTY);
    assert_eq!(page.sizes().expanded.height, DEFAULT_CONTENT_HEIGHT);
    assert_eq!(page.sizes().expanded.width, CONTENT_WIDTH);
}

#[test]
fn a_single_item_grows_the_page_to_the_content_height() {
    let mut page = ClipboardPage::sample(EMPTY);
    page.history.push(Content::Text("one".to_owned()), Instant::now());
    assert_eq!(page.sizes().expanded.height, CONTENT_HEIGHT);
    assert_eq!(ClipboardPage::sample(TEXT_SELECTED).sizes().expanded.height, CONTENT_HEIGHT);
}

#[test]
fn removing_the_last_item_returns_to_the_default_height() {
    let mut page = ClipboardPage::sample(EMPTY);
    page.history.push(Content::Text("one".to_owned()), Instant::now());
    page.history.remove(0);
    assert_eq!(page.sizes().expanded.height, DEFAULT_CONTENT_HEIGHT);
    assert!(page.history.undo());
    assert_eq!(page.sizes().expanded.height, CONTENT_HEIGHT);
}

#[test]
fn the_page_claims_the_open_notch_only_as_a_background_page() {
    let page = ClipboardPage::new();
    assert_eq!(page.priority(Mode::Resting), None);
    assert_eq!(page.priority(Mode::Expanded), Some(Priority::BACKGROUND));
}

#[test]
fn a_new_page_is_off_and_holds_nothing() {
    let page = ClipboardPage::new();
    assert!(!page.is_enabled);
    assert!(page.history.is_empty());
}

#[test]
fn the_text_sample_lists_seven_copies_with_the_newest_selected() {
    let page = ClipboardPage::sample(TEXT_SELECTED);
    assert_eq!(page.history.items().len(), 7);
    assert_eq!(page.history.selected_index(), 0);
    assert!(page.confirm.is_none() && page.bar.is_none());
}

#[test]
fn the_image_sample_selects_the_picture() {
    let page = ClipboardPage::sample(IMAGE_SELECTED);
    let selected = page.history.selected().unwrap();
    assert_eq!(item::row_label(selected), "Image (360×70)");
}

#[test]
fn the_picture_samples_list_five_pictures_and_the_text_sample_one() {
    let pictures =
        |page: &ClipboardPage| page.history.items().iter().filter(|item| item.kind == item::Kind::Image).count();
    assert_eq!(pictures(&ClipboardPage::sample(IMAGE_SELECTED)), 5);
    assert_eq!(ClipboardPage::sample(IMAGE_SELECTED).history.items().len(), 7);
    assert_eq!(pictures(&ClipboardPage::sample(TEXT_SELECTED)), 1);
}

#[test]
fn the_pasted_and_copied_samples_confirm_the_selected_row() {
    assert_eq!(ClipboardPage::sample(PASTED).confirm, Some(Confirm::Pasted));
    let copied = ClipboardPage::sample(COPIED);
    assert_eq!(copied.confirm, Some(Confirm::Copied));
    assert_eq!(copied.history.selected_index(), 0);
    assert_eq!(item::details(copied.history.selected().unwrap()), "Image · 360×70 · PNG");
}

#[test]
fn the_removed_sample_has_six_rows_and_offers_undo() {
    let page = ClipboardPage::sample(REMOVED);
    assert_eq!(page.history.items().len(), 6);
    assert!(page.history.can_undo());
    assert_eq!(page.bar, Some(Bar::Removed));
}

#[test]
fn the_skipped_sample_shows_the_password_manager_bar_without_undo() {
    let page = ClipboardPage::sample(SKIPPED);
    assert_eq!(page.bar.map(Bar::text).as_deref(), Some("Skipped: password manager"));
    assert!(!page.history.can_undo());
}

#[test]
fn the_empty_sample_has_no_rows_and_answers_no_clicks() {
    let page = ClipboardPage::sample(EMPTY);
    let frame = frame();
    let layout = ClipboardPage::layout(&frame);
    assert!(page.history.is_empty());
    for rect in [layout.row(0), layout.paste_button(), layout.copy_button(), layout.delete_button()] {
        let center = Point::new(rect.center_x(), rect.center_y());
        assert_eq!(page.target_at(center, &frame), None);
        assert!(!page.uses_right_click(center, &frame));
    }
}

#[test]
fn right_click_means_something_only_over_a_row() {
    let page = ClipboardPage::sample(TEXT_SELECTED);
    let frame = frame();
    let layout = ClipboardPage::layout(&frame);
    let over = |rect: Rect| page.uses_right_click(Point::new(rect.center_x(), rect.center_y()), &frame);
    assert!(over(layout.row(3)));
    assert!(!over(layout.panel()));
    assert!(!over(layout.paste_button()));
}

#[test]
fn clicks_land_on_the_designed_targets() {
    let page = ClipboardPage::sample(TEXT_SELECTED);
    let frame = frame();
    let layout = ClipboardPage::layout(&frame);
    let at = |rect: Rect| page.target_at(Point::new(rect.center_x(), rect.center_y()), &frame);
    assert_eq!(at(layout.row(2)), Some(Target::Row(2)));
    assert_eq!(at(layout.delete_button()), Some(Target::Delete));
    assert_eq!(at(layout.copy_button()), Some(Target::Copy));
    assert_eq!(at(layout.paste_button()), Some(Target::Paste));
}

#[test]
fn only_the_removed_bar_makes_its_right_end_an_undo_button() {
    let frame = frame();
    let layout = ClipboardPage::layout(&frame);
    let on_undo = Point::new(layout.bar().right - 10.0, layout.bar().center_y());
    assert_eq!(ClipboardPage::sample(REMOVED).target_at(on_undo, &frame), Some(Target::Undo));
    assert_eq!(ClipboardPage::sample(SKIPPED).target_at(on_undo, &frame), None);
}

#[test]
fn the_pinned_sample_lists_its_pins_first_and_offers_a_pin_on_the_hovered_row() {
    let page = ClipboardPage::sample(PINNED);
    let frame = frame();
    let layout = ClipboardPage::layout(&frame);
    assert!(page.history.is_pinned(0) && page.history.is_pinned(1) && !page.history.is_pinned(2));
    let pin_of = |row: usize| Point::new(layout.pin_area(row).center_x(), layout.pin_area(row).center_y());
    assert_eq!(page.target_at(pin_of(4), &frame), Some(Target::Pin(4)));
    assert_eq!(page.target_at(pin_of(0), &frame), Some(Target::Pin(0)));
}

#[test]
fn with_three_pins_only_the_pinned_rows_answer_on_their_pin() {
    let page = ClipboardPage::sample(PINNED_FULL);
    let frame = frame();
    let layout = ClipboardPage::layout(&frame);
    let pin_of = |row: usize| Point::new(layout.pin_area(row).center_x(), layout.pin_area(row).center_y());
    assert_eq!(page.target_at(pin_of(2), &frame), Some(Target::Pin(2)));
    assert_eq!(page.target_at(pin_of(4), &frame), Some(Target::Row(4)));
}

#[test]
fn right_click_on_a_pin_removes_the_row_like_anywhere_else_on_it() {
    let page = ClipboardPage::sample(PINNED);
    let frame = frame();
    let layout = ClipboardPage::layout(&frame);
    let on_pin = Point::new(layout.pin_area(0).center_x(), layout.pin_area(0).center_y());
    assert!(page.uses_right_click(on_pin, &frame));
}
