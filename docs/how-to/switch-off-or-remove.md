# Switch pages and activities off, or remove them

> **How-to.** Read when: you want fewer pages or activities, temporarily or for good. Skip if: you are adding one: see [the tutorial](../tutorials/add-a-page.md) or [add an activity](add-an-activity.md).

## Switch pages off

Without any setting, every registered page is on. To use only some of them, list their ids, separated by commas:

- **Environment variable:** `NOTCH_PAGES=shelf,media`
- **Settings file:** a line `pages=shelf,media` in `%APPDATA%\notch\settings.txt`

The environment variable wins when both are set. Page ids are the values returned by each page's `id()`; the built-in
ones are `media`, `shelf`, `calendar`, `timer` and `stats`.

- Pages keep their registration order, whatever order you list them in.
- Ids that match no page are ignored. If none match, every page stays on.

Example, to run without the media page:

```
set NOTCH_PAGES=shelf
notch.exe
```

To change which pages are *compiled in*, edit `crates/notch/src/pages/mod.rs`; see [adding-a-page.md](../tutorials/add-a-page.md).

## Switch an activity off

Volume, brightness, charger and battery, Bluetooth, microphone, camera, copied and Caps Lock are always enabled.
Old `activity.<id>=off` entries for these activities are ignored. Only **Notifications** has a toggle (in the tray menu
and the settings window), on by default. Its choice is remembered as `activity.notifications`; when off at startup, its
listener is not started.

### How an optional activity switches off

Only **Notifications** is optional among the built-in activities. Right-click the tray icon to switch it on or off.
The choice is stored as `activity.notifications=off` or `on` in `%APPDATA%\notch\settings.txt`, defaulting to on.
An optional activity that is off at start is never started; switching it off while running hides it but retains its
listeners until exit. The same happens when another program changes `activity.<id>` in the file and posts
`WM_SETTINGS_CHANGED`: switched on, the activity starts if it never had; switched off, it is hidden. Required activities
ignore saved disable settings and reject toggle requests.

## Remove a page for good

Delete its folder, its `mod` line and its entry in `all()` in `crates/notch/src/pages/mod.rs`. Nothing else refers to it. When no
page has a priority, the collapsed pill shows the time and the open notch only its header.

To only turn a page off without deleting it, list the pages you want in `NOTCH_PAGES` or the `pages` setting; see
[configuration.md](../reference/settings-keys.md).

## Remove an activity for good

Delete its file, its `mod` line and its entry in `all()`.
