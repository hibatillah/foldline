---
paths:
  - "crates/notch/src/host/settings.rs"
  - "crates/notch/src/host/tray.rs"
  - "crates/notch/src/host/window.rs"
  - "crates/notch/src/host/settings_launcher.rs"
  - "crates/notch/src/host/notch/header_row.rs"
  - "crates/settings/src/**"
---

# Settings and tray

- Every settings-window control is backed by one settings key (`notch.*`, `activity.<id>`, `advanced.*`, page
  keys start with the page id), written through `host/settings.rs` or `sections::apply`, read into a field of `Notch` at
  start and re-read on `WM_SETTINGS_CHANGED` (`Notch::reload_settings`, then `on_settings_changed` on every page and
  started activity). A re-read falls back to the field's current value, not the default.
- A new key, its default and its label go into `docs/reference/settings-keys.md` and the `README.md` menu list and defaults
  table in the same change. The steps are in `docs/how-to/add-a-setting.md`.
- The tray menu has Snooze notifications (a submenu: 10 minutes, 30 minutes, 1 hour, until turned off; Turn off snooze while snoozed), Pause Foldline (Resume Foldline while paused), Settings, then Quit Foldline; every toggle, Start with Windows included, lives in the settings window.
- Write boolean keys as `on` or `off`, and at start treat a missing or unreadable value as the documented default.
- The notch starts the settings window from the gear in the open header (`host/notch/header_row.rs`) and the tray's **Settings** item, both through `host/settings_launcher.rs`. A second start brings the open window forward (`crates/settings/src/single_instance.rs`).
- The window's pages (`crates/settings/src/sections/mod.rs`): General, Pages, Behavior and Advanced with no group label, then under "Page settings" Calendar, Timer, Shalat ("Shalat" in Indonesian), System stats and Weather, with About pinned at the bottom of the sidebar. A new key goes on the page that matches it.
- Sizes: `UI_FONT_SIZE` in `main.rs` is the one scale knob (gpui-component sets the rem size from it, so text, `px_*`, `gap_*` and controls follow); change a size through it, not by editing many constants. Keep the compact sizes the constants in `main.rs`, `layout.rs`, `sidebar.rs` and `header.rs` hold, and ask before changing one.
- Frame: the window background is blurred (`WindowBackgroundAppearance::Blurred`); `palette.rs` sets `theme.background` to transparent so gpui-component's Root does not paint over it, and the translucent frame color (dark near-black, light white, both partly clear) carries the sidebar. Only the content pane is solid and flush to the top, right and bottom edges (shadcn "sidebar" variant, no radius); inside it, blocks are `#2a2a2d` on a `#1b1b1d` card in the dark theme, and every color is a `palette.rs` token. `FOLDLINE_SETTINGS_SOLID=1` gives an opaque neutral frame. Do not tint the frame blue.
- Pages follow the v7 boards in `.superpowers/design/settings-v7-full/` (`<page>-full.png`). A redesigned page uses
  `Page::plain` and opens with its first section; pictures that explain a setting are GPUI drawings in an
  `Item::picture_row` (`illustration.rs`), never images; an "After restart" tag (`Item::tag`) goes only on a key the
  notch reads at start (`advanced.debug_log`). General holds Startup, Location, Notifications, Show the
  notch on (picture cards; the chosen screen is picked on a drawing of the connected screens) and Windows popups;
  Advanced holds Troubleshooting; Timer has its own page. A row that sets a length in
  minutes uses `stepper.rs` (− value +) and `chips.rs` (presets), never a typed number.
- Dragging: the header strip, the brand strip, the group labels and the sidebar's gutters are `WindowControlArea::Drag` areas; a sidebar item is never inside one, so its click stays a click.
- One blue accent (dark `#4c9aff`, light `#2563eb`) marks icons, the active breadcrumb, switches, links, focus rings and the selected card. The palette tests hold text contrast on the card and on the frame over a bright and a dark desktop: keep them passing when a color or an opacity changes.
- The brand tile (`brand.rs`) follows the build's channel, blue for stable and indigo for a pre-release (see `branding-and-build.md`).
