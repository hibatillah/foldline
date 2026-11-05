//! Offscreen images go to target/snapshots; run cargo test snapshots -- --ignored --nocapture.

use super::*;

const IMAGE_WIDTH: f32 = 700.0;

fn output_directory() -> PathBuf {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/snapshots");
    fs::create_dir_all(&directory).unwrap();
    directory
}

fn background(fraction: f32) -> [f32; 3] {
    let stops = [(0.0, [214.0, 164.0, 230.0]), (0.6, [240.0, 138.0, 75.0]), (1.0, [239.0, 106.0, 42.0])];
    let (low, high) = if fraction < stops[1].0 { (stops[0], stops[1]) } else { (stops[1], stops[2]) };
    let amount = ((fraction - low.0) / (high.0 - low.0)).clamp(0.0, 1.0);
    std::array::from_fn(|channel| low.1[channel] + (high.1[channel] - low.1[channel]) * amount)
}

/// Composite cropped premultiplied BGRA over the background and write a 24-bit bitmap.
fn write_image(name: &str, pixels: &[u8], surface_width: usize, width: usize, height: usize) {
    let left = (surface_width - width) / 2;
    let row_bytes = (width * 3).div_ceil(4) * 4;
    let mut file = Vec::with_capacity(54 + row_bytes * height);
    file.extend_from_slice(b"BM");
    file.extend_from_slice(&((54 + row_bytes * height) as u32).to_le_bytes());
    file.extend_from_slice(&[0, 0, 0, 0]);
    file.extend_from_slice(&54u32.to_le_bytes());
    file.extend_from_slice(&40u32.to_le_bytes());
    file.extend_from_slice(&(width as i32).to_le_bytes());
    file.extend_from_slice(&(height as i32).to_le_bytes());
    file.extend_from_slice(&1u16.to_le_bytes());
    file.extend_from_slice(&24u16.to_le_bytes());
    file.extend_from_slice(&[0; 24]);
    for row in (0..height).rev() {
        let start = file.len();
        for column in 0..width {
            let source = (row * surface_width + left + column) * 4;
            let alpha = f32::from(pixels[source + 3]) / 255.0;
            let behind = background(column as f32 / width as f32);
            // BGR order; the source is premultiplied, so what shows through is scaled by the missing alpha.
            for channel in [0usize, 1, 2] {
                let covered = f32::from(pixels[source + channel]);
                file.push((covered + behind[2 - channel] * (1.0 - alpha)).round().clamp(0.0, 255.0) as u8);
            }
        }
        file.resize(start + row_bytes, 0);
    }
    fs::write(output_directory().join(format!("{name}.bmp")), file).unwrap();
}

fn clear(notch: &Notch) {
    let surface = &notch.surface;
    let region = RECT { left: 0, top: 0, right: surface.width, bottom: surface.height };
    surface.render_and_present(notch.window, notch.screen_position, region, &notch.fonts, SCALE, |_| {}).unwrap();
}

fn save(notch: &Notch, name: &str, height_dip: f32) {
    clear(notch);
    notch.paint();
    let surface = &notch.surface;
    let height = ((height_dip * SCALE) as usize).min(surface.height as usize);
    write_image(name, surface.read_pixels(), surface.width as usize, (IMAGE_WIDTH * SCALE) as usize, height);
}

#[test]
#[ignore = "writes images to target/snapshots; run on purpose"]
fn snapshots_of_every_open_page() {
    let options = SampleOptions { media: Some((true, false)), timer: true, calendar: true, ..SampleOptions::default() };
    let mut notch = rig(options);
    for id in ["media", "shelf", "calendar", "timer", "stats", "todo", "shalat"] {
        open(&mut notch, id);
        save(&notch, &format!("open-{id}"), WINDOW_HEIGHT);
    }
    open(&mut notch, "media");
    notch.holds_page_position = true;
    for (name, progress) in [("swipe-long-short-25", 0.25), ("swipe-long-short-50", 0.5), ("swipe-long-short-75", 0.75)]
    {
        notch.track.jump_to(progress);
        notch.refresh_layout();
        notch.follow_page_width();
        save(&notch, name, 260.0);
    }
    notch.track.jump_to(0.5);
    notch.follow_page_width();
    save(&notch, "open-swipe", 260.0);
    notch.drag_pages(0.0);
    notch.end_swipe();
    notch.refresh_layout();
    save(&notch, "open-swipe-release", 260.0);
    let mut plain = rig(SampleOptions { media: Some((false, false)), ..options });
    open(&mut plain, "media");
    save(&plain, "open-media-plain", 260.0);
    open(&mut notch, "media");
    for (name, expansion) in [("stage-header", HEADER_STAGE), ("stage-halfway", 0.75)] {
        notch.springs.expansion.jump_to(expansion);
        notch.refresh_layout();
        save(&notch, name, 260.0);
    }
    let mut idle = rig(SampleOptions::default());
    open(&mut idle, "media");
    save(&idle, "open-media-empty", 260.0);
    let mut dragged_over = rig(SampleOptions { file_drag: true, ..options });
    open(&mut dragged_over, "shelf");
    save(&dragged_over, "open-shelf-drag", 260.0);
    open(&mut notch, "timer");
    notch.page_list.set_enabled(4, false);
    notch.editor = Some(PageEditor::default());
    notch.panel.tab = Tab::Pages;
    settle(&mut notch);
    save(&notch, "open-edit-pages", WINDOW_HEIGHT);
    for (tab, name) in [(Tab::General, "open-settings-general"), (Tab::Behavior, "open-settings-behavior")] {
        notch.panel.tab = tab;
        settle(&mut notch);
        save(&notch, name, WINDOW_HEIGHT);
    }
    for (id, name) in [
        ("timer", "open-settings-timer"),
        ("calendar", "open-settings-calendar"),
        ("stats", "open-settings-stats"),
        ("shalat", "open-settings-shalat"),
    ] {
        open(&mut notch, id);
        notch.open_panel();
        settle(&mut notch);
        save(&notch, name, WINDOW_HEIGHT);
        notch.editor = None;
    }
}

#[test]
#[ignore = "writes images to target/snapshots; run on purpose"]
fn snapshots_of_the_collapsed_pill() {
    let scenarios: [(&str, SampleOptions); 9] = [
        ("resting-idle", SampleOptions::default()),
        ("resting-idle-with-timer", SampleOptions { timer: true, ..SampleOptions::default() }),
        ("resting-idle-with-calendar", SampleOptions { calendar: true, ..SampleOptions::default() }),
        ("resting-media-bars", SampleOptions { media: Some((false, false)), ..SampleOptions::default() }),
        ("resting-media-lyrics", SampleOptions { media: Some((true, false)), ..SampleOptions::default() }),
        ("resting-media-title", SampleOptions { media: Some((false, true)), ..SampleOptions::default() }),
        (
            "resting-media-with-timer",
            SampleOptions { media: Some((false, false)), timer: true, ..SampleOptions::default() },
        ),
        ("resting-shalat-mark", SampleOptions { shalat_mark: 1, ..SampleOptions::default() }),
        ("resting-shalat-message", SampleOptions { shalat_mark: 2, ..SampleOptions::default() }),
    ];
    for (name, options) in scenarios {
        let mut notch = rig(options);
        close(&mut notch);
        save(&notch, name, 40.0);
    }
}

fn pill(notch: &Notch, name: &str, width: f32, content: impl FnOnce(&Canvas, Rect)) {
    let surface = &notch.surface;
    let scale = SCALE;
    let region = RECT { left: 0, top: 0, right: surface.width, bottom: (40.0 * scale) as i32 };
    clear(notch);
    let result =
        surface.render_and_present(notch.window, notch.screen_position, region, &notch.fonts, scale, |canvas| {
            let left = (surface.width as f32 - width * scale) / 2.0;
            let bounds = Rect::new(left, 0.0, left + width * scale, RESTING_HEIGHT * scale);
            canvas.fill_rounded_rect(
                Rect::new(bounds.left, -14.0 * scale, bounds.right, bounds.bottom),
                14.0 * scale,
                Color::BLACK,
            );
            canvas.fill_top_flares(bounds, RESTING_FLARE * scale, Color::BLACK);
            content(canvas, bounds);
        });
    result.unwrap();
    write_image(
        name,
        surface.read_pixels(),
        surface.width as usize,
        (IMAGE_WIDTH * SCALE) as usize,
        (40.0 * scale) as usize,
    );
}

#[test]
#[ignore = "writes images to target/snapshots; run on purpose"]
fn snapshot_of_the_notification_banner() {
    let notch = rig(SampleOptions::default());
    banner_snapshot(&notch, "banner-notification");
}

#[test]
#[ignore = "writes images to target/snapshots; run on purpose"]
fn snapshot_of_the_hovered_notification_banner() {
    let notch = rig(SampleOptions::default());
    let body = "Warp finished the task you gave it. The summary has the changed files, the checks that ran and what is left to do before the release goes out. ".repeat(2);
    let activity = crate::activities::notifications::NotificationsActivity::sample_with_body(&notch.fonts, &body);
    banner_snapshot_of(&notch, Box::new(activity.hovered()), "banner-notification-hovered");
}

fn banner_snapshot(notch: &Notch, name: &str) {
    banner_snapshot_of(notch, crate::activities::sample_notification(&notch.fonts), name);
}

fn banner_snapshot_of(notch: &Notch, activity: Box<dyn Activity>, name: &str) {
    let Some(Presence::Banner { width, height }) = activity.presence() else { panic!("the sample has a banner") };
    let surface = &notch.surface;
    let scale = SCALE;
    let image_height = ((height + 20.0) * scale) as i32;
    let region = RECT { left: 0, top: 0, right: surface.width, bottom: image_height };
    clear(notch);
    surface
        .render_and_present(notch.window, notch.screen_position, region, &notch.fonts, scale, |canvas| {
            let left = (surface.width as f32 - width * scale) / 2.0;
            let bottom = (height + RESTING_BOTTOM_PADDING) * scale;
            let bounds = Rect::new(left, 0.0, left + width * scale, bottom);
            let radius = 18.0 * scale;
            canvas.fill_rounded_rect(
                Rect::new(bounds.left, -radius, bounds.right, bounds.bottom),
                radius,
                Color::BLACK,
            );
            canvas.fill_top_flares(bounds, RESTING_FLARE * scale, Color::BLACK);
            let area = Rect::new(bounds.left, bounds.top, bounds.right, bounds.bottom - RESTING_BOTTOM_PADDING * scale);
            activity.draw(canvas, area, scale, 1.0);
        })
        .unwrap();
    write_image(
        name,
        surface.read_pixels(),
        surface.width as usize,
        (IMAGE_WIDTH * SCALE) as usize,
        image_height as usize,
    );
}

#[test]
#[ignore = "writes images to target/snapshots; run on purpose"]
fn snapshots_of_the_short_lived_messages() {
    let notch = rig(SampleOptions::default());
    let fonts = &notch.fonts;
    let green = Color::rgb(0.19, 0.82, 0.35);
    let orange = Color::rgb(1.0, 0.62, 0.04);
    let blue = Color::rgb(0.302, 0.671, 0.969);
    let level_width = ui::LEVEL_WIDTH;
    pill(&notch, "hud-volume", level_width, |canvas, area| {
        ui::draw_level(canvas, area, 1.0, Icon::Speaker, 0.42, "42")
    });
    let muted_width = ui::message_width(fonts, "Muted", None);
    pill(&notch, "hud-muted", muted_width, |canvas, area| {
        ui::draw_message(canvas, area, 1.0, Icon::SpeakerMuted, Color::WHITE, "Muted", None)
    });
    pill(&notch, "hud-brightness", level_width, |canvas, area| ui::draw_level(canvas, area, 1.0, Icon::Sun, 0.7, "70"));
    let messages: [(&str, Icon, Color, &str, Option<&str>); 9] = [
        ("hud-charging", Icon::Bolt, green, "Charging", Some("56%")),
        ("hud-on-battery", Icon::Battery, Color::WHITE, "On battery", Some("56%")),
        ("hud-low-battery", Icon::BatteryLow, Color::rgb(1.0, 0.27, 0.23), "Low battery", Some("20%")),
        ("hud-bluetooth", Icon::Bluetooth, blue, "DAXA SL1 connected", Some("80%")),
        ("hud-microphone", Icon::Microphone, orange, "Microphone on", None),
        ("hud-camera", Icon::Camera, green, "Camera on", None),
        ("hud-copied", Icon::Copy, Color::WHITE, "Copied", None),
        ("hud-caps-lock", Icon::CapsLock, Color::WHITE, "Caps Lock on", None),
        ("hud-timer", Icon::Bell, green, "Focus session done", Some("25 min")),
    ];
    for (name, icon, color, message, value) in messages {
        let width = ui::message_width(fonts, message, value);
        pill(&notch, name, width, |canvas, area| ui::draw_message(canvas, area, 1.0, icon, color, message, value));
    }
}
