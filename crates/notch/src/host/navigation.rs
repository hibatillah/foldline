//! Page selection and scroll state, independent of the window.

use super::motion::PAGE_SLIDE_RESPONSE;
use crate::{gfx::Spring, page::Priority};

/// The position of the page dots' long mark along the row of pages; whole numbers are pages.
pub(super) fn page_transition(position: f32) -> Spring {
    Spring::new(position, PAGE_SLIDE_RESPONSE.0, PAGE_SLIDE_RESPONSE.1)
}

/// The order of the pages and which ones the user chose to show.
pub(super) struct PageList {
    order: Vec<usize>,
    chosen: Vec<bool>,
    opt_in: Vec<bool>,
}

impl PageList {
    pub fn new(count: usize) -> Self {
        Self { order: (0..count).collect(), chosen: vec![true; count], opt_in: vec![false; count] }
    }

    /// Ignore unknown IDs, append missing pages, and apply opt-in defaults only without saved disables.
    pub fn load(ids: &[&str], opt_in: &[bool], order_text: Option<&str>, disabled_text: Option<&str>) -> Self {
        let words = |text: &str| -> Vec<String> {
            text.split(',').map(str::trim).filter(|word| !word.is_empty()).map(str::to_owned).collect()
        };
        let mut order: Vec<usize> = Vec::new();
        for word in order_text.map(words).unwrap_or_default() {
            if let Some(index) = ids.iter().position(|id| *id == word)
                && !order.contains(&index)
            {
                order.push(index);
            }
        }
        let missing: Vec<usize> = (0..ids.len()).filter(|index| !order.contains(index)).collect();
        order.extend(missing);
        let chosen = match disabled_text {
            Some(text) => {
                let disabled = words(text);
                ids.iter().map(|id| !disabled.iter().any(|word| word == id)).collect()
            }
            None => opt_in.iter().map(|is_opt_in| !is_opt_in).collect(),
        };
        Self { order, chosen, opt_in: opt_in.to_vec() }
    }

    #[cfg(test)]
    pub fn order(&self) -> &[usize] {
        &self.order
    }

    /// Chosen; when no page is chosen, the pages that are not opt-in show instead.
    pub fn is_enabled(&self, page: usize) -> bool {
        if self.chosen.iter().any(|is_chosen| *is_chosen) {
            self.is_chosen(page)
        } else {
            page < self.chosen.len() && !self.opt_in.get(page).copied().unwrap_or(false)
        }
    }

    /// The user's own on or off, which `disabled_text` saves.
    pub fn is_chosen(&self, page: usize) -> bool {
        self.chosen.get(page).copied().unwrap_or(false)
    }

    #[cfg(test)]
    pub fn set_enabled(&mut self, page: usize, is_enabled: bool) {
        if let Some(slot) = self.chosen.get_mut(page) {
            *slot = is_enabled;
        }
    }

    pub fn order_text(&self, ids: &[&str]) -> String {
        self.order.iter().filter_map(|&index| ids.get(index).copied()).collect::<Vec<_>>().join(",")
    }

    pub fn disabled_text(&self, ids: &[&str]) -> String {
        let off = |index: &usize| !self.is_chosen(*index);
        (0..ids.len()).filter(off).map(|index| ids[index]).collect::<Vec<_>>().join(",")
    }

    pub fn navigable(&self, priorities: &[Option<Priority>]) -> Vec<usize> {
        self.order
            .iter()
            .copied()
            .filter(|&index| self.is_enabled(index) && priorities.get(index).is_some_and(Option::is_some))
            .collect()
    }

    pub fn effective(&self, priorities: &[Option<Priority>]) -> Vec<Option<Priority>> {
        priorities
            .iter()
            .enumerate()
            .map(|(index, priority)| if self.is_enabled(index) { *priority } else { None })
            .collect()
    }
}

/// The `id:Label[:opt_in]` list the settings window reads as `pages.known`; it lists pages without linking to the notch.
pub(super) fn known_pages_text(ids: &[&str], labels: &[&str], opt_in: &[bool]) -> String {
    debug_assert!(ids.len() == labels.len() && ids.len() == opt_in.len());
    ids.iter()
        .zip(labels)
        .zip(opt_in)
        .map(|((id, label), is_opt_in)| format!("{id}:{label}{}", if *is_opt_in { ":opt_in" } else { "" }))
        .collect::<Vec<_>>()
        .join(",")
}

/// Urgent pages override selection; otherwise retain an available selection before falling back to priority.
pub(super) fn choose(selected: Option<usize>, priorities: &[Option<Priority>]) -> Option<usize> {
    if let Some(index) = highest(priorities.iter().copied(), Priority::URGENT) {
        return Some(index);
    }
    if let Some(index) = selected
        && priorities.get(index).is_some_and(Option::is_some)
    {
        return Some(index);
    }
    highest(priorities.iter().copied(), Priority::BACKGROUND)
}

/// The first page with the highest priority of at least `lowest`.
pub(super) fn highest(priorities: impl IntoIterator<Item = Option<Priority>>, lowest: Priority) -> Option<usize> {
    let mut best: Option<(usize, Priority)> = None;
    for (index, priority) in priorities.into_iter().enumerate() {
        let Some(priority) = priority else { continue };
        if priority >= lowest && best.is_none_or(|(_, best_priority)| priority > best_priority) {
            best = Some((index, priority));
        }
    }
    best.map(|(index, _)| index)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOW: Option<Priority> = Some(Priority(1));
    const HIGH: Option<Priority> = Some(Priority(5));

    const IDS: [&str; 4] = ["clock", "media", "timer", "stats"];

    #[test]
    fn only_pages_with_something_to_show_can_be_opened() {
        let list = PageList::new(3);
        assert_eq!(list.navigable(&[LOW, None, HIGH]), vec![0, 2]);
        assert!(list.navigable(&[None, None, None]).is_empty());
    }

    #[test]
    fn pages_are_reached_in_the_users_order_and_switched_off_ones_are_skipped() {
        let mut list = PageList::load(&IDS[..3], &[false; 3], Some("timer"), None);
        assert_eq!(list.navigable(&[LOW, LOW, LOW]), vec![2, 0, 1]);
        list.chosen[0] = false;
        assert_eq!(list.navigable(&[LOW, LOW, LOW]), vec![2, 1]);
        assert_eq!(list.effective(&[LOW, LOW, LOW]), vec![None, LOW, LOW]);
    }

    #[test]
    fn the_saved_order_and_choices_come_back_by_id() {
        let mut list = PageList::load(&IDS, &[false; 4], Some("clock,stats"), None);
        list.chosen[2] = false;
        let (order, disabled) = (list.order_text(&IDS), list.disabled_text(&IDS));
        assert_eq!(order, "clock,stats,media,timer");
        assert_eq!(disabled, "timer");
        let restored = PageList::load(&IDS, &[false; 4], Some(&order), Some(&disabled));
        assert_eq!(restored.order, list.order);
        assert!(!restored.is_enabled(2) && restored.is_enabled(3));
    }

    #[test]
    fn unknown_and_repeated_ids_are_ignored_and_missing_pages_go_last() {
        let list = PageList::load(&IDS, &[false; 4], Some("timer, nope, timer, clock"), None);
        assert_eq!(list.order, [2, 0, 1, 3]);
    }

    #[test]
    fn without_a_saved_choice_only_opt_in_pages_start_switched_off() {
        let list = PageList::load(&IDS, &[false, false, false, true], None, None);
        assert!(list.is_enabled(0) && !list.is_enabled(3));
        let saved = PageList::load(&IDS, &[false, false, false, true], None, Some(""));
        assert!(saved.is_enabled(3));
    }

    #[test]
    fn switching_every_page_off_shows_the_pages_that_are_not_opt_in_and_keeps_the_saved_choice() {
        let list = PageList::load(&IDS, &[false, false, false, true], None, Some("clock,media,timer,stats"));
        let enabled: Vec<bool> = (0..IDS.len()).map(|index| list.is_enabled(index)).collect();
        assert_eq!(enabled, [true, true, true, false]);
        assert_eq!(list.disabled_text(&IDS), "clock,media,timer,stats");
    }

    #[test]
    fn known_pages_text_joins_ids_and_labels() {
        let text = known_pages_text(&["clock", "stats"], &["Clock", "Stats"], &[false, true]);
        assert_eq!(text, "clock:Clock,stats:Stats:opt_in");
        assert_eq!(known_pages_text(&[], &[], &[]), "");
    }

    #[test]
    fn without_a_choice_the_highest_priority_opens() {
        assert_eq!(choose(None, &[LOW, HIGH, LOW]), Some(1));
        assert_eq!(choose(None, &[None, None]), None);
    }

    #[test]
    fn the_chosen_page_opens_even_when_another_ranks_higher() {
        assert_eq!(choose(Some(0), &[LOW, HIGH]), Some(0));
    }

    #[test]
    fn an_urgent_page_comes_before_the_chosen_one() {
        const URGENT: Option<Priority> = Some(Priority::URGENT);
        assert_eq!(choose(Some(0), &[LOW, URGENT]), Some(1));
        assert_eq!(choose(None, &[URGENT, URGENT]), Some(0));
    }

    #[test]
    fn a_chosen_page_with_nothing_to_show_gives_way_to_the_best_one() {
        assert_eq!(choose(Some(0), &[None, HIGH]), Some(1));
        assert_eq!(choose(Some(7), &[LOW]), Some(0));
    }

    #[test]
    fn no_page_wants_to_show() {
        assert_eq!(highest([None, None], Priority::BACKGROUND), None);
        assert_eq!(highest(Vec::new(), Priority::BACKGROUND), None);
    }

    #[test]
    fn the_highest_priority_wins_and_the_earliest_page_wins_a_tie() {
        assert_eq!(highest([LOW, HIGH, LOW], Priority::BACKGROUND), Some(1));
        assert_eq!(highest([HIGH, HIGH], Priority::BACKGROUND), Some(0));
    }

    #[test]
    fn pages_with_nothing_to_show_or_below_the_floor_are_skipped() {
        assert_eq!(highest([None, LOW, None], Priority::BACKGROUND), Some(1));
        assert_eq!(highest([LOW, HIGH], Priority::URGENT), None);
    }

    #[test]
    fn page_transitions_arrive_quickly_without_overshooting_and_then_settle() {
        for refresh_rate in [60, 120] {
            let mut transition = page_transition(0.0);
            transition.target = 1.0;
            let mut previous = 0.0;
            for frame in 0..refresh_rate / 2 {
                transition.step(1.0 / refresh_rate as f32);
                assert!(transition.position >= previous && transition.position <= 1.0);
                previous = transition.position;
                if frame == refresh_rate / 4 {
                    assert!(transition.position > 0.98, "still at {} after 250 ms", transition.position);
                }
            }
            assert!(transition.is_settled());
        }
    }
}
