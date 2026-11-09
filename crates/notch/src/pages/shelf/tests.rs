use super::*;

#[test]
fn the_shelf_is_a_page_to_open_that_takes_drops() {
    let page = ShelfPage::new();
    assert!(page.accepts_drops());
    assert_eq!(page.priority(Mode::Resting), None);
    assert!(page.priority(Mode::Expanded).is_some());
    assert_eq!(page.presence(), None);
}

#[test]
fn a_new_file_arrives_and_then_the_page_stops_animating() {
    let mut page = ShelfPage::new();
    let mut spring = Spring::new(0.0, ARRIVAL_RESPONSE.0, ARRIVAL_RESPONSE.1);
    spring.target = 1.0;
    page.arrivals.insert(PathBuf::from("a.txt"), spring);
    assert!(page.animate(0.016));
    for _ in 0..200 {
        page.animate(0.016);
    }
    assert!(!page.animate(0.016));
    assert!(page.arrivals.is_empty());
}
