//! Offscreen images go to target/snapshots; run cargo test snapshots -- --ignored --nocapture.

use super::*;
use crate::{host::pill_row, page::PointerEvent};

pub(super) const IMAGE_WIDTH: f32 = 700.0;

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
pub(super) fn write_image(name: &str, pixels: &[u8], surface_width: usize, width: usize, height: usize) {
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

pub(super) fn clear(notch: &Notch) {
    let surface = &notch.surface;
    let region = RECT { left: 0, top: 0, right: surface.width, bottom: surface.height };
    surface.render_and_present(notch.window, notch.screen_position, region, &notch.fonts, SCALE, |_| {}).unwrap();
}

pub(super) fn save(notch: &Notch, name: &str, height_dip: f32) {
    clear(notch);
    notch.paint();
    let surface = &notch.surface;
    let height = ((height_dip * SCALE) as usize).min(surface.height as usize);
    write_image(name, surface.read_pixels(), surface.width as usize, (IMAGE_WIDTH * SCALE) as usize, height);
}

#[test]
#[ignore = "writes images to target/snapshots; run on purpose"]
fn snapshots_of_every_open_page() {
    let options = SampleOptions { media: Some(true), timer: true, calendar: true, ..SampleOptions::default() };
    let mut notch = rig(options);
    for id in ["media", "shelf", "calendar", "timer", "stats", "todo", "shalat", "weather", "devices"] {
        open(&mut notch, id);
        save(&notch, &format!("open-{id}"), WINDOW_HEIGHT);
    }
    open(&mut notch, "media");
    save_swap_frames(&mut notch, "calendar", "swap-media-calendar");
    let mut plain = rig(SampleOptions { media: Some(false), ..options });
    open(&mut plain, "media");
    save(&plain, "open-media-plain", 260.0);
    open(&mut notch, "media");
    for (name, expansion) in [("stage-header", 0.5), ("stage-halfway", 0.75)] {
        notch.springs.expansion.jump_to(expansion);
        notch.refresh_layout();
        save(&notch, name, 260.0);
    }
    let mut idle = rig(SampleOptions::default());
    open(&mut idle, "media");
    save(&idle, "open-media-empty", 260.0);
    // No linked calendar: no dots, so the week is lower and the message fills the page.
    let mut unlinked = rig(SampleOptions { calendar: false, ..options });
    open(&mut unlinked, "calendar");
    save(&unlinked, "open-calendar-no-link", 260.0);
    let mut week = rig(SampleOptions { calendar_week: true, ..options });
    open(&mut week, "calendar");
    save(&week, "open-calendar-week", 260.0);
    let mut dragged_over = rig(SampleOptions { file_drag: true, ..options });
    open(&mut dragged_over, "shelf");
    save(&dragged_over, "open-shelf-drag", 260.0);
}

/// Frames of a swap from the open page to `page_id`, stepped like the frame timer so size and content move together.
fn save_swap_frames(notch: &mut Notch, page_id: &str, name: &str) {
    let target = notch.pages.iter().position(|page| page.id() == page_id).unwrap();
    notch.go_to_page(target);
    notch.flush();
    let mut elapsed = 0.0;
    for milliseconds in [0, 40, 80, 120, 160, 220, 300, 420] {
        while elapsed + 0.0005 < milliseconds as f32 / 1000.0 {
            notch.advance_by(1.0 / 240.0);
            elapsed += 1.0 / 240.0;
        }
        save(notch, &format!("{name}-{milliseconds:03}ms"), WINDOW_HEIGHT);
    }
    settle(notch);
}

#[test]
#[ignore = "writes images to target/snapshots; run on purpose"]
fn snapshots_of_the_devices_page_states() {
    let states = ["normal", "attention", "unknown-battery", "many", "no-devices", "bluetooth-off"];
    for (scenario, name) in states.into_iter().enumerate() {
        let mut notch = rig(SampleOptions { devices: scenario as u8, ..SampleOptions::default() });
        open(&mut notch, "devices");
        save(&notch, &format!("devices-{name}"), WINDOW_HEIGHT);
    }
}

#[test]
#[ignore = "writes images to target/snapshots; run on purpose"]
fn snapshots_of_the_collapsed_pill() {
    let scenarios: [(&str, SampleOptions); 11] = [
        ("resting-idle", SampleOptions::default()),
        ("resting-idle-with-timer", SampleOptions { timer: true, ..SampleOptions::default() }),
        ("resting-idle-with-calendar", SampleOptions { calendar: true, ..SampleOptions::default() }),
        ("resting-media-bars", SampleOptions { media: Some(false), ..SampleOptions::default() }),
        ("resting-media-lyrics", SampleOptions { media: Some(true), ..SampleOptions::default() }),
        (
            "resting-media-lyrics-with-timer",
            SampleOptions {
                media: Some(true),
                timer: true,
                calendar: true,
                shalat_mark: 1,
                ..SampleOptions::default()
            },
        ),
        (
            "resting-idle-with-group",
            SampleOptions {
                media: Some(false),
                timer: true,
                calendar: true,
                shalat_mark: 1,
                ..SampleOptions::default()
            },
        ),
        ("resting-media-with-timer", SampleOptions { media: Some(false), timer: true, ..SampleOptions::default() }),
        ("resting-shalat-mark", SampleOptions { shalat_mark: 1, ..SampleOptions::default() }),
        ("resting-shalat-message", SampleOptions { shalat_mark: 2, ..SampleOptions::default() }),
        ("resting-weather-rain-badge", SampleOptions { weather: 5, ..SampleOptions::default() }),
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

#[test]
#[ignore = "writes images to target/snapshots; run on purpose"]
fn snapshots_of_the_download_banners() {
    let notch = rig(SampleOptions::default());
    banner_snapshot_of(&notch, crate::activities::sample_download_started(&notch.fonts), "banner-download-started");
    banner_snapshot_of(&notch, crate::activities::sample_download_finished(&notch.fonts), "banner-download-finished");
    let hovered = crate::activities::sample_download_finished_hovered(&notch.fonts, |image| {
        notch.surface.create_image_brush(image).ok()
    });
    banner_snapshot_of(&notch, hovered, "banner-download-finished-hovered");
}

#[test]
#[ignore = "writes images to target/snapshots; run on purpose"]
fn snapshots_of_the_running_download_badge_in_the_pill_and_the_open_header() {
    let mut notch = configured_rig(
        SampleOptions { timer: true, ..SampleOptions::default() },
        Settings::at(None),
        vec![crate::activities::sample_download_running()],
    );
    notch.activities[0].is_started = true;
    close(&mut notch);
    save(&notch, "resting-download-badge", 40.0);
    open(&mut notch, "timer");
    save(&notch, "open-header-download-badge", 160.0);
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
            crate::host::notch::units::draw_activity(activity.as_ref(), canvas, area, 1.0);
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
    for (name, is_muted) in [("hud-volume", false), ("hud-muted", true)] {
        let volume = crate::activities::sample_volume(0.42, is_muted);
        let Some(Presence::Transient { width }) = volume.presence() else { panic!("the sample shows its level") };
        pill(&notch, name, width, |canvas, area| {
            crate::host::notch::units::draw_activity(volume.as_ref(), canvas, area, 1.0)
        });
    }
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

#[test]
#[ignore = "writes images to target/snapshots; run on purpose"]
fn snapshots_of_the_idle_row_and_the_open_header() {
    let group =
        SampleOptions { media: Some(false), timer: true, calendar: true, shalat_mark: 1, ..SampleOptions::default() };
    let marks = |presences: &[Presence]| -> Vec<Mark> {
        presences.iter().map(|&presence| Mark::showing(Some(presence)).0).collect()
    };
    for (name, options, presences) in [
        ("resting-empty", SampleOptions::default(), &[][..]),
        ("resting-battery", SampleOptions::default(), &[RING][..]),
        ("resting-battery-and-dot", SampleOptions::default(), &[RING, DOT][..]),
        ("resting-group-battery-and-dot", group, &[RING, DOT][..]),
        ("resting-group-on-a-pc", group, &[][..]),
        ("resting-lyrics-with-group-and-dot", SampleOptions { media: Some(true), ..group }, &[RING, DOT][..]),
    ] {
        let mut notch = marked_rig(options, marks(presences));
        close(&mut notch);
        save(&notch, name, 40.0);
    }
    let mut notch = marked_rig(group, marks(&[RING, DOT]));
    open(&mut notch, "timer");
    save(&notch, "open-header-with-group-battery-and-dot", 160.0);
    notch.open.is_pinned = true;
    save(&notch, "open-header-with-group-battery-and-dot-pinned", 160.0);
    let mut plain = rig(SampleOptions::default());
    open(&mut plain, "media");
    save(&plain, "open-header-unpinned", 80.0);
    plain.open.is_pinned = true;
    save(&plain, "open-header-pinned", 80.0);
    let row = plain.header_layout(plain.frame().bounds);
    let center_y = chrome::header_rect(plain.frame().bounds, plain.scale).center_y();
    let pin = Point::new((row.pin_left + pill_row::PIN_SIZE / 2.0) * plain.scale, center_y);
    let gear = Point::new((row.gear_left + pill_row::GEAR_SIZE / 2.0) * plain.scale, center_y);
    for (is_pinned, state) in [(false, "unpinned"), (true, "pinned")] {
        plain.open.is_pinned = is_pinned;
        for (point, item) in [(pin, "pin"), (gear, "gear")] {
            plain.follow_header_hover(PointerEvent::Moved(point));
            save(&plain, &format!("open-header-{state}-{item}-hovered"), 80.0);
        }
    }
}

#[test]
#[ignore = "writes images to target/snapshots; run on purpose"]
fn snapshots_of_every_open_page_in_indonesian() {
    store::language::with_override(store::language::Language::Indonesian, || {
        let options = SampleOptions { media: Some(true), timer: true, calendar: true, ..SampleOptions::default() };
        let mut notch = rig(options);
        for id in ["media", "shelf", "calendar", "timer", "stats", "todo", "shalat", "weather", "devices"] {
            open(&mut notch, id);
            save(&notch, &format!("id-open-{id}"), WINDOW_HEIGHT);
        }
        let mut week = rig(SampleOptions { calendar_week: true, ..options });
        open(&mut week, "calendar");
        save(&week, "id-open-calendar-week", 260.0);
    });
}
