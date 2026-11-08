//! How the listener looks for new toasts after the notification folder changed (no Windows calls).

use std::collections::HashSet;

/// Consecutive ids with no notification after which asking by id stops; room for ids Windows gave to other things.
pub const MAX_ID_MISSES: u32 = 8;

/// Ids in `listed` that are not in `seen`, in list order.
pub fn unseen_ids(listed: &[u32], seen: &HashSet<u32>) -> Vec<u32> {
    listed.iter().copied().filter(|id| !seen.contains(id)).collect()
}

/// Windows numbers notifications upward, so the next arrivals are the ids above the highest one seen. Calls `ask` with
/// each id after `high_water` until `MAX_ID_MISSES` in a row find nothing (`ask` returns whether it found one, and takes
/// the news on as it does); returns the highest id found, or `high_water` when none.
pub fn follow_ids(high_water: u32, mut ask: impl FnMut(u32) -> bool) -> u32 {
    let mut newest = high_water;
    let mut id = high_water;
    let mut misses = 0;
    while misses < MAX_ID_MISSES {
        let Some(next_id) = id.checked_add(1) else { break };
        id = next_id;
        if ask(id) {
            newest = id;
            misses = 0;
        } else {
            misses += 1;
        }
    }
    newest
}

/// What one wake does after a round of asking for the ids above the highest one seen.
#[derive(Debug, PartialEq, Eq)]
pub enum Step {
    /// The round found news: wait out the changes it made, then ask again for toasts that came in meanwhile.
    AskAgain,
    /// Nothing arrived by id, so a toast may have a number outside the guess: read the whole list.
    ReadList,
    Done,
}

/// A wake whose rounds find nothing falls back to one full read, because asking by id is only a guess about Windows'
/// numbering; after the read, one more round finds toasts that arrived while it ran. A wake that found news earlier,
/// at the first change or in an earlier round, needs no read.
pub fn next_step(round_found_news: bool, wake_found_news: bool, list_was_read: bool) -> Step {
    if round_found_news {
        Step::AskAgain
    } else if wake_found_news || list_was_read {
        Step::Done
    } else {
        Step::ReadList
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_ids_not_seen_before_are_new_and_keep_list_order() {
        let seen: HashSet<u32> = [3, 5].into_iter().collect();
        assert_eq!(unseen_ids(&[9, 3, 4, 5, 6], &seen), vec![9, 4, 6]);
        assert!(unseen_ids(&[3, 5], &seen).is_empty());
    }

    fn follow(high_water: u32, existing: &[u32]) -> (Vec<u32>, u32, Vec<u32>) {
        let mut asked = Vec::new();
        let mut found = Vec::new();
        let newest = follow_ids(high_water, |id| {
            asked.push(id);
            let exists = existing.contains(&id);
            if exists {
                found.push(id);
            }
            exists
        });
        (found, newest, asked)
    }

    #[test]
    fn asking_by_id_finds_consecutive_arrivals_and_moves_the_high_water_to_the_last() {
        let (found, newest, _) = follow(100, &[101, 102, 103]);
        assert_eq!(found, vec![101, 102, 103]);
        assert_eq!(newest, 103);
    }

    #[test]
    fn asking_by_id_steps_over_a_gap_shorter_than_the_miss_limit() {
        let (found, newest, _) = follow(100, &[104, 105]);
        assert_eq!(found, vec![104, 105]);
        assert_eq!(newest, 105);
    }

    #[test]
    fn asking_by_id_stops_after_the_miss_limit_and_leaves_the_high_water_alone() {
        let (found, newest, asked) = follow(100, &[]);
        assert!(found.is_empty());
        assert_eq!(newest, 100);
        assert_eq!(asked, (101..=100 + MAX_ID_MISSES).collect::<Vec<_>>());
    }

    #[test]
    fn asking_by_id_does_not_reach_a_notification_past_the_miss_limit() {
        let (found, newest, _) = follow(100, &[101 + MAX_ID_MISSES]);
        assert!(found.is_empty());
        assert_eq!(newest, 100);
    }

    #[test]
    fn asking_by_id_stops_at_the_largest_id_instead_of_overflowing() {
        let (found, newest, _) = follow(u32::MAX - 1, &[u32::MAX]);
        assert_eq!(found, vec![u32::MAX]);
        assert_eq!(newest, u32::MAX);
    }

    #[test]
    fn a_round_that_finds_news_is_followed_by_another_after_the_directory_is_still() {
        assert_eq!(next_step(true, false, false), Step::AskAgain);
        assert_eq!(next_step(true, true, true), Step::AskAgain);
    }

    #[test]
    fn a_wake_that_found_nothing_by_id_reads_the_whole_list_once() {
        assert_eq!(next_step(false, false, false), Step::ReadList);
        assert_eq!(next_step(false, false, true), Step::Done);
    }

    #[test]
    fn a_wake_whose_toast_was_found_at_the_first_change_does_not_read_the_whole_list() {
        assert_eq!(next_step(false, true, false), Step::Done);
    }
}
