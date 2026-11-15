use super::{PAGE_TIMER_BASE, PAGE_TIMER_LIMIT, TIMERS_PER_PAGE, TimerId};

#[cfg(test)]
pub(crate) fn timer_key(page_index: usize, timer: TimerId) -> usize {
    owned_timer_key(PAGE_TIMER_BASE, page_index, timer)
}

pub(crate) fn owned_timer_key(timer_base: usize, owner_index: usize, timer: TimerId) -> usize {
    debug_assert!(usize::from(timer.0) < TIMERS_PER_PAGE, "timer ids must be below {TIMERS_PER_PAGE}");
    timer_base + owner_index * TIMERS_PER_PAGE + usize::from(timer.0)
}

pub(crate) fn split_timer_key(key: usize) -> Option<(usize, TimerId)> {
    split_owned_timer_key(PAGE_TIMER_BASE, PAGE_TIMER_LIMIT, key)
}

pub(crate) fn split_owned_timer_key(timer_base: usize, limit: usize, key: usize) -> Option<(usize, TimerId)> {
    if key >= limit {
        return None;
    }
    let offset = key.checked_sub(timer_base)?;
    Some((offset / TIMERS_PER_PAGE, TimerId((offset % TIMERS_PER_PAGE) as u8)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_keys_round_trip_to_page_and_timer() {
        for (page, timer) in [(0, 0), (0, 15), (1, 0), (3, 7)] {
            let key = timer_key(page, TimerId(timer));
            assert_eq!(split_timer_key(key), Some((page, TimerId(timer))));
        }
    }

    #[test]
    fn timers_of_different_pages_never_collide() {
        assert_ne!(timer_key(0, TimerId(15)), timer_key(1, TimerId(0)));
    }

    #[test]
    fn keys_below_the_page_range_belong_to_the_host() {
        assert_eq!(split_timer_key(0), None);
        assert_eq!(split_timer_key(PAGE_TIMER_BASE - 1), None);
    }

    #[test]
    fn keys_from_the_activity_range_up_are_not_page_timers() {
        assert_eq!(split_timer_key(PAGE_TIMER_LIMIT), None);
        assert!(split_timer_key(PAGE_TIMER_LIMIT - 1).is_some());
    }
}
