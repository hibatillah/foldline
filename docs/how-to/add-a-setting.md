# Add a setting

> **How-to.** Read when: you are adding a key, a control in the notch's settings panel, or a control in the settings window. Skip if: you only need a key's meaning; see [settings keys](../reference/settings-keys.md).

A setting is one key in `%APPDATA%\notch\settings.txt` that the notch reads and the settings window can write. The
keys, defaults and the window's sections are listed in [configuration.md](../reference/settings-keys.md); this page covers the code.

1. **Pick the key and the default.** Start the key with `notch.`, `activity.<id>` or the page id. Write booleans as `on`
   or `off`. The same default goes into the notch and the window, and a test does not compare them (the Behavior
   section's words are the exception: its test reads `crates/notch/src/host/visibility.rs`), so check both by hand.
2. **Read it in the notch.** Read the key once at start into a field of `Notch`, a page or an activity, through
   `Context::settings`. Read it again on `WM_SETTINGS_CHANGED`: `Notch::reload_settings` and `on_settings_changed` run
   then, and the re-read falls back to the field's current value, not the default. A missing or unreadable value uses
   the documented default.
3. **Add the control to the window** (`crates/settings/src/`):
   - A toggle: add `Item::new(label, field::toggle(key, default)).description(text)` to a `Group` in the section's
     `page()`.
   - A text or number field: use `field::input(key, Kind::Text { width }, placeholder)` or
     `field::input(key, Kind::WholeNumber { default, range }, placeholder)` as the item's control. It saves on Enter or when it loses
     focus, and shows the saved value again when the typed one is rejected.
   - A choice from a list: `field::dropdown(key, choices, shown)`.
   - Every item gets a description of one to three short lines that says what the setting does.
   - A new section: create `sections/<name>.rs` with a `page() -> Page` function (title, one-line description, groups)
     and add it to `sections::all` in `sections/mod.rs`.
   - Every write goes through `sections::apply`, which saves the key and posts `WM_SETTINGS_CHANGED` to the notch.
4. **Document it** in the same change: the key in the `docs/reference/settings-keys.md` table, the control in its window table, and
   the user-facing default in the `README.md` table.
5. **Check it:** `cargo +stable-x86_64-pc-windows-msvc test --workspace`, then open the window from the tray and change
   the control while the notch runs. The notch must pick the change up without a restart.

To show a page's setting in the notch's own panel, return it from the page's `Page::settings()` as a `Control`:
`Switch` (on/off), `Stepper` (a whole number, negative allowed, with - and +), `Choice` (one word out of a few), `Text`
(typed in the notch, which takes the keyboard while it is being edited), `Search` (typed text whose matches come from
`Page::search`; picking a match saves every key in its `SearchHit`) or `Picks` (up to a limit out of a list, saved as a
comma list). A page with more settings than one tab holds (220 px) returns several `SettingsGroup`s from
`settings_groups()`, each its own tab. The
panel shows them under the page's name, saves each through the same settings file and re-reads it, so the page's
`on_settings_changed` runs as usual. Global switches and choices are listed in `crates/notch/src/host/settings_panel/controls.rs`.

A tray or menu toggle follows [the same rules](../../.claude/rules/settings-and-tray.md): one key, read at start, re-read on
`WM_SETTINGS_CHANGED`.
