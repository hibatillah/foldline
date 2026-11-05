---
paths:
  - "crates/notch/src/host/settings.rs"
  - "crates/notch/src/host/tray.rs"
  - "crates/notch/src/host/window.rs"
  - "crates/settings/src/**"
---

# Settings and tray

- Every in-notch panel toggle and every settings-window control is backed by one settings key (`notch.*`, `activity.<id>`, `advanced.*`, page
  keys start with the page id), written through `host/settings.rs` or `sections::apply`, read into a field of `Notch` at
  start and re-read on `WM_SETTINGS_CHANGED` (`Notch::reload_settings`, then `on_settings_changed` on every page and
  started activity). A re-read falls back to the field's current value, not the default.
- A new key, its default and its label go into `docs/reference/settings-keys.md` and the `README.md` menu list and defaults
  table in the same change. The steps are in `docs/how-to/add-a-setting.md`.
- The tray menu has only Start with Windows, Settings and Quit Notch; every other toggle lives in the in-notch panel or the settings window.
- Write boolean keys as `on` or `off`, and at start treat a missing or unreadable value as the documented default.
