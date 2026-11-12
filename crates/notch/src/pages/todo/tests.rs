use super::{
    layout::{VISIBLE_ROWS, WHEEL_NOTCH_DISTANCE, add_button, max_scroll, row_rect},
    list::Item,
    *,
};
use crate::gfx::{Point, Rect};

fn page_with(open: usize, done: usize) -> TodoPage {
    let items = (0..open)
        .map(|index| Item { text: format!("open {index}"), is_done: false })
        .chain((0..done).map(|index| Item { text: format!("done {index}"), is_done: true }))
        .collect();
    TodoPage::with_list(TodoList::with_items(items))
}

#[test]
fn an_empty_list_is_as_wide_as_a_full_one_and_keeps_the_usual_height() {
    let mut page = page_with(0, 0);
    assert_eq!(page.sizes().expanded.width, CONTENT_WIDTH);
    assert_eq!(page.sizes().expanded.height, 86.0);
    page.is_adding = true;
    assert_eq!(page.sizes().expanded.width, CONTENT_WIDTH);
    assert_eq!(page.rows(), [Row::Field]);
}

#[test]
fn the_page_height_follows_the_toolbar_and_the_task_rows() {
    assert_eq!(page_with(3, 0).sizes().expanded.height, 144.0);
    assert_eq!(page_with(5, 0).sizes().expanded.height, 220.0);
    assert_eq!(page_with(2, 0).sizes().expanded.width, CONTENT_WIDTH);
    let mut adding = page_with(4, 0);
    adding.is_adding = true;
    assert_eq!(adding.sizes().expanded.height, 220.0, "the input is a fifth row");
}

#[test]
fn the_list_takes_any_number_of_tasks_and_the_page_stops_growing_at_five_rows() {
    let mut page = page_with(0, 0);
    for index in 0..12 {
        assert!(page.list.add(&format!("task {index}")));
    }
    assert_eq!(page.list.len(), 12);
    assert_eq!(page.sizes().expanded.height, 220.0);
}

#[test]
fn scrolling_stops_at_the_last_row_and_a_cleared_list_returns_to_the_top() {
    let mut page = page_with(8, 0);
    page.scroll.jump_to_end(max_scroll(page.rows().len()));
    assert_eq!(page.scroll.value(), 3.0 * WHEEL_NOTCH_DISTANCE);
    page.list.clear();
    page.refresh_texts();
    assert_eq!(page.scroll.value(), 0.0);
}

#[test]
fn finished_items_follow_open_ones_and_the_input_comes_last() {
    let mut page = page_with(1, 2);
    assert_eq!(page.rows(), [Row::Item(0), Row::Item(1), Row::Item(2)]);
    page.is_adding = true;
    assert_eq!(page.rows().last(), Some(&Row::Field));
    assert!(page.list.item(1).is_some_and(|item| item.is_done));
    assert!(page.rows().len() <= VISIBLE_ROWS);
}

#[test]
fn clearing_the_list_empties_it_and_the_count_and_shows_the_empty_message() {
    let mut page = page_with(2, 1);
    page.list.clear();
    page.refresh_texts();
    assert_eq!(page.count_text, "0 left");
    assert!(page.is_empty_view());
    assert_eq!(page.sizes().expanded.width, CONTENT_WIDTH);
}

#[test]
fn moving_from_a_row_to_the_add_button_repaints_once() {
    let mut page = page_with(3, 0);
    let bounds = Rect::new(0.0, 0.0, CONTENT_WIDTH, 144.0);
    let center = |rect: Rect| Point::new(rect.center_x(), rect.center_y());
    let on_row = center(row_rect(bounds, 1, 0.0));
    let on_add = center(add_button(bounds));
    assert!(page.hover.set(page.target_at(bounds, on_row)), "entering the row repaints");
    assert_eq!(page.hover.get(), Some(TodoTarget::Row(1)));
    assert!(page.hover.set(page.target_at(bounds, on_add)), "the move to Add repaints");
    assert_eq!(page.hover.get(), Some(TodoTarget::Add));
    assert!(!page.hover.set(page.target_at(bounds, on_add)), "staying on Add does not repaint");
}

#[test]
fn the_todo_count_reads_in_indonesian() {
    store::language::with_override(store::language::Language::Indonesian, || {
        let mut page = page_with(2, 1);
        page.refresh_texts();
        assert_eq!(page.count_text, "2 tersisa \u{b7} 1 selesai");
        let mut open_only = page_with(3, 0);
        open_only.refresh_texts();
        assert_eq!(open_only.count_text, "3 tersisa");
    });
}

#[test]
fn the_undo_pill_answers_only_while_a_removal_can_be_undone() {
    let mut page = page_with(3, 0);
    let bounds = Rect::new(0.0, 0.0, CONTENT_WIDTH, 144.0);
    let on_undo = |page: &TodoPage| {
        let area = undo_button(bounds);
        page.target_at(bounds, Point::new(area.center_x(), area.center_y()))
    };
    assert_eq!(on_undo(&page), None);
    page.list.remove(0);
    assert_eq!(on_undo(&page), Some(TodoTarget::Undo));
    page.list.clear();
    assert_eq!(on_undo(&page), Some(TodoTarget::Undo), "after Clear all, Undo stays in its own place");
    page.list.forget_removed();
    assert_eq!(on_undo(&page), None);
}

#[test]
fn the_deleted_sample_offers_undo_and_hovers_its_first_row() {
    let page = TodoPage::sample(sample::DELETED);
    assert!(page.list.can_undo());
    assert_eq!(page.hover.get(), Some(TodoTarget::Row(0)));
    assert_eq!(page.count_text, "2 left \u{b7} 1 done");
}

#[test]
fn the_cleared_sample_is_the_empty_page_with_undo() {
    let page = TodoPage::sample(sample::CLEARED);
    assert!(page.is_empty_view());
    assert!(page.list.can_undo());
}

fn center(rect: Rect) -> Point {
    Point::new(rect.center_x(), rect.center_y())
}

#[test]
fn a_second_press_where_clear_all_was_does_nothing_after_the_list_is_cleared() {
    let mut page = page_with(3, 1);
    let bounds = Rect::new(0.0, 0.0, CONTENT_WIDTH, 144.0);
    let on_clear = center(layout::clear_button(bounds));
    assert_eq!(page.target_at(bounds, on_clear), Some(TodoTarget::ClearAll));
    page.clear_tasks(bounds, on_clear);
    assert_eq!(page.target_at(bounds, on_clear), None, "the empty Clear all place takes no press");
    assert!(page.list.is_empty() && page.list.can_undo());
}

#[test]
fn undo_is_pressed_at_its_own_place_and_brings_the_cleared_tasks_back() {
    let mut page = page_with(3, 1);
    let bounds = Rect::new(0.0, 0.0, CONTENT_WIDTH, 144.0);
    page.clear_tasks(bounds, center(layout::clear_button(bounds)));
    let on_undo = center(undo_button(bounds));
    assert_eq!(page.target_at(bounds, on_undo), Some(TodoTarget::Undo));
    assert!(page.undo_removal(bounds, on_undo));
    assert_eq!(page.list.open_count() + page.list.done_count(), 4);
    assert_eq!(page.count_text, "3 left \u{b7} 1 done");
}

#[test]
fn the_hover_follows_a_button_that_disappears_under_the_pointer() {
    let mut page = page_with(2, 0);
    let bounds = Rect::new(0.0, 0.0, CONTENT_WIDTH, 144.0);
    let on_clear = center(layout::clear_button(bounds));
    page.hover.set(page.target_at(bounds, on_clear));
    assert_eq!(page.hover.get(), Some(TodoTarget::ClearAll));
    page.clear_tasks(bounds, on_clear);
    assert_eq!(page.hover.get(), None, "Clear all is gone, so nothing is hovered");
    let on_undo = center(undo_button(bounds));
    page.hover.set(page.target_at(bounds, on_undo));
    assert_eq!(page.hover.get(), Some(TodoTarget::Undo));
    page.undo_removal(bounds, on_undo);
    assert_eq!(page.hover.get(), None, "Undo is gone after it is used");
}

#[test]
fn deleting_the_copied_task_clears_its_check() {
    let mut page = page_with(3, 0);
    page.copy_check.start(1);
    page.delete_task(1);
    assert!(!page.copy_check.shows(1) && !page.copy_check.shows(0) && !page.copy_check.shows(2));
}

#[test]
fn deleting_another_task_clears_the_check_so_it_never_lands_on_a_task_that_shifted_up() {
    let mut page = page_with(3, 0);
    page.copy_check.start(2);
    page.delete_task(0);
    assert!(!page.copy_check.shows(1), "the copied task is index 1 now and must not show the check");
    assert!(!page.copy_check.shows(2));
}

#[test]
fn clear_all_and_undo_clear_the_check() {
    let mut page = page_with(3, 1);
    let bounds = Rect::new(0.0, 0.0, CONTENT_WIDTH, 144.0);
    page.copy_check.start(0);
    page.clear_tasks(bounds, center(layout::clear_button(bounds)));
    assert!(!page.copy_check.shows(0));
    page.undo_removal(bounds, center(undo_button(bounds)));
    page.copy_check.start(0);
    page.delete_task(3);
    page.undo_removal(bounds, center(undo_button(bounds)));
    assert!(!page.copy_check.shows(0), "an undo shifts the indexes after it, so it clears the check too");
}

#[test]
fn pressing_undo_with_nothing_to_undo_still_drops_the_check_whose_timer_it_stops() {
    let mut page = page_with(3, 0);
    let bounds = Rect::new(0.0, 0.0, CONTENT_WIDTH, 144.0);
    page.copy_check.start(0);
    assert!(page.press_undo(bounds, center(undo_button(bounds))), "the check went away, so the page repaints");
    assert!(!page.copy_check.shows(0));
}

#[test]
fn ticking_the_copied_task_off_keeps_the_check_on_it() {
    let mut page = page_with(3, 0);
    page.copy_check.start(0);
    page.list.toggle(0);
    page.refresh_texts();
    assert!(page.copy_check.shows(0), "a tick moves the row, not the task's index");
    assert_eq!(page.rows().last(), Some(&Row::Item(0)), "the ticked task is now the last row");
}

#[test]
fn the_copied_sample_hovers_its_first_row_with_the_check_on_it() {
    let page = TodoPage::sample(sample::COPIED);
    assert_eq!(page.hover.get(), Some(TodoTarget::Row(0)));
    assert!(page.copy_check.shows(0));
}

#[test]
fn the_copy_button_still_takes_a_press_while_it_shows_the_check() {
    let row = row_rect(Rect::new(0.0, 0.0, CONTENT_WIDTH, 144.0), 0, 0.0);
    assert_eq!(actions::row_press(row, center(layout::copy_rect(row)), true), actions::RowPress::Copy);
}
