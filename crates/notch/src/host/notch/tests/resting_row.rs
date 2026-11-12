//! The collapsed pill: the empty pill, the idle group and the lyric pill, and how their items move.

use super::*;
use crate::host::{claimed_row, clock, notch::row_motion::RowItem, pill_row};

fn clock_width(notch: &Notch) -> f32 {
    clock::layout_width(&notch.fonts)
}

/// Distance of the item's settled left edge from the pill's right edge.
fn resting_inset(notch: &Notch, item: RowItem) -> Option<f32> {
    notch.row_motion.resting.position(item)
}

fn badge(rank: u8, width: f32) -> (Mark, Rc<Cell<Option<Presence>>>) {
    let (mut mark, shared) = Mark::showing(Some(Presence::Badge { width }));
    mark.rank = rank;
    (mark, shared)
}

fn page_index(notch: &Notch, id: &str) -> usize {
    notch.pages.iter().position(|page| page.id() == id).unwrap()
}

#[test]
fn an_empty_pill_on_a_pc_centers_the_clock_and_hugs_it_narrower_than_150() {
    let notch = marked_rig(SampleOptions::default(), Vec::new());
    let width = clock_width(&notch) + 2.0 * pill_row::EMPTY_SIDE;
    assert_eq!(notch.springs.resting_width.target, width);
    assert!(width < crate::page::RESTING_MIN_WIDTH);
    assert_eq!(notch.row_motion.clock_left.target, pill_row::EMPTY_SIDE);
    assert_eq!(notch.idle_clock_center(), width / 2.0, "the clock sits in the middle");
}

#[test]
fn a_laptop_shows_the_battery_ring_at_the_right_edge_and_the_clock_at_the_left_in_a_150_pill() {
    let (ring, _) = Mark::showing(Some(RING));
    let notch = marked_rig(SampleOptions::default(), vec![ring]);
    assert_eq!(notch.springs.resting_width.target, crate::page::RESTING_MIN_WIDTH);
    assert_eq!(notch.row_motion.clock_left.target, pill_row::PILL_EDGE);
    let ring = RowItem::Source(Source::Activity(0));
    assert_eq!(resting_inset(&notch, ring), Some(pill_row::PILL_EDGE + 14.0));
}

#[test]
fn the_clock_glides_to_the_left_edge_when_the_first_item_appears_and_back_when_the_group_empties() {
    let (dot, shared) = Mark::showing(None);
    let mut notch = marked_rig(SampleOptions::default(), vec![dot]);
    assert_eq!(notch.row_motion.clock_left.position, pill_row::EMPTY_SIDE);
    shared.set(Some(DOT));
    notch.refresh_layout();
    assert_eq!(notch.row_motion.clock_left.target, pill_row::PILL_EDGE);
    assert_eq!(notch.row_motion.clock_left.position, pill_row::EMPTY_SIDE, "nothing jumps when the dot appears");
    assert_eq!(notch.springs.resting_width.target, crate::page::RESTING_MIN_WIDTH);
    assert!(notch.frame_timer.is_running);
    for _ in 0..12 {
        notch.advance_by(1.0 / 60.0);
    }
    let moving = notch.row_motion.clock_left.position;
    assert!(moving < pill_row::EMPTY_SIDE && moving > pill_row::PILL_EDGE, "on its way left: {moving}");
    settle(&mut notch);
    shared.set(None);
    notch.refresh_layout();
    assert_eq!(notch.row_motion.clock_left.target, pill_row::EMPTY_SIDE);
    assert_eq!(notch.row_motion.clock_left.position, pill_row::PILL_EDGE, "the way back starts where it was");
    assert_eq!(notch.springs.resting_width.target, clock_width(&notch) + 2.0 * pill_row::EMPTY_SIDE);
}

#[test]
fn a_dot_appearing_slides_the_ring_left_by_the_dot_and_the_4_between_them_instead_of_jumping() {
    let (ring, _) = Mark::showing(Some(RING));
    let (dot, shared) = Mark::showing(None);
    let mut notch = marked_rig(SampleOptions::default(), vec![ring, dot]);
    let ring = RowItem::Source(Source::Activity(0));
    let before = resting_inset(&notch, ring).unwrap();
    shared.set(Some(DOT));
    notch.refresh_layout();
    assert_eq!(resting_inset(&notch, ring), Some(before), "the ring has not jumped");
    assert_eq!(resting_inset(&notch, RowItem::Dots), Some(pill_row::PILL_EDGE + 6.0), "the dot starts at its place");
    notch.advance_by(1.0 / 30.0);
    notch.advance_by(1.0 / 30.0);
    assert!(resting_inset(&notch, ring).unwrap() > before, "the ring is sliding left");
    settle(&mut notch);
    assert_eq!(resting_inset(&notch, ring), Some(before + 6.0 + pill_row::RING_DOTS_GAP));
}

#[test]
fn the_dot_sits_4_after_the_battery_ring_in_the_idle_row() {
    let (ring, _) = Mark::showing(Some(RING));
    let (dot, _) = Mark::showing(Some(DOT));
    let notch = marked_rig(SampleOptions::default(), vec![ring, dot]);
    let (_, gaps) = notch.resting_row.spacing(&notch.idle_items(), pill_row::ITEM_GAP);
    assert_eq!(gaps, [pill_row::RING_DOTS_GAP]);
    let ring = RowItem::Source(Source::Activity(0));
    let expected = pill_row::PILL_EDGE + 6.0 + pill_row::RING_DOTS_GAP + 14.0;
    assert_eq!(resting_inset(&notch, ring), Some(expected));
}

#[test]
fn a_dot_after_a_badge_without_a_battery_ring_keeps_the_8_gap() {
    let (muted, _) = badge(2, 16.0);
    let (dot, _) = Mark::showing(Some(DOT));
    let notch = marked_rig(SampleOptions::default(), vec![muted, dot]);
    let (_, gaps) = notch.resting_row.spacing(&notch.idle_items(), pill_row::ITEM_GAP);
    assert_eq!(gaps, [pill_row::ITEM_GAP]);
    let muted = RowItem::Source(Source::Activity(0));
    let expected = pill_row::PILL_EDGE + 6.0 + pill_row::ITEM_GAP + 16.0;
    assert_eq!(resting_inset(&notch, muted), Some(expected));
}

#[test]
fn a_new_badge_joins_left_of_the_group_and_nothing_right_of_it_moves() {
    let (ring, _) = Mark::showing(Some(RING));
    let (timer_like, shared) = badge(3, 40.0);
    shared.set(None);
    let mut notch = marked_rig(SampleOptions::default(), vec![ring, timer_like]);
    let ring = RowItem::Source(Source::Activity(0));
    let before = resting_inset(&notch, ring);
    shared.set(Some(Presence::Badge { width: 40.0 }));
    settle(&mut notch);
    assert_eq!(resting_inset(&notch, ring), before);
    let joined = resting_inset(&notch, RowItem::Source(Source::Activity(1))).unwrap();
    assert_eq!(joined, before.unwrap() + pill_row::ITEM_GAP + 40.0);
}

#[test]
fn the_idle_group_runs_wave_caps_lock_muted_timer_event_prayer_then_battery_then_dots() {
    let (caps, _) = badge(1, 16.0);
    let (muted, _) = badge(2, 16.0);
    let (ring, _) = Mark::showing(Some(RING));
    let (dot, _) = Mark::showing(Some(DOT));
    let options =
        SampleOptions { media: Some(false), timer: true, calendar: true, shalat_mark: 1, ..SampleOptions::default() };
    let notch = marked_rig(options, vec![caps, muted, ring, dot]);
    let order: Vec<RowItem> = notch.idle_items().iter().map(|&(item, _)| item).collect();
    let page = |id| RowItem::Source(Source::Page(page_index(&notch, id)));
    let activity = |index| RowItem::Source(Source::Activity(index));
    let expected = [
        page("media"),
        activity(0),
        activity(1),
        page("timer"),
        page("calendar"),
        page("shalat"),
        activity(2),
        RowItem::Dots,
    ];
    assert_eq!(order, expected);
    let insets: Vec<f32> = order.iter().map(|&item| resting_inset(&notch, item).unwrap()).collect();
    assert!(insets.windows(2).all(|pair| pair[0] > pair[1]), "left to right in that order: {insets:?}");
    assert_eq!(notch.row_motion.clock_left.target, pill_row::PILL_EDGE);
}

#[test]
fn several_badges_with_text_take_their_short_form_and_a_lone_one_keeps_the_long_form() {
    let mut alone = rig(SampleOptions { timer: true, ..SampleOptions::default() });
    settle(&mut alone);
    assert!(!alone.resting_row.uses_short_badges);
    let mut several = rig(SampleOptions { timer: true, calendar: true, shalat_mark: 1, ..SampleOptions::default() });
    settle(&mut several);
    assert!(several.resting_row.uses_short_badges);
    for &(source, width) in &several.presences.badges {
        let Source::Page(index) = source else { continue };
        if let Some(short) = several.pages[index].short_presence_width() {
            assert_eq!(width, short, "{} takes its short form", several.pages[index].id());
        }
    }
}

#[test]
fn music_without_a_lyric_line_is_a_wave_badge_and_does_not_claim_the_pill() {
    let mut notch = playing(false);
    settle(&mut notch);
    assert_eq!(notch.resting_index, None);
    assert_eq!(notch.presences.badges.len(), 1);
    assert!(notch.has_music_badge());
}

fn lyric_rig(dot: Option<Presence>) -> (Notch, Rc<Cell<Option<Presence>>>) {
    let (caps, _) = badge(1, 16.0);
    let (muted, _) = badge(2, 16.0);
    let (ring, _) = Mark::showing(Some(RING));
    let (dot, shared) = Mark::showing(dot);
    let options =
        SampleOptions { media: Some(true), timer: true, calendar: true, shalat_mark: 1, ..SampleOptions::default() };
    (marked_rig(options, vec![caps, muted, ring, dot]), shared)
}

#[test]
fn the_lyric_pill_shows_muted_then_caps_lock_then_only_the_countdown_with_the_least_time_left() {
    let (notch, _) = lyric_rig(None);
    assert!(notch.resting_index.is_some(), "the lyric line claims the pill");
    let countdowns =
        ["timer", "calendar", "shalat"].map(|id| (notch.pages[page_index(&notch, id)].countdown_seconds(), id));
    let nearest = countdowns.iter().filter_map(|&(seconds, id)| seconds.map(|seconds| (seconds, id))).min().unwrap().1;
    let sources: Vec<Source> = notch.resting_row.claimed_group.iter().map(|&(source, _)| source).collect();
    assert_eq!(sources, [Source::Activity(1), Source::Activity(0), Source::Page(page_index(&notch, nearest))]);
}

#[test]
fn the_lyric_pill_has_no_clock_no_battery_and_no_wave_and_grows_to_fit_its_group() {
    let (notch, _) = lyric_rig(None);
    let index = notch.resting_index.unwrap();
    let page_width = notch.pages[index].sizes().resting_width;
    let widths: Vec<f32> = notch.resting_row.claimed_group.iter().map(|&(_, width)| width).collect();
    let group = widths.iter().sum::<f32>() + 2.0 * pill_row::ITEM_GAP;
    assert_eq!(notch.springs.resting_width.target, 12.0 + 20.0 + 24.0 + page_width + 24.0 + group + 12.0);
    assert_eq!(resting_inset(&notch, RowItem::Source(Source::Activity(2))), None, "the battery ring is not drawn");
    assert!(!notch.has_music_badge());
}

#[test]
fn the_lyric_pill_shows_the_countdown_as_its_icon_alone() {
    let (notch, _) = lyric_rig(None);
    let &(source, width) = notch.resting_row.claimed_group.last().unwrap();
    let Source::Page(index) = source else { panic!("the countdown is a page badge") };
    assert_eq!(notch.pages[index].header_status_width(), Some(width));
    assert_ne!(notch.pages[index].presence(), Some(Presence::Badge { width }), "the badge's text is left out");
}

#[test]
fn a_dot_in_the_lyric_pill_sits_at_the_far_right_as_in_the_idle_row() {
    let (notch, _) = lyric_rig(Some(DOT));
    assert_eq!(resting_inset(&notch, RowItem::Dots), Some(pill_row::PILL_EDGE + 6.0));
}

#[test]
fn a_dot_in_the_lyric_pill_sits_8_after_the_countdown() {
    let (notch, _) = lyric_rig(Some(DOT));
    let &(countdown, width) = notch.resting_row.claimed_group.last().unwrap();
    let expected = pill_row::PILL_EDGE + 6.0 + pill_row::ITEM_GAP + width;
    assert_eq!(resting_inset(&notch, RowItem::Source(countdown)), Some(expected));
}

#[test]
fn a_dot_in_the_lyric_pill_leaves_the_line_24_after_the_artwork() {
    let (notch, _) = lyric_rig(Some(DOT));
    assert_eq!(notch.resting_row.claim.lead_gap(), claimed_row::CONTENT_GAP);
    assert_eq!(notch.resting_row.claim.left_inset(), pill_row::PILL_EDGE);
}

#[test]
fn without_a_dot_the_lyric_pill_reserves_no_dot_column() {
    let (notch, _) = lyric_rig(None);
    assert_eq!(resting_inset(&notch, RowItem::Dots), None);
    assert_eq!(notch.resting_row.claim.lead_gap(), claimed_row::CONTENT_GAP);
    let &(countdown, width) = notch.resting_row.claimed_group.last().unwrap();
    assert_eq!(resting_inset(&notch, RowItem::Source(countdown)), Some(pill_row::PILL_EDGE + width));
}

#[test]
fn a_dot_appearing_in_the_lyric_pill_widens_it_and_slides_the_group_left_while_the_line_keeps_its_place() {
    let (mut notch, shared) = lyric_rig(None);
    let &(countdown, _) = notch.resting_row.claimed_group.last().unwrap();
    let countdown = RowItem::Source(countdown);
    let (before_inset, before_width) = (resting_inset(&notch, countdown).unwrap(), notch.springs.resting_width.target);
    let dot_room = pill_row::ITEM_GAP + 6.0;
    shared.set(Some(DOT));
    notch.refresh_layout();
    assert_eq!(notch.springs.resting_width.target, before_width + dot_room);
    assert_eq!(resting_inset(&notch, countdown), Some(before_inset), "the group has not jumped");
    assert!(notch.frame_timer.is_running);
    settle(&mut notch);
    assert_eq!(resting_inset(&notch, countdown), Some(before_inset + dot_room), "the dot takes the right end");
    assert_eq!(notch.springs.claim_lead_gap.position, claimed_row::CONTENT_GAP, "the line stays after the artwork");
}
