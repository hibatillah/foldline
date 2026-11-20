# Add a setting

> **How-to.** Read when: you are adding a key or a control in the settings window. Skip if: you only need a key's meaning; see [settings keys](../reference/settings-keys.md).

A setting is one key in `%APPDATA%\Foldline\settings.txt` that the notch reads and the settings window writes; the notch has no controls of its own. The
keys, defaults and the window's sections are listed in [settings keys](../reference/settings-keys.md); this page covers the code.

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
   - A text or number field: use `field::input(key, Kind::Path { width }, placeholder)` or
     `field::input(key, Kind::WholeNumber { default, range }, placeholder)` as the item's control. It saves on Enter or when it loses
     focus, and shows the saved value again when the typed one is rejected.
   - A choice among pictures: a picture card (`card::frame` and `card::grid`, as in `sections/behavior.rs`) whose click
     calls `field::write(key, word, cx)`.
   - A setting that takes effect only when the notch starts again gets `.tag(tr!("advanced.after_restart"))`.
   - A picture that explains a setting goes in its own `Item::picture_row` under the row, drawn with `illustration.rs`.
   - Every item gets a description of one to three short lines that says what the setting does.
   - Write the label and description through `tr!("<section>.<key>")`. For a control on an existing section, add the English and Indonesian text beside the section's other texts, as `general.track_location` does in `crates/settings/locales/app.yml`. For a section of your own, use `crates/settings/locales/pages/<id>.yml` (first line `_version: 2`) with `<id>.title`, `<id>.group_<name>`, `<id>.<setting>` and `<id>.<setting>_note`. A key lives in one locale file only.
   - A new section needs three registrations, and `scripts/new-content.sh settings <id>` makes them ([add your own content](add-your-content.md)):
     - the folder `sections/<id>/mod.rs` with a `page() -> Page` function (`Page::plain(title, groups)`; the page opens with its first section) and `mod <id>;` in `sections/mod.rs`;
     - one `NavEntry::new(NavGroup::PageSettings, icons::<ICON>, <id>::page())` in `sections::all` (`NavGroup::Notch` for a section that is not a page's);
     - one `own_icons!` line in `icons.rs` and its `assets/icons/<stem>.svg`.
   - A new page also gets a `PageRow` in `sections/pages/list.rs` and a `pages.desc_<id>` text, so the Pages section lists it with a description and an icon.
   - Every write goes through `sections::apply`, which saves the key and posts `WM_SETTINGS_CHANGED` to the notch.
4. **Document it** in the same change: the key in the `docs/reference/settings-keys.md` table, the control in its window table, and
   the user-facing default in the `README.md` table.
5. **Check it:** `cargo +stable-x86_64-pc-windows-msvc test --workspace`, then open the window from the gear in the open header or the tray and change
   the control while the notch runs. The notch must pick the change up without a restart.

A tray toggle follows [the same rules](../../.claude/rules/settings-and-tray.md): one key, read at start, re-read on
`WM_SETTINGS_CHANGED`.
