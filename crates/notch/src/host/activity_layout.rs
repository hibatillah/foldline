//! Newer transients take priority; older ones wait for space.

/// Transients on screen at once; older ones wait until a newer one ends.
pub(super) const MAX_TRANSIENTS: usize = 2;
pub(super) const DIVIDER_WIDTH: f32 = 1.0;

/// Keeps `order` as the activation order of the activities in `active`: ended ones leave, new ones join at the end.
pub(super) fn track_order<T: Copy + PartialEq>(order: &mut Vec<T>, active: &[T]) {
    order.retain(|item| active.contains(item));
    for &item in active {
        if !order.contains(&item) {
            order.push(item);
        }
    }
}

/// The newest transients that fit on screen, oldest first.
pub(super) fn newest<T>(order: &[T]) -> &[T] {
    &order[order.len().saturating_sub(MAX_TRANSIENTS)..]
}

pub(super) fn total_width(widths: impl IntoIterator<Item = f32>) -> f32 {
    let (sum, count) = widths.into_iter().fold((0.0, 0usize), |(sum, count), width| (sum + width, count + 1));
    sum + DIVIDER_WIDTH * count.saturating_sub(1) as f32
}

/// The left edge of each transient from the first one's, lazily so a paint allocates nothing.
pub(super) fn offsets(widths: impl IntoIterator<Item = f32>) -> impl Iterator<Item = f32> {
    widths.into_iter().scan(0.0, |left, width| {
        let start = *left;
        *left += width + DIVIDER_WIDTH;
        Some(start)
    })
}

/// The debug-log line naming what the collapsed pill shows, so a wake nobody asked for can be traced to its source.
pub(super) fn presence_summary(transients: &[&str], badges: &[&str], banner: Option<&str>) -> String {
    format!(
        "pill shows transients=[{}] badges=[{}] banner={}",
        transients.join(","),
        badges.join(","),
        banner.unwrap_or("none")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pill_summary_names_every_source_by_its_id() {
        assert_eq!(
            presence_summary(&["bluetooth", "volume"], &["battery"], Some("notifications")),
            "pill shows transients=[bluetooth,volume] badges=[battery] banner=notifications"
        );
    }

    #[test]
    fn an_empty_pill_summary_says_there_is_no_banner() {
        assert_eq!(presence_summary(&[], &[], None), "pill shows transients=[] badges=[] banner=none");
    }

    #[test]
    fn a_new_activity_joins_the_end_and_an_ended_one_leaves() {
        let mut order = vec![2, 0];
        track_order(&mut order, &[0, 1]);
        assert_eq!(order, vec![0, 1]);
    }

    #[test]
    fn the_order_of_activation_is_kept_while_activities_stay() {
        let mut order = vec![3, 1];
        track_order(&mut order, &[1, 3]);
        assert_eq!(order, vec![3, 1]);
    }

    #[test]
    fn only_the_newest_transients_are_shown() {
        assert_eq!(newest(&[4, 5, 6]), &[5, 6]);
        assert_eq!(newest(&[4]), &[4]);
        assert!(newest::<usize>(&[]).is_empty());
    }

    #[test]
    fn dividers_sit_between_transients_only() {
        assert_eq!(total_width([100.0]), 100.0);
        assert_eq!(total_width([100.0, 50.0]), 100.0 + DIVIDER_WIDTH + 50.0);
        assert_eq!(total_width([]), 0.0);
    }

    #[test]
    fn each_transient_starts_after_the_previous_one_and_its_divider() {
        assert_eq!(offsets([100.0, 50.0]).collect::<Vec<_>>(), vec![0.0, 100.0 + DIVIDER_WIDTH]);
    }
}
