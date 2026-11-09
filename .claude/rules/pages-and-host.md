---
paths:
  - "crates/notch/src/pages/**"
  - "crates/notch/src/host/**"
  - "crates/notch/src/platform/**"
  - "crates/notch/src/page.rs"
  - "crates/notch/src/activity.rs"
  - "crates/notch/src/activities/**"
  - "crates/notch/src/gfx/**"
  - "crates/notch/src/ui/**"
---

# Pages and host boundaries

Read `docs/how-to/add-a-page.md` before writing a page, `docs/how-to/add-an-activity.md` before writing an activity and
`docs/explanation/architecture.md` before restructuring.

## Pages

- A page lives in `crates/notch/src/pages/<name>/` and is listed only in `crates/notch/src/pages/mod.rs`; adding or removing one touches no
  other file. The pages are media, shelf, calendar, timer, todo, shalat, stats, weather, devices and clipboard.
- A page depends on `page`, `activity` (for `Presence` and `Flash`), `gfx`, `ui`, `platform` and the `places` crate, plus
  `host::Settings` for its own keys; it uses nothing else of `host`.
- `host`, `gfx`, `ui`, `page` and `activity` stay ignorant of any particular page or activity. The one known exception is
  the timer's open width floor (`WIDTH_FLOOR_PAGE_ID` in `host/notch/layout.rs`); do not add another.
- Page settings keys start with the page id (`media.lyrics`).
- A page reports the size of its content only; the host adds its padding and header (`host/chrome.rs`).
- A page that also needs the collapsed pill (a ring, an alert) answers `Page::presence` instead of depending on an
  activity; it keeps its own state. Pointer input on its pill content arrives in `Page::on_presence_pointer`.
- Pages draw through `Canvas` and multiply colors by the `opacity` they are given, so content follows the open and
  close animation.
- Input hooks: `Page::on_pointer` (with `PointerEvent::Scrolled` only where `Page::claims_scroll` says the page uses
  the wheel), `on_key` after `Context::take_keyboard`, `on_system_message` for the messages the page listens to
  (`Context::listen_to_clipboard`, `listen_to_power_setting`), and `on_enabled_changed` when the user switches the
  page on or off (also once at start). A page closes or opens the notch with `Context::request_close` and
  `Context::request_open`; it never touches the window (`Context::window` is crate-private).
- A page that holds user data off by default (`Page::is_opt_in`, the clipboard page) reads nothing while it is off and
  drops what it holds when `on_enabled_changed(false)` arrives.

## Activities

- An activity lives in `crates/notch/src/activities/` and is listed only in `crates/notch/src/activities/mod.rs`. It depends on `activity`, `page`
  (for `Context`), `gfx`, `ui` and `platform`, never on a page or another activity.
- An activity never polls: it listens to a Windows callback, message, event, blocking wait or watcher and wakes the UI
  thread through its `Notifier`. The slow behaviours allowed are one-shot: a `Flash` timer that hides its message, a
  short delay before a read that Windows is not ready for (the Caps Lock read, the Bluetooth battery read after a
  connect), and a worker thread that sleeps to let something settle. `device_battery` is a WinRT `DeviceWatcher`
  that Windows feeds; there is no polling fallback.
- Activity input hooks: `on_pointer` (banners, over their own area), `on_system_message`, `on_notify`,
  `on_settings_changed`. An activity started once keeps its listener thread until exit; switching it off hides its
  presence only.
- Activity settings keys are `activity.<id>` (`activity.device_battery` among them).

## Timer ids

- Host timer ids are below 100 and live as `*_TIMER_ID` constants in `host/notch.rs` (1 to 4, 8, 9 and 10; the clock's
  6 is in `host/notch/timers.rs`); add a new one with the next free number and stop it as soon as its work ends.
- Page timer ids are `100 + page_index * 16 + n` with `n` below 16, up to 999; activity timer ids are
  `1000 + activity_index * 16 + n`. `Context` computes them from the `TimerId(n)` a page or activity passes.

## Host jobs outside the Notch struct

Each is a file or folder of its own in `host/`; keep a new job of this kind out of `notch.rs`.

- `volume_flyout` (with `hook_thread`) hides Windows' volume flyout; `notification_banners` (with `backup` and
  `passes`) turns Windows' banners off and restores them; `screen_mode` (with `location_hook` and the pure `classify`) tracks the foreground
  window; `settings_launcher` starts `notch-settings.exe`. `window/focus.rs` holds the topmost and keyboard-focus calls.
- The rows of the pill and header: `pill_row`, `claimed_row`, and in `host/notch/` `resting_row`, `header_row` and
  `row_motion` (one spring per item).
- `host/settings.rs` reads and writes `settings.txt`; the Windows-popup keys are `notch.hide_windows_volume_popup` and
  `notch.hide_windows_notifications`.

## Colors

Orange as a main action belongs to the timer page only (`FOCUS_ORANGE` for focus; a break uses `BREAK_BLUE`); every other page and the settings window use the blue accent, and green is the battery ring's. A new color needs a rule in `docs/reference/design-rules.md`.
