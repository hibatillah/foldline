//! The row springs: items slide when others come and go, and the frame timer stops once they settle.

use super::*;
use crate::host::notch::row_motion::RowItem;

/// Runs frames until the frame timer stops; the seconds it took, or `None` when it still runs after four seconds.
fn seconds_until_the_timer_stops(notch: &mut Notch) -> Option<f32> {
    notch.start_animation();
    let frame_seconds = 1.0 / 60.0;
    for frame in 0..240 {
        notch.advance_by(frame_seconds);
        if !notch.frame_timer.is_running {
            return Some((frame + 1) as f32 * frame_seconds);
        }
    }
    None
}

#[test]
fn the_frame_timer_stops_after_dots_badges_and_the_battery_come_and_go() {
    let (ring, ring_presence) = Mark::showing(Some(RING));
    let (dot, dot_presence) = Mark::showing(None);
    let (badge, badge_presence) = Mark::showing(None);
    let mut notch = marked_rig(SampleOptions { timer: true, ..SampleOptions::default() }, vec![ring, dot, badge]);
    let changes = [
        (&dot_presence, Some(DOT)),
        (&badge_presence, Some(Presence::Badge { width: 30.0 })),
        (&ring_presence, None),
        (&dot_presence, None),
        (&badge_presence, None),
    ];
    for (presence, value) in changes {
        presence.set(value);
        notch.refresh_layout();
        assert!(seconds_until_the_timer_stops(&mut notch).is_some(), "the springs settle after {value:?}");
    }
}

#[test]
fn the_frame_timer_stops_when_a_dot_flickers_faster_than_the_springs_settle() {
    let (ring, _) = Mark::showing(Some(RING));
    let (dot, dot_presence) = Mark::showing(None);
    let mut notch = marked_rig(SampleOptions::default(), vec![ring, dot]);
    notch.start_animation();
    for flicker in 0..10 {
        dot_presence.set(if flicker % 2 == 0 { Some(DOT) } else { None });
        notch.refresh_layout();
        for _ in 0..5 {
            notch.advance_by(1.0 / 60.0);
        }
    }
    assert!(seconds_until_the_timer_stops(&mut notch).is_some());
}

#[test]
fn opening_slides_the_header_icons_into_place_when_one_joins_while_open() {
    let (mut icon, presence) = Mark::showing(None);
    icon.header_width = None;
    let (ring, _) = Mark::showing(Some(RING));
    let mut notch = marked_rig(SampleOptions { timer: true, ..SampleOptions::default() }, vec![icon, ring]);
    open(&mut notch, "media");
    let timer = notch.pages.iter().position(|page| page.id() == "timer").unwrap();
    let timer = RowItem::Source(Source::Page(timer));
    let before = notch.row_motion.header.position(timer).unwrap();
    presence.set(Some(DOT));
    notch.refresh_layout();
    assert_eq!(notch.row_motion.header.position(timer), Some(before), "the timer icon has not jumped");
    settle(&mut notch);
    let after = notch.row_motion.header.position(timer).unwrap();
    assert_eq!(
        after,
        before + 6.0 + crate::host::pill_row::RING_DOTS_GAP,
        "it moved left by the dot and the 4 after the ring"
    );
}
