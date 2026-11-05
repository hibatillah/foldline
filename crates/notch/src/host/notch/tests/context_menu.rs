//! The right-click menu opens on the open notch and goes away with it.

use super::*;

fn open_notch() -> Notch {
    let mut notch = rig(SampleOptions::default());
    open(&mut notch, "timer");
    notch
}

fn center(notch: &Notch) -> (i32, i32) {
    let bounds = notch.frame().bounds;
    (bounds.center_x() as i32, bounds.center_y() as i32)
}

#[test]
fn a_right_click_on_the_open_notch_opens_the_menu_and_the_next_one_closes_it() {
    let mut notch = open_notch();
    let (x, y) = center(&notch);
    notch.right_clicked(x, y);
    assert!(notch.menu.is_some());
    notch.right_clicked(x, y);
    assert!(notch.menu.is_none());
}

#[test]
fn a_right_click_on_the_collapsed_notch_opens_nothing() {
    let mut notch = open_notch();
    close(&mut notch);
    let (x, y) = center(&notch);
    notch.right_clicked(x, y);
    assert!(notch.menu.is_none());
}

#[test]
fn a_press_outside_the_menu_closes_it_without_reaching_the_page() {
    let mut notch = open_notch();
    let (x, y) = center(&notch);
    notch.right_clicked(x, y);
    let bounds = notch.frame().bounds;
    notch.pointer_pressed(bounds.left as i32 + 2, bounds.bottom as i32 - 2);
    assert!(notch.menu.is_none());
}

#[test]
fn the_menu_closes_when_the_pointer_leaves_the_notch() {
    let mut notch = open_notch();
    let (x, y) = center(&notch);
    notch.right_clicked(x, y);
    notch.set_hovering(false);
    assert!(notch.menu.is_none());
}
