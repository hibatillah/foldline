//! Pictures of the shelf, todo, shalat, weather and timer pages in each state of the design: cargo test page_state -- --ignored.

use super::{images::save, *};

#[test]
#[ignore = "writes images to target/snapshots; run on purpose"]
fn page_state_snapshots_of_the_shelf_todo_shalat_and_weather() {
    let scenarios: [(&str, &str, SampleOptions); 22] = [
        ("shelf", "state-shelf-empty", SampleOptions { shelf: 1, ..SampleOptions::default() }),
        ("shelf", "state-shelf-empty-drag", SampleOptions { shelf: 1, file_drag: true, ..SampleOptions::default() }),
        ("shelf", "state-shelf-seven", SampleOptions { shelf: 2, ..SampleOptions::default() }),
        ("shelf", "state-shelf-full-drag", SampleOptions { shelf: 3, file_drag: true, ..SampleOptions::default() }),
        ("todo", "state-todo-empty", SampleOptions { todo: 1, ..SampleOptions::default() }),
        ("todo", "state-todo-empty-adding", SampleOptions { todo: 2, ..SampleOptions::default() }),
        ("todo", "state-todo-full", SampleOptions { todo: 3, ..SampleOptions::default() }),
        ("todo", "state-todo-typing", SampleOptions { todo: 4, ..SampleOptions::default() }),
        ("todo", "state-todo-done", SampleOptions { todo: 5, ..SampleOptions::default() }),
        ("todo", "state-todo-deleted", SampleOptions { todo: 6, ..SampleOptions::default() }),
        ("todo", "state-todo-cleared", SampleOptions { todo: 7, ..SampleOptions::default() }),
        ("shalat", "state-shalat-soon", SampleOptions { shalat_mark: 3, ..SampleOptions::default() }),
        ("shalat", "state-shalat-after-isha", SampleOptions { shalat_mark: 4, ..SampleOptions::default() }),
        ("weather", "state-weather-days", SampleOptions { weather: 1, ..SampleOptions::default() }),
        ("weather", "state-weather-offline", SampleOptions { weather: 2, ..SampleOptions::default() }),
        ("weather", "state-weather-no-location", SampleOptions { weather: 4, ..SampleOptions::default() }),
        ("timer", "state-timer-idle", SampleOptions::default()),
        ("timer", "state-timer-running", SampleOptions { timer: true, ..SampleOptions::default() }),
        ("todo", "state-todo-sample", SampleOptions::default()),
        ("shelf", "state-shelf-six", SampleOptions::default()),
        ("shalat", "state-shalat-next", SampleOptions::default()),
        ("weather", "state-weather-hours", SampleOptions::default()),
    ];
    for (page_id, name, options) in scenarios {
        let mut notch = rig(options);
        open(&mut notch, page_id);
        save(&notch, name, 260.0);
    }
}

#[test]
#[ignore = "writes images to target/snapshots; run on purpose"]
fn page_state_snapshot_of_the_rain_alert_banner() {
    let notch = rig(SampleOptions { weather: 3, ..SampleOptions::default() });
    let page = notch.pages.iter().find(|page| page.id() == "weather").unwrap();
    let Some(Presence::Banner { width, height }) = page.presence() else { panic!("the sample shows the banner") };
    let surface = &notch.surface;
    let image_height = ((height + 20.0) * SCALE) as i32;
    let region = RECT { left: 0, top: 0, right: surface.width, bottom: image_height };
    images::clear(&notch);
    surface
        .render_and_present(notch.window, notch.screen_position, region, &notch.fonts, SCALE, |canvas| {
            let left = (surface.width as f32 - width * SCALE) / 2.0;
            let bottom = (height + RESTING_BOTTOM_PADDING) * SCALE;
            let bounds = Rect::new(left, 0.0, left + width * SCALE, bottom);
            let radius = 18.0 * SCALE;
            canvas.fill_rounded_rect(
                Rect::new(bounds.left, -radius, bounds.right, bounds.bottom),
                radius,
                Color::BLACK,
            );
            let area = Rect::new(bounds.left, bounds.top, bounds.right, bounds.bottom - RESTING_BOTTOM_PADDING * SCALE);
            crate::host::notch::units::draw_page_presence(page.as_ref(), canvas, area, 1.0);
        })
        .unwrap();
    images::write_image(
        "state-weather-rain-banner",
        surface.read_pixels(),
        surface.width as usize,
        (images::IMAGE_WIDTH * SCALE) as usize,
        image_height as usize,
    );
}
