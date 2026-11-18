//! The page list as the notch keeps it: the known pages, their order, the ones switched off, and the digit key
//! that leads to each. Mirrors `PageList::load` in the notch.

pub const KNOWN_KEY: &str = "pages.known";
pub const ORDER_KEY: &str = "navigation.order";
pub const DISABLED_KEY: &str = "navigation.disabled";

pub struct KnownPage {
    pub id: String,
    pub label: String,
    pub opt_in: bool,
}

/// Reads the notch's `id:Label[:opt_in],...` list; an entry without a label shows its id.
pub fn parse_known(text: Option<&str>) -> Vec<KnownPage> {
    word_list(text)
        .into_iter()
        .filter_map(|entry| {
            let (entry, opt_in) = match entry.strip_suffix(":opt_in") {
                Some(rest) => (rest, true),
                None => (entry.as_str(), false),
            };
            let (id, label) = entry.split_once(':').unwrap_or((entry, ""));
            let (id, label) = (id.trim(), label.trim());
            (!id.is_empty()).then(|| KnownPage {
                id: id.to_owned(),
                label: if label.is_empty() { id } else { label }.to_owned(),
                opt_in,
            })
        })
        .collect()
}

/// Mirrors `PageList::load`: unknown and repeated ids are dropped and pages missing from the list go last.
pub fn current_order(known: &[KnownPage], order_text: Option<&str>) -> Vec<String> {
    let mut order: Vec<String> = Vec::new();
    for word in word_list(order_text) {
        if known.iter().any(|page| page.id == word) && !order.contains(&word) {
            order.push(word);
        }
    }
    let missing: Vec<String> =
        known.iter().filter(|page| !order.contains(&page.id)).map(|page| page.id.clone()).collect();
    order.extend(missing);
    order
}

/// Mirrors `PageList::load`: without a saved list only the opt-in pages start switched off.
pub fn disabled_pages(known: &[KnownPage], disabled_text: Option<&str>) -> Vec<String> {
    match disabled_text {
        Some(_) => word_list(disabled_text),
        None => known.iter().filter(|page| page.opt_in).map(|page| page.id.clone()).collect(),
    }
}

pub fn word_list(text: Option<&str>) -> Vec<String> {
    text.unwrap_or_default().split(',').map(str::trim).filter(|word| !word.is_empty()).map(str::to_owned).collect()
}

pub fn reorder(order: &mut Vec<String>, from: usize, to: usize) {
    if from == to || from >= order.len() || to >= order.len() {
        return;
    }
    let page = order.remove(from);
    order.insert(to, page);
}

pub fn order_text(order: &[String], disabled: &[String]) -> (String, String) {
    (order.join(","), disabled.join(","))
}

/// Refuses to switch off the last enabled page: the notch would have nothing to open.
pub fn switch_page(order: &[String], disabled: &mut Vec<String>, id: &str, enabled: bool) -> bool {
    let is_disabled = disabled.iter().any(|word| word == id);
    if enabled {
        disabled.retain(|word| word != id);
        return true;
    }
    if is_disabled {
        return true;
    }
    let enabled_count = order.iter().filter(|page| !disabled.contains(page)).count();
    if enabled_count <= 1 {
        return false;
    }
    disabled.push(id.to_owned());
    true
}

/// The digit key that leads to a page while the open notch is hovered (`page_hotkeys.rs` in the notch).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyHint {
    /// A page that is off: the notch skips it, so no key leads there.
    Skipped,
    /// 1 to 9.
    Digit(u8),
    /// The last page shown when it also has a digit: that digit and 0.
    DigitAndZero(u8),
    /// The last page shown when it comes after the ninth: only 0.
    Zero,
}

/// The key of each page in `order`: 1 to 9 for the first nine pages shown, 0 for the last; a page in `skipped` (off) gets none.
pub fn key_hints(order: &[String], skipped: &[String]) -> Vec<KeyHint> {
    let shown_count = order.iter().filter(|page| !skipped.contains(page)).count();
    let mut position = 0;
    order
        .iter()
        .map(|page| {
            if skipped.contains(page) {
                return KeyHint::Skipped;
            }
            position += 1;
            let is_last = position == shown_count;
            match (u8::try_from(position).ok().filter(|digit| *digit <= 9), is_last) {
                (Some(digit), false) => KeyHint::Digit(digit),
                (Some(digit), true) => KeyHint::DigitAndZero(digit),
                (None, true) => KeyHint::Zero,
                (None, false) => KeyHint::Skipped,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    fn known(text: &str) -> Vec<KnownPage> {
        parse_known(Some(text))
    }

    #[test]
    fn reordering_moves_a_page_to_the_new_position() {
        let mut order = strings(&["clock", "media", "timer", "stats"]);
        reorder(&mut order, 0, 2);
        assert_eq!(order, strings(&["media", "timer", "clock", "stats"]));
        reorder(&mut order, 3, 0);
        assert_eq!(order, strings(&["stats", "media", "timer", "clock"]));
    }

    #[test]
    fn reordering_to_the_same_or_an_out_of_range_position_changes_nothing() {
        let mut order = strings(&["clock", "media", "timer"]);
        let original = order.clone();
        reorder(&mut order, 1, 1);
        reorder(&mut order, 3, 0);
        reorder(&mut order, 0, 3);
        reorder(&mut order, 99, 99);
        assert_eq!(order, original);
        let mut empty: Vec<String> = Vec::new();
        reorder(&mut empty, 0, 0);
        assert!(empty.is_empty());
    }

    #[test]
    fn switching_a_page_off_adds_it_to_disabled() {
        let order = strings(&["clock", "media", "timer"]);
        let mut disabled = Vec::new();
        assert!(switch_page(&order, &mut disabled, "media", false));
        assert_eq!(disabled, strings(&["media"]));
        assert!(switch_page(&order, &mut disabled, "media", false), "switching off twice adds it once");
        assert_eq!(disabled, strings(&["media"]));
        assert!(switch_page(&order, &mut disabled, "media", true));
        assert!(disabled.is_empty());
    }

    #[test]
    fn the_last_enabled_page_cannot_be_switched_off() {
        let order = strings(&["clock", "media"]);
        let mut disabled = strings(&["media"]);
        assert!(!switch_page(&order, &mut disabled, "clock", false));
        assert_eq!(disabled, strings(&["media"]));
    }

    #[test]
    fn order_and_disabled_are_written_as_comma_separated_ids() {
        let (order, disabled) = order_text(&strings(&["media", "clock"]), &strings(&["clock"]));
        assert_eq!((order.as_str(), disabled.as_str()), ("media,clock", "clock"));
        assert_eq!(order_text(&[], &[]), (String::new(), String::new()));
    }

    #[test]
    fn known_pages_are_read_with_their_labels_and_junk_is_skipped() {
        let pages = known("clock:Clock,media:Now Playing,,:Nameless,broken");
        let ids: Vec<&str> = pages.iter().map(|page| page.id.as_str()).collect();
        assert_eq!(ids, ["clock", "media", "broken"]);
        assert_eq!(pages[1].label, "Now Playing");
        assert_eq!(pages[2].label, "broken", "a missing label falls back to the id");
        assert!(parse_known(None).is_empty());
        assert!(parse_known(Some("  ")).is_empty());
    }

    #[test]
    fn the_shown_order_ignores_unknown_and_repeated_ids_and_appends_missing_pages() {
        let pages = known("clock:Clock,media:Media,timer:Timer,stats:Stats");
        let shown = current_order(&pages, Some("timer, nope, timer, clock"));
        assert_eq!(shown, strings(&["timer", "clock", "media", "stats"]));
        assert_eq!(current_order(&pages, None), strings(&["clock", "media", "timer", "stats"]));
        assert!(current_order(&[], Some("clock")).is_empty());
    }

    #[test]
    fn a_saved_order_with_an_unknown_page_survives_a_reorder_without_the_unknown_id() {
        let pages = known("clock:Clock,media:Media,timer:Timer");
        let mut order = current_order(&pages, Some("ghost,timer"));
        reorder(&mut order, 2, 0);
        let (text, _) = order_text(&order, &[]);
        assert_eq!(text, "media,timer,clock");
    }

    #[test]
    fn an_opt_in_page_is_shown_off_when_no_disabled_list_is_saved() {
        let pages = known("clock:Clock,media:Media,stats:Stats:opt_in");
        assert!(pages[2].opt_in && !pages[0].opt_in);
        assert_eq!(pages[2].label, "Stats");
        assert_eq!(disabled_pages(&pages, None), strings(&["stats"]));
        assert!(disabled_pages(&pages, Some("")).is_empty(), "a saved empty list switches everything on");
        assert_eq!(disabled_pages(&pages, Some("media")), strings(&["media"]));
    }

    #[test]
    fn old_two_field_entries_are_not_opt_in() {
        let pages = known("clock:Clock,stats:Stats");
        assert!(disabled_pages(&pages, None).is_empty());
    }

    #[test]
    fn the_first_toggle_of_another_page_keeps_the_opt_in_page_off() {
        let pages = known("clock:Clock,media:Media,stats:Stats:opt_in");
        let order = current_order(&pages, None);
        let mut disabled = disabled_pages(&pages, None);
        assert!(switch_page(&order, &mut disabled, "media", false));
        assert_eq!(order_text(&order, &disabled).1, "stats,media");
    }

    #[test]
    fn the_last_enabled_guard_counts_the_opt_in_default() {
        let pages = known("clock:Clock,stats:Stats:opt_in");
        let order = current_order(&pages, None);
        let mut disabled = disabled_pages(&pages, None);
        assert!(!switch_page(&order, &mut disabled, "clock", false));
        assert_eq!(disabled, strings(&["stats"]));
    }

    #[test]
    fn a_stored_disabled_list_is_trimmed_and_blank_entries_dropped() {
        assert_eq!(word_list(Some(" timer , ,stats")), strings(&["timer", "stats"]));
        assert!(word_list(None).is_empty());
    }

    #[test]
    fn the_pages_shown_are_numbered_in_order_and_the_last_also_takes_zero() {
        let hints = key_hints(&strings(&["media", "shelf", "stats", "weather"]), &strings(&["stats"]));
        assert_eq!(hints, [KeyHint::Digit(1), KeyHint::Digit(2), KeyHint::Skipped, KeyHint::DigitAndZero(3)]);
    }

    #[test]
    fn a_page_after_the_ninth_has_no_key_unless_it_is_the_last() {
        let order: Vec<String> = (0..11).map(|index| format!("page{index}")).collect();
        let hints = key_hints(&order, &[]);
        assert_eq!(hints[8], KeyHint::Digit(9));
        assert_eq!(hints[9], KeyHint::Skipped);
        assert_eq!(hints[10], KeyHint::Zero);
    }

    #[test]
    fn with_every_page_off_no_key_leads_anywhere() {
        let order = strings(&["media"]);
        assert_eq!(key_hints(&order, &order), [KeyHint::Skipped]);
    }
}
