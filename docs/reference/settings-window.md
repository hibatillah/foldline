# Settings window

> **Reference.** Read when: you need what each section of `notch-settings.exe` sets, or how it saves. Skip if: you are looking for a key's default; it is in [settings keys](settings-keys.md).

Settings are edited only in this window; the notch has no settings panel. Open it from the gear at the right of the open
header or from the tray menu's **Settings** item. Both start `notch-settings.exe` from the folder of the running
`notch.exe` without waiting for it (`crates/notch/src/host/settings_launcher.rs`) and log `could not start settings` when the program
is missing. The window allows one instance: starting it again focuses the open one.

## Layout

The frame is a neutral color (dark `#101010` at 64 %, light white at 35 %) over a blurred acrylic backdrop
(`palette::window_background`), so the desktop shows through; it carries the sidebar directly, with no card and no solid
fill, and has no tint or glow. The content is the one solid surface, a rounded (8 px), bordered card inset 8 px from the
top, right and bottom edges (`FRAME_PADDING`, `CARD_RADIUS` in `crates/settings/src/main.rs`). The sidebar is 200 px wide. The window follows the Windows display scale; `UI_FONT_SIZE` (14.7, `main.rs`, applied as `theme.font_size` in `palette::apply`) is the one knob that shrinks every rem-based size. The colors are in `crates/settings/src/palette.rs`, with one
blue accent; the font is Geist (`theme.font_family`, set in `palette::apply`), and the reasons are in
[design-rules.md](design-rules.md#settings-window). Where the blur is unwanted, start the window with
`NOTCH_SETTINGS_SOLID=1`: the frame is then a neutral opaque color (`#141414` dark, `#eeeeee` light). gpui 0.2.2 offers only `Opaque`, `Transparent` and
`Blurred` on Windows (no Mica), and cannot report whether blur works, so the frame keeps a tint that keeps the text readable
over most desktops.

| Part | What it holds | Source |
| --- | --- | --- |
| Brand strip | 58 px high (`BRAND_HEIGHT`, with 6 px top padding), the squared tile logo (28 px: blue `tile-stable.png` on a stable build, orange `tile-dev.png` on a pre-release, chosen by `release_version`), "Notch" (`text_base` semibold) and the version, muted; a drag area | `main.rs`, `brand.rs`, `sidebar.rs` |
| Sidebar | group "Notch": General, Pages, Behavior, Advanced; group "Page settings": Calendar, Shalat, System stats, Weather; **About** at the bottom (`NavGroup::Footer`, no label). Items are 32 px high with a blue 14 px icon and `text_sm` text; the empty sidebar (gutters, space below the items, group labels) is a drag area, the items are not; the selected one has a rounded, faint fill | `sidebar.rs`, `sections/mod.rs` (`all`) |
| Sidebar icons | SVG files in `crates/settings/assets/icons`, served by `WindowAssets`, all drawn in the accent blue: a hex nut for General (the notch's gear), Lucide layers (Pages), puzzle (Behavior), calendar-days (Calendar), moon-star (Shalat), chart-column (System stats), cloud-drizzle (Weather), wrench (Advanced) and info (About); folder is the icon of the Files section | `icons.rs` |
| Card header | 36 px, a little lighter than the card, drawn by the window and not by the component library's title bar: the breadcrumb "Settings › <Page>" (the page in blue), then minimize, maximize and close at the right end; a drag area. Its top corners are rounded to the card's, and the close button rounds its own, so the red hover follows the corner | `header.rs` |
| Card content | the page's description (`text_base`, full white, one sentence), then the sections: each is one bordered block (8 px radius, one step lighter than the card) of rows with dividers between them, its title (`text_base`, with a small blue icon where one fits: Files, Google Calendar) above the block; a row has its title (medium) and description at the left and the control at the right; buttons are dark bordered rounded pills, and an inert one is dimmed with "Coming soon" to its left; picture cards for option pickers | `layout.rs`, `field.rs`, `card.rs`, `picture.rs`, `sections/about/widgets.rs` |

## Sections

| Section | What you set | Key |
| --- | --- | --- |
| General | Startup: start the notch at sign-in | the `Run` registry value, see [Start with Windows](settings-keys.md#start-with-windows) |
| General | Startup: hide the notch unless music plays or a message or banner shows | `notch.hide_when_idle` |
| General | Location: the city, searched by name; up to five matches are listed and the pick is saved | `location.city`, `location.lat`, `location.lon`, `location.country` |
| General | Notifications: show Windows notification banners (its description points to the switch that hides Windows' own banner); show what a notification says; warn when a connected device's battery is low; show browser downloads (**Browser downloads**, on by default; its description says only file names in the Downloads folder are read) | `activity.notifications`, `notifications.show_text`, `activity.device_battery`, `activity.downloads` |
| General | Windows popups: hide the Windows volume and brightness popups; hide Windows notification banners (both on by default) | `notch.hide_windows_volume_popup`, `notch.hide_windows_notifications` |
| General | Pause: a **Pause Notch** group with four buttons while the notch runs (10 min, 30 min, 1 hour, Until I resume); while it is paused the buttons give way to "Paused until 14:30" (24-hour local time) or "Paused until you resume" and a **Resume** button. A pause hides the notch completely and gives Windows' volume flyout and banners back, see [pausing the notch](settings-keys.md#pausing-the-notch) | `notch.paused_until` |
| General | Screen: **Show the notch on**, a dropdown with Main screen and Focused screen (default Main screen; a missing or unknown word shows Main screen). Applies live, and a note says the distance from the top is measured from the top of that screen | `notch.monitor` |
| General | Timer: focus, short break and long break lengths | `timer.focus_minutes`, `timer.short_break_minutes`, `timer.long_break_minutes` |
| Pages | the order of the pages, and which are on; drag a page to reorder it. Each known page shows a one-line description under its name (media, shelf, calendar, timer, stats, clipboard); the opt-in Clipboard page starts switched off and has no section of its own because it has no settings | `navigation.order`, `navigation.disabled` |
| Behavior | an intro sentence and a read-only matrix of what each level shows (rows 0-3, columns On-demand messages, Lyrics, Idle pill; blue checks), then what the notch shows over a maximized window and over a full-screen app as four level cards in two columns (0 · Hidden, 1 · Activities, 2 · Lyrics, 3 · Always visible; each level also shows everything below it), and whether pointing at it opens it there (**Open by pointing**; a pinned notch stays open regardless) | `notch.maximized`, `notch.fullscreen`, `notch.hover_maximized`, `notch.hover_fullscreen` |
| Calendar | the calendar address and how often it is downloaded again; the section lists the steps for finding a Google Calendar address | `calendar.ics_url`, `calendar.refresh_minutes` |
| System stats | Disks (one checkbox per known disk, at most 3) and GPUs (one per known GPU, at most 2); unset shows the first 3 disks and first 2 GPUs, and the first click writes the explicit list in the notch's order (GPUs joined with `|`). Until the notch has published a list the block says to open the System stats page in the notch once; an empty GPU list says "No GPU found" | `stats.disks`, `stats.gpus`, read from `stats.known_disks`, `stats.known_gpus` |
| Shalat | a note that the location is set in General (the page has no city field); the calculation method as picture cards, Auto and six methods, each showing how far below the horizon the sun is at Fajr and at Isha; Standard or Hanafi Asr, drawn as a stick with its shadow; the two reminders with a preview of each; minus 30 to plus 30 minutes for each prayer, with a day bar that shows where each prayer falls | `shalat.method`, `shalat.hanafi`, `shalat.message`, `shalat.badge`, `shalat.offset_fajr`, `_sunrise`, `_dhuhr`, `_asr`, `_maghrib`, `_isha` |
| Advanced | the debug log file; how far the notch moves down from the top edge of its screen. Both apply after a restart of the notch | `advanced.debug_log`, `notch.top` |
| About | the app card (the build's tile, "Notch", "Version x", and a **Check now** button); What's new, Licenses and Lyrics rows; under "Files", the settings file and the data folder. See [About](#about) | none; it writes nothing |

How the window saves:

- A toggle or choice writes when you click it; picking a city writes its four `location.*` keys and posts one reload message. Dragging a page, and showing or hiding one, writes at once.
- A text or number field writes on Enter or when it loses focus.
- A number outside its range (focus 5 to 120, breaks 1 to 120, calendar refresh 1 to 1440, move down 0 to 2000) is dropped
  and the field shows the saved value again.
- The calendar address shows as dots, with a button that reveals it. An empty field shows a placeholder with the
  default or an example.
- Every write posts `WM_SETTINGS_CHANGED` to the notch (see [Settings file](settings-keys.md#settings-file)), which applies the change
  without a restart.
- The Pages section writes nothing until the notch has published `pages.known`; start the notch once, then reopen the
  window. It keeps one page on.

### Size

The window opens centered at 1040 x 660 and resizes down to 720 x 480. When the primary display is smaller, the opening size
is the default cut to the display minus an 80 px margin on each axis, never below 720 x 480 (`opening_size`, `SCREEN_MARGIN`
in `main.rs`). A display under 800 x 560 therefore gets a window larger than the display less the margin.

### About

The page is `crates/settings/src/sections/about/`. It is the one item in the sidebar's footer.

- **App card:** the tile of the build's channel (the dev tile for a pre-release version, the stable tile otherwise, chosen by
  `release_version::icon_for_version` like the executables' icons), "Notch" and "Version x". **Check now** is inert and shows
  "Coming soon", because the update check is not built; `update_check()` in `about/mod.rs` returns the handler when it is.
- **What's new:** inert until the constant `WHATS_NEW_URL` in `about/mod.rs` is set to the release notes address.
- **Licenses:** opens `THIRD-PARTY-LICENSES.txt`, looked for in the folder of `notch-settings.exe` and then in up to six
  parent folders (so `run/` and `target/.../release` find the repository-root file; the installer puts it beside the
  program, and `scripts/reload.sh` copies it into `run/`). When it is not found, a warning notification in the window's top
  right corner says so (`about/licenses.rs`).
- **Lyrics:** a link to LRCLIB (`https://lrclib.net`), where the synced lyrics come from.
- **Files:** **Settings file** opens `%APPDATA%\notch\settings.txt` (the folder when the file does not exist yet);
  **Data folder** opens `%APPDATA%\notch`, which holds `todo.txt`, `shelf.txt` and `calendar.ics`. When the folder is missing,
  a warning says the notch has not saved anything yet (`about/paths.rs`).

The window's memory and CPU figures are in [performance baseline](performance-baseline.md#settings-window). To add a setting or a
section, see [add a setting](../how-to/add-a-setting.md).

The window's layers, row layout and option cards follow the "Settings window" rules in [design-rules.md](design-rules.md#settings-window).
