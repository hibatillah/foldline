use super::*;
use crate::{gfx::ImageData, pages::clipboard::item::StoredImage};

fn text(value: &str) -> Content {
    Content::Text(value.to_owned())
}

fn texts(history: &History) -> Vec<String> {
    history
        .items()
        .iter()
        .map(|item| match &item.content {
            Content::Text(text) => text.clone(),
            Content::Image(_) => "image".to_owned(),
        })
        .collect()
}

#[test]
fn the_newest_copy_goes_on_top_and_is_selected() {
    let mut history = History::default();
    history.push(text("one"), Instant::now());
    history.push(text("two"), Instant::now());
    assert_eq!(texts(&history), ["two", "one"]);
    assert_eq!(history.selected_index(), 0);
}

#[test]
fn only_the_last_seven_copies_are_kept() {
    let mut history = History::default();
    for number in 0..10 {
        history.push(text(&format!("copy {number}")), Instant::now());
    }
    assert_eq!(history.items().len(), MAX_ITEMS);
    assert_eq!(texts(&history)[0], "copy 9");
    assert_eq!(texts(&history)[6], "copy 3");
}

#[test]
fn copying_the_same_text_again_moves_it_to_the_front_instead_of_adding_it() {
    let mut history = History::default();
    history.push(text("a"), Instant::now());
    history.push(text("b"), Instant::now());
    history.push(text("c"), Instant::now());
    assert_eq!(history.push(text("a"), Instant::now()), Pushed::MovedToFront);
    assert_eq!(texts(&history), ["a", "c", "b"]);
    assert_eq!(history.selected_index(), 0);
}

#[test]
fn moving_an_item_to_the_front_refreshes_its_time_and_selects_it() {
    let mut history = History::default();
    let old = Instant::now();
    history.push(text("a"), old);
    history.push(text("b"), old);
    let later = old + std::time::Duration::from_secs(60);
    history.move_to_front(1, later);
    assert_eq!(texts(&history), ["a", "b"]);
    assert_eq!(history.items()[0].copied_at, later);
    assert_eq!(history.selected_index(), 0);
}

#[test]
fn removing_closes_the_gap_and_undo_restores_the_place_and_selection() {
    let mut history = History::default();
    for value in ["a", "b", "c", "d"] {
        history.push(text(value), Instant::now());
    }
    history.remove(1);
    assert_eq!(texts(&history), ["d", "b", "a"]);
    assert!(history.can_undo());
    assert!(history.undo());
    assert_eq!(texts(&history), ["d", "c", "b", "a"]);
    assert_eq!(history.selected_index(), 1);
    assert!(!history.can_undo());
    assert!(!history.undo());
}

#[test]
fn a_second_removal_replaces_what_the_first_could_have_undone() {
    let mut history = History::default();
    for value in ["a", "b", "c"] {
        history.push(text(value), Instant::now());
    }
    history.remove(0);
    history.remove(0);
    assert!(history.undo());
    assert_eq!(texts(&history), ["b", "a"]);
}

#[test]
fn removing_the_last_item_leaves_an_empty_history_that_can_still_undo() {
    let mut history = History::default();
    history.push(text("only"), Instant::now());
    history.remove(0);
    assert!(history.is_empty());
    assert!(history.selected().is_none());
    assert!(history.undo());
    assert_eq!(texts(&history), ["only"]);
}

#[test]
fn the_selection_stays_on_the_same_item_when_one_above_it_is_removed() {
    let mut history = History::default();
    for value in ["a", "b", "c"] {
        history.push(text(value), Instant::now());
    }
    history.select(2);
    history.remove(0);
    assert_eq!(history.selected_index(), 1);
    assert!(matches!(&history.selected().unwrap().content, Content::Text(text) if text == "a"));
}

#[test]
fn pictures_over_the_total_limit_push_the_oldest_copies_out() {
    let mut history = History::default();
    history.push(text("old"), Instant::now());
    let big = |value: u8| {
        let side = 1024u32;
        let pixels = ImageData::new(side, side, vec![value; (side * side * 4) as usize]);
        Content::Image(StoredImage::new(pixels, side, side, "PNG"))
    };
    for value in 1..=4 {
        history.push(big(value), Instant::now());
    }
    // Three 4 MiB pictures fill the 12 MiB limit, so the text and the first picture are gone.
    assert_eq!(history.items().len(), 3);
    assert!(history.total_bytes() <= MAX_TOTAL_BYTES);
}

#[test]
fn clearing_forgets_the_items_and_the_undo() {
    let mut history = History::default();
    history.push(text("a"), Instant::now());
    history.remove(0);
    history.push(text("b"), Instant::now());
    history.clear();
    assert!(history.is_empty());
    assert!(!history.can_undo());
}

fn history_of(values: &[&str]) -> History {
    let mut history = History::default();
    for value in values {
        history.push(text(value), Instant::now());
    }
    history
}

#[test]
fn a_pinned_copy_moves_above_the_others_and_pins_keep_the_order_they_were_made_in() {
    let mut history = history_of(&["a", "b", "c", "d"]);
    assert!(history.toggle_pin(2));
    assert_eq!(texts(&history), ["b", "d", "c", "a"]);
    assert!(history.toggle_pin(3));
    assert_eq!(texts(&history), ["b", "a", "d", "c"]);
    assert!(history.is_pinned(0) && history.is_pinned(1) && !history.is_pinned(2));
}

#[test]
fn a_fourth_pin_is_refused_and_unpinned_rows_lose_their_pin_button() {
    let mut history = history_of(&["a", "b", "c", "d", "e"]);
    for _ in 0..3 {
        assert!(history.toggle_pin(4));
    }
    assert_eq!(texts(&history), ["a", "b", "c", "e", "d"]);
    assert!(!history.has_pin_button(3));
    assert!(!history.toggle_pin(3));
    assert_eq!(texts(&history), ["a", "b", "c", "e", "d"]);
    assert!(history.has_pin_button(0), "a pinned row keeps its button to unpin");
    assert!(history.toggle_pin(0));
    assert!(history.has_pin_button(4));
}

#[test]
fn unpinning_puts_the_copy_back_among_the_others_by_the_time_it_was_copied() {
    let start = Instant::now();
    let mut history = History::default();
    for (seconds, value) in ["a", "b", "c", "d"].iter().enumerate() {
        history.push(text(value), start + std::time::Duration::from_secs(seconds as u64));
    }
    history.toggle_pin(2);
    assert_eq!(texts(&history), ["b", "d", "c", "a"]);
    assert!(history.toggle_pin(0));
    assert_eq!(texts(&history), ["d", "c", "b", "a"]);
    assert!(!history.is_pinned(0));
}

#[test]
fn a_new_copy_lands_under_the_pinned_ones_and_is_selected() {
    let mut history = history_of(&["a", "b"]);
    history.toggle_pin(1);
    history.push(text("c"), Instant::now());
    assert_eq!(texts(&history), ["a", "c", "b"]);
    assert_eq!(history.selected_index(), 1);
}

#[test]
fn copying_again_never_lifts_an_unpinned_copy_above_a_pinned_one() {
    let mut history = history_of(&["a", "b", "c"]);
    history.toggle_pin(2);
    assert_eq!(history.push(text("b"), Instant::now()), Pushed::MovedToFront);
    assert_eq!(texts(&history), ["a", "b", "c"]);
    assert_eq!(history.selected_index(), 1);
    history.move_to_front(2, Instant::now());
    assert_eq!(texts(&history), ["a", "c", "b"]);
}

#[test]
fn copying_a_pinned_copy_again_keeps_it_pinned_in_its_place() {
    let mut history = history_of(&["a", "b", "c"]);
    history.toggle_pin(2);
    history.toggle_pin(2);
    let later = Instant::now() + std::time::Duration::from_secs(60);
    assert_eq!(history.push(text("b"), later), Pushed::MovedToFront);
    assert_eq!(texts(&history), ["a", "b", "c"]);
    assert_eq!(history.items()[1].copied_at, later);
    assert_eq!(history.selected_index(), 1);
}

#[test]
fn the_seven_copy_limit_drops_the_oldest_unpinned_copy_and_never_a_pinned_one() {
    let mut history = history_of(&["oldest", "b", "c"]);
    history.toggle_pin(2);
    for number in 0..10 {
        history.push(text(&format!("copy {number}")), Instant::now());
    }
    assert_eq!(history.items().len(), MAX_ITEMS);
    assert_eq!(texts(&history)[0], "oldest");
    assert_eq!(texts(&history)[1], "copy 9");
    assert_eq!(texts(&history)[6], "copy 4");
}

#[test]
fn the_byte_limit_drops_unpinned_pictures_and_keeps_pinned_ones() {
    let big = |value: u8| {
        let side = 1024u32;
        let pixels = ImageData::new(side, side, vec![value; (side * side * 4) as usize]);
        Content::Image(StoredImage::new(pixels, side, side, "PNG"))
    };
    let mut history = History::default();
    history.push(big(1), Instant::now());
    history.toggle_pin(0);
    history.push(text("note"), Instant::now());
    for value in 2..=5 {
        history.push(big(value), Instant::now());
    }
    assert!(history.is_pinned(0));
    assert!(matches!(&history.items()[0].content, Content::Image(image) if image.pixels.bgra_premultiplied[0] == 1));
    assert!(history.total_bytes() <= MAX_TOTAL_BYTES);
    assert!(!texts(&history).contains(&"note".to_owned()), "the oldest unpinned copy went first");
}

#[test]
fn pinned_copies_over_a_limit_stay_with_the_newest_copy() {
    let mut history = History::default();
    let side = 1024u32;
    for value in 1..=3 {
        let pixels = ImageData::new(side, side, vec![value; (side * side * 4) as usize]);
        history.push(Content::Image(StoredImage::new(pixels, side, side, "PNG")), Instant::now());
        history.toggle_pin(history.pinned_count);
    }
    let pixels = ImageData::new(side, side + 300, vec![9; (side * (side + 300) * 4) as usize]);
    history.push(Content::Image(StoredImage::new(pixels, side, side + 300, "PNG")), Instant::now());
    assert!(history.total_bytes() > MAX_TOTAL_BYTES);
    assert_eq!(history.items().len(), 4);
}

#[test]
fn undoing_the_removal_of_a_pinned_copy_restores_it_pinned_in_its_place() {
    let mut history = history_of(&["a", "b", "c", "d"]);
    history.toggle_pin(3);
    history.toggle_pin(3);
    assert_eq!(texts(&history), ["a", "b", "d", "c"]);
    history.remove(1);
    assert!(!history.is_pinned(1));
    assert!(history.has_pin_button(2));
    assert!(history.undo());
    assert_eq!(texts(&history), ["a", "b", "d", "c"]);
    assert!(history.is_pinned(1));
    assert_eq!(history.selected_index(), 1);
}

#[test]
fn an_undone_pin_comes_back_unpinned_when_three_were_pinned_since() {
    let mut history = history_of(&["a", "b", "c", "d", "e"]);
    history.toggle_pin(4);
    history.remove(0);
    for _ in 0..3 {
        history.toggle_pin(3);
    }
    assert!(history.undo());
    assert_eq!(texts(&history)[3], "a");
    assert!(!history.is_pinned(3));
    assert_eq!(history.selected_index(), 3);
}

#[test]
fn the_selection_follows_its_copy_when_another_is_pinned() {
    let mut history = history_of(&["a", "b", "c"]);
    history.select(1);
    history.toggle_pin(2);
    assert_eq!(texts(&history), ["a", "c", "b"]);
    assert_eq!(history.selected_index(), 2);
}

#[test]
fn the_selection_follows_its_copy_when_another_is_unpinned() {
    let start = Instant::now();
    let mut history = History::default();
    for (seconds, value) in ["a", "b", "c", "d"].iter().enumerate() {
        history.push(text(value), start + std::time::Duration::from_secs(seconds as u64));
    }
    history.toggle_pin(3);
    assert_eq!(texts(&history), ["a", "d", "c", "b"]);
    history.select(2);
    assert!(history.toggle_pin(0));
    assert_eq!(texts(&history), ["d", "c", "b", "a"]);
    assert_eq!(history.selected_index(), 1);
    assert!(matches!(&history.selected().unwrap().content, Content::Text(text) if text == "c"));
}

#[test]
fn clearing_drops_the_pins_too() {
    let mut history = history_of(&["a", "b"]);
    history.toggle_pin(0);
    history.clear();
    history.push(text("c"), Instant::now());
    assert!(!history.is_pinned(0));
}
