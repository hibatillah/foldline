# Test map

> **Reference.** Read when: you need which file tests which behavior. Skip if: you write a test; see [test and snapshot](../how-to/test-and-snapshot.md).

## Where the tests live

Each test sits in the file it tests, in a `#[cfg(test)] mod tests` block at the bottom. There is no separate `tests/`
folder, because the notch is a binary crate and an outside test folder could not reach its internals. The tests are
spread over many files:

| Area | Files with tests |
| --- | --- |
| page contract | `crates/notch/src/page.rs` |
| activity contract | `crates/notch/src/activity.rs` (change detection) |
| host | `crates/notch/src/host/notch/` (`layout.rs` page choice and badge room, `animation.rs` size springs and bounds, `keyboard.rs` text keys, `pages.rs`; `tests/` the hiding rules, screen modes, settings reload and page navigation on an off-screen notch), `navigation.rs` (page order, page layout), `scroll_gate.rs` (wheel cooldown, swipe accumulation), `motion.rs` (spring damping ratio), `activity_layout.rs`, `registry.rs`, `settings.rs`, `tray.rs` (menu commands), `screen_mode.rs` (maximized or full screen), `startup.rs` |
| drawing toolkit | `crates/notch/src/gfx/color.rs`, `geometry.rs`, `spring.rs`, `image.rs`, `path.rs`, `raster.rs`, `canvas.rs` (flare shape) |
| widgets | `crates/notch/src/ui/icon_button.rs`, `icons.rs`, `outline.rs`, `ring.rs`, `hud.rs`, `text_field.rs` (editing keys) |
| shalat page | `crates/notch/src/pages/shalat/times.rs` (the calculation, checked against Aladhan's Kemenag times for Jakarta), `cities.rs` (search order, accents), `config.rs`, `schedule.rs` and `text.rs` (settings, the minute-by-minute waiting rules, wording), `tests.rs` |
| todo page | `crates/notch/src/pages/todo/list.rs` (the file format, order, add/tick/remove), `mod.rs` (rows and height) |
| timer page | `crates/notch/src/pages/timer/session.rs` and `session/` (rules, saving, wake times), `view.rs`, `layout.rs` |
| stats page | `crates/notch/src/pages/stats/format.rs` (numbers), `sampler.rs` (the readings, which disks and GPUs show), `gpu.rs` (GPU load and names), `mod.rs` (layout) |
| shelf page | `crates/notch/src/pages/shelf/items.rs` (the list, wording, saving), `layout.rs` (places, hit targets, name lines, double click), `mod.rs` (arrival) |
| platform | `crates/notch/src/platform.rs` (clipboard text, pixel conversion of Explorer pictures) |
| calendar page | `crates/notch/src/pages/calendar/dates.rs`, `ics/` (reading and repeating), `agenda.rs` (what a day says), `fetch.rs` |
| notifications activity | `crates/notch/src/activities/notifications/items.rs` (the wording), `mod.rs` (banner height, lines) |
| editing pages and settings | `crates/notch/src/host/page_editor.rs`, `settings_panel/` (controls and the value a press saves), `chrome.rs`, `navigation.rs` (the page list) |
| activities | `crates/notch/src/activities/battery.rs` (charge rules), `volume.rs`, `camera.rs`, `privacy.rs`, `caps_lock.rs` |
| media page | `crates/notch/src/pages/media/tests.rs`, `layout.rs`, `resting.rs`, `session.rs`, `lyric_follower.rs`, `lyrics/mod.rs`, `lyrics/client.rs` |

To run one file's tests, or a single test, filter by name:

```
cargo test lyric_follower
cargo test a_paused_track_keeps_its_line_but_arms_no_timer
cargo test -- --nocapture
```
