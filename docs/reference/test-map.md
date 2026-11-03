# Test map

> **Reference.** Read when: you need which file tests which behavior. Skip if: you write a test; see [test and snapshot](../how-to/test-and-snapshot.md).

## Where the tests live

Each test sits in the file it tests, in a `#[cfg(test)] mod tests` block at the bottom. There is no separate `tests/`
folder, because the notch is a binary crate and an outside test folder could not reach its internals. The tests are
spread over many files:

| Area | Files with tests |
| --- | --- |
| page contract | `src/page.rs` |
| activity contract | `src/activity.rs` (change detection) |
| host | `src/host/notch.rs` (page choice, badge room), `navigation.rs` (page steps, wheel, swipe limit, dots), `activity_layout.rs`, `registry.rs`, `settings.rs`, `tray.rs` (menu commands), `screen_mode.rs` (maximized or full screen), `startup.rs`, `notch/snapshots.rs` (the hiding rules) |
| drawing toolkit | `src/gfx/color.rs`, `geometry.rs`, `spring.rs`, `image.rs`, `path.rs`, `raster.rs`, `canvas.rs` (flare shape) |
| widgets | `src/ui/icon_button.rs`, `icons.rs`, `outline.rs`, `ring.rs`, `hud.rs`, `text_field.rs` (editing keys) |
| shalat page | `src/pages/shalat/times.rs` (the calculation, checked against Aladhan's Kemenag times for Jakarta), `cities.rs` (search order, accents), `mod.rs` (settings, the minute-by-minute waiting rules) |
| todo page | `src/pages/todo/list.rs` (the file format, order, add/tick/remove), `mod.rs` (rows and height) |
| timer page | `src/pages/timer/session.rs` (rules, saving), `view.rs`, `layout.rs` |
| stats page | `src/pages/stats/format.rs` (numbers), `sampler.rs` (the readings, which disks and GPUs show), `gpu.rs` (GPU load and names), `mod.rs` (layout) |
| shelf page | `src/pages/shelf/items.rs` (the list, wording, saving), `mod.rs` (places, hit targets, name lines, double click, arrival) |
| platform | `src/platform.rs` (clipboard text, pixel conversion of Explorer pictures) |
| calendar page | `src/pages/calendar/dates.rs`, `ics.rs` (reading and repeating), `agenda.rs` (what a day says), `fetch.rs` |
| notifications activity | `src/activities/notifications/items.rs` (the wording), `mod.rs` (banner height, lines) |
| editing pages and settings | `src/host/page_editor.rs`, `settings_panel.rs` (controls and the value a press saves), `chrome.rs`, `navigation.rs` (the page list) |
| activities | `src/activities/battery.rs` (charge rules), `volume.rs`, `camera.rs`, `privacy.rs`, `caps_lock.rs` |
| media page | `src/pages/media/mod.rs`, `layout.rs`, `resting.rs`, `session.rs`, `lyric_follower.rs`, `lyrics/mod.rs`, `lyrics/client.rs` |

To run one file's tests, or a single test, filter by name:

```
cargo test lyric_follower
cargo test a_paused_track_keeps_its_line_but_arms_no_timer
cargo test -- --nocapture
```
