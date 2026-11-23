# Settings window

> **Reference.** Read when: you need what each section of `foldline-settings.exe` sets, or how it saves. Skip if: you are looking for a key's default; it is in [settings keys](settings-keys.md).

Settings are edited only in this window; the notch has no settings panel. Open it from the gear at the right of the open
header or from the tray menu's **Settings** item. Both start `foldline-settings.exe` from the folder of the running
`foldline.exe` without waiting for it (`crates/notch/src/host/settings_launcher.rs`) and log `could not start settings` when the program
is missing. The window allows one instance: starting it again focuses the open one.

## Layout

The window follows shadcn's "sidebar" variant, not "inset". The sidebar is a neutral color (dark `#101010` at 64 %, light white at
35 %) over a blurred acrylic backdrop (`palette::window_background`), so the desktop shows through it; it has no card and
no tint or glow. The content pane is one solid surface that fills the window to the top, right and bottom edges, with no
padding, margin or rounded corner around it and a one-pixel hairline (the card border color) between it and the
sidebar (`SettingsWindow::render` in `crates/settings/src/main.rs`). The sidebar is 216 px wide. The window follows the Windows display scale; `UI_FONT_SIZE` (14.5, `main.rs`, applied as `theme.font_size` in `palette::apply`) is the one knob that shrinks every rem-based size. The colors are in `crates/settings/src/palette.rs`, with one
blue accent; the font is Geist (`theme.font_family`, set in `palette::apply`), and the reasons are in
[design-rules.md](design-rules.md#settings-window). Where the blur is unwanted, start the window with
`FOLDLINE_SETTINGS_SOLID=1`: the sidebar is then a neutral opaque color (`#141414` dark, `#eeeeee` light). gpui 0.2.2 offers only `Opaque`, `Transparent` and
`Blurred` on Windows (no Mica), and cannot report whether blur works, so the frame keeps a tint that keeps the text readable
over most desktops.

| Part | What it holds | Source |
| --- | --- | --- |
| Brand strip | the sidebar's 16 px top padding, then the squared tile logo (28 px: blue `tile-stable.png` on a stable build, indigo `tile-dev.png` on a pre-release, chosen by `release_version`), "Foldline" (`text_lg` semibold) and the version (`text_xs`, muted), 8 px apart and 18 px from the left edge, with 36 px below, the gap to the first item; a drag area | `main.rs`, `brand.rs`, `sidebar.rs` |
| Sidebar | General, Behavior, Pages, Advanced with no group label (`NavGroup::Notch.label()` is `None`); group "Page settings": Calendar, Timer, Shalat, System stats, Weather; **About** at the bottom (`NavGroup::Footer`, no label). Padding 16 px above, 10 px at the sides, 12 px below. Group labels are `text_sm`, muted, with 12 px above and 5 px below. Items are 36 px high, 1 px apart, with an 8 px radius, a blue 17 px icon 9 px from the label (14 px text, between `text_sm` and the base size; 10 px of padding), all in the foreground color (the primary text color of the theme); the empty sidebar (gutters, space below the items, group labels) is a drag area, the items are not; the selected one differs only by its faint fill, the label color and weight stay the same. Each section is one `NavEntry` in `sections::all`; a page's settings of your own is one more entry in the Page settings group ([add your own content](../how-to/add-your-content.md)) | `sidebar.rs`, `sections/mod.rs` (`all`) |
| Sidebar icons | SVG files in `crates/settings/assets/icons`, served by `WindowAssets`, all drawn in the accent blue: a hex nut for General (the notch's gear), Lucide layers (Pages), puzzle (Behavior), calendar-days (Calendar), timer (Timer), moon-star (Shalat), chart-column (System stats), cloud-drizzle (Weather), wrench (Advanced) and info (About). One `own_icons!` line in `icons.rs` declares each file, and a test fails when a file in the folder is not declared once | `icons.rs` |
| Pane header | 44 px, a little lighter than the pane, drawn by the window and not by the component library's title bar: the breadcrumb "Settings › <Page>" (`text_base`, 22 px from the pane's left edge, 10 px around the 13 px chevron, the page in blue and medium weight), then minimize, maximize and close at the right end; a drag area. Square corners: the pane is flush with the window, so the close button's red hover fills the window's corner | `header.rs` |
| Pane content | padded 16 px above, 22 px at the sides and 20 px below. General, Behavior, Pages, Advanced and About open straight with their first section (`Page::plain`); the page settings (Calendar, Timer, Shalat, System stats, Weather) open the same way; a section may carry a muted note under its title (`Group::description`). Sections sit 14 px apart. A section opens with a heading row (`text_sm`, muted: its title at the left, an optional note at the right; a section with no title, such as About's two blocks, has no heading row), then its parts 7 px apart: blocks (12 px radius, one step lighter than the card, `#2a2a2d` dark) of rows with dividers between them, and items drawn outside a block (`Item::outside_block`, the Behavior cards, the hint boxes). A row is at least 50 px high with 9 px by 14 px padding: its title (`text_base`, medium, with an optional tag such as "After restart" after it: `Item::tag`, `tag.rs`) and description (`text_sm`, 17 px lines) at the left, 12 px from the control at the right. A picture row (`Item::picture_row`) holds pictures or a wide control instead, padded 12 px by 14 px. Pictures of the desktop are drawn with GPUI elements, never images (`illustration.rs`: a dark blue scene, a window, the notch, a strip of equal pictures with captions). A hint box (`info_box.rs`) is a blue-bordered box with a blue icon and a sentence. Switches are 42 x 25 (`switch.rs`); buttons are rounded (8 px, 30 px high) a step lighter than the block with a thin border, every button does something (none is dimmed or marked "Coming soon"); a select (a dropdown button, `sections/countdown.rs`) is a 190 px outline button with a chevron that opens a popup menu; picture cards for option pickers (10 px radius, 1.5 px border, 8 px padding, 6 px between parts, 10 px between cards; title 13.5 px semibold, description 12 px on 16 px lines) | `layout.rs`, `field.rs`, `card.rs`, `picture.rs`, `illustration.rs`, `info_box.rs`, `tag.rs`, `sections/about/widgets.rs` |

## Sections

| Section | What you set | Key |
| --- | --- | --- |
| General | Snooze notifications, under Startup and above Pause, with the same control as Pause: a select that reads "Choose how long" and lists 10 min, 30 min, 1 hour and Until I turn it off (picking one starts the snooze); while snoozed the select gives way to "Snoozed until 14:30" or "Snoozed until you turn it off" and a **Turn off** button. It hides banner-style messages and page alerts without hiding the notch | `notch.snoozed_until` |
| General | Pause, under Snooze notifications: a **Pause Foldline** row with a select that reads "Choose how long" and lists 10 min, 30 min, 1 hour and Until I resume (picking one pauses the notch); while it is paused the select gives way to "Paused until 14:30" (24-hour local time) or "Paused until you resume" and a **Resume** button. A pause hides the notch completely and gives Windows' volume flyout and banners back, see [pausing the notch](settings-keys.md#pausing-the-notch) | `notch.paused_until` |
| General | Language, second from the bottom, above Windows popups: the description ("For the notch and this window", then what Automatic follows now, such as "Follows Windows, which is in English.") sits inside the section's block above three equal compact cards (a radio and the bold name on one row: Automatic, English, Bahasa Indonesia) | `general.language` |
| General | Startup, the topmost section: start the notch at sign-in | the `Run` registry value, or the `FoldlineStartup` task in an MSIX build; see [Start with Windows](settings-keys.md#start-with-windows) |
| General | Startup: hide the notch unless music plays or a message or banner shows, with three pictures under it (the notch in the tray, the pill hidden while idle, the pill back when the pointer reaches the top edge) | `notch.hide_when_idle` |
| General | Location: a **City** row, then the search field (up to five matches, the saved city checked; "Shalat times" and "Weather" chips say who uses it) beside a drawn map (street grid, river, a pin, the city's name and its coordinates; a picture, not a real map), then **Track location** | `location.city`, `location.lat`, `location.lon`, `location.country`, `location.track` |
| General | Notifications: show Windows notification banners (its description points to the switch that hides Windows' own banner); show what a notification says, with two banner cards under it (Show the message, Hide the message: "New message") that set the same key; warn when a connected device's battery is low; show browser downloads (**Browser downloads**, on by default; its description says only file names in the Downloads folder are read) | `activity.notifications`, `notifications.show_text`, `activity.device_battery`, `activity.downloads` |
| General | **Show the notch on**: three picture cards, Main screen, Focused screen and A chosen screen, each drawing three screens with the notch on the one that choice picks (default Main screen; a missing or unknown word shows Main screen). While A chosen screen is selected a **Chosen screen** row draws the connected screens where they stand on the desktop, scaled to fit (`store::screens::list()`, read when the page paints, no polling): each shows its number, its size and "Main" on the primary, the chosen one is outlined in blue with the notch on it, pointing at one shows "Screen N, model, 1920x1080 (main)", and a click stores its id. A stored id missing from the list says the notch uses the main screen until it returns; painting never overwrites the stored id. The heading says it applies at once | `notch.monitor`, `notch.monitor_id` |
| General | Windows popups: hide the Windows volume and brightness popups; hide Windows notification banners (both on by default) | `notch.hide_windows_volume_popup`, `notch.hide_windows_notifications` |
| Timer | focus, short break and long break lengths, each a minus and plus stepper, with a picture of the round and its total | `timer.focus_minutes`, `timer.short_break_minutes`, `timer.long_break_minutes` |
| Pages | The first thing on the page is "How it looks in the notch": the open header at the first, the third (or middle) and the last page shown, one dot per page shown and the long dot on the current one, captioned with its key ("First page (key 1): Media", "Last page (key 0): ..."). Second comes a blue hint box saying that a page's number is its shortcut key (while the pointer is on the open notch, press 1 to 9, and 0 for the last page; pages that are off are skipped and get no number). Then the list, headed "N pages · M on · drag to reorder": one row per page with its key (1 to 9 in a blue square for the pages shown, a dashed "–" for a page that is off; the last page shown carries an "Also key 0" tag, or 0 when it comes after the ninth), a drag handle, the page's icon, its name and a one-line description, and its switch; a page that is off is dimmed. Drag a row to reorder it. One more hint box follows the list: a page switched off keeps its place. The page ends with the section "Want to add your own page?": a row saying Foldline is open source (check the repository, clone it, or open a pull request to add more pages) with a **github.com/hibatillah/foldline** button, a GitHub icon before the address, that opens `links::REPOSITORY_URL`. A page's icon and description come from its `PageRow` in `sections/pages/list.rs`; a page without a row is listed with no description and the Pages icon. The digit keys are handled in `host/notch/page_hotkeys.rs`. The opt-in Clipboard page starts switched off and has no section of its own. | `navigation.order`, `navigation.disabled` |
| Behavior | one section each for a maximized window and a full-screen app (the heading row says when it applies): four picture cards in one row, each a 78 px picture above the level and its sentence (Hidden, Activities only, Lyrics, Always visible; each level also shows everything below it), the selected one with a blue check after its title, then **Open on hover** (a pinned notch stays open regardless) | `notch.maximized`, `notch.fullscreen`, `notch.hover_maximized`, `notch.hover_fullscreen` |
| Calendar | "Calendar view" first (Month or Week cards); then the Google Calendar section with the calendar address (with a **Reset** button) and how often it is downloaded again; last a collapsible help, closed until clicked, that lists the five steps for finding a Google Calendar address and the two notes about keeping it private (`sections/calendar/steps.rs`) | `calendar.ics_url`, `calendar.refresh_minutes` |
| System stats | Disks (one checkbox per known disk, at most 3) and GPUs (one per known GPU, at most 2); each row is a checkbox, the bold name, the muted description on the same baseline (for a disk "Local disk · 512 GB"), the free width, a thin usage bar and the percent at the right; the section heading shows "Pick up to 3 · 2 of 3" at the right; unset shows the first 3 disks and first 2 GPUs, and the first click writes the explicit list in the notch's order (GPUs joined with `|`). Until the notch has published a list the block says to open the System stats page in the notch once; an empty GPU list says "No GPU found" | `stats.disks`, `stats.gpus`, read from `stats.known_disks`, `stats.known_gpus` |
| Shalat | a section note that the location is set in General (the page has no city field); each section opens with a bold heading line; the calculation method as picture cards, Auto and six methods, each showing how far below the horizon the sun is at Fajr and at Isha; Standard or Hanafi Asr, drawn as a stick with its shadow; the two reminders with a preview of each; **Adjust the times** is one rounded card: a day bar that shows where each prayer falls, then one row per prayer with its name at the left and a minus and plus stepper (minus 30 to plus 30 minutes) at the right, thin dividers between the rows | `shalat.method`, `shalat.hanafi`, `shalat.message`, `shalat.badge`, `shalat.offset_fajr`, `_sunrise`, `_dhuhr`, `_asr`, `_maghrib`, `_isha` |
| Weather | the **Rain alert** switch: a banner and then a countdown badge from 30 minutes before the next rain (`sections/weather/`) | `weather.rain_alert` |
| Advanced | Troubleshooting: the debug log file (a path field with a file icon and **Browse**, which opens Windows' save dialog in the log's folder, else `%APPDATA%\Foldline`), with three sample lines in the log's own format under it. It carries an "After restart" tag. | `advanced.debug_log` |
| About | the app card in its own block (the build's tile, "Foldline", "Version x", and a **Check for updates** button); What's new, Source code and Licenses rows; then, in a block with no title, the settings file and the data folder. See [About](#about) | none; it writes nothing |

How the window saves:

- A toggle or choice writes when you click it; picking a city writes its four `location.*` keys and posts one reload message. Dragging a page, and showing or hiding one, writes at once.
- A text or number field writes on Enter or when it loses focus.
- A number outside its range (focus 5 to 120, breaks 1 to 120, calendar refresh 1 to 1440) is dropped
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

The page is `crates/settings/src/sections/about/`. It sits at the bottom of the sidebar, in the footer.

- **App card:** the tile of the build's channel (the dev tile for a pre-release version, the stable tile otherwise, chosen by
  `release_version::icon_for_version` like the executables' icons), "Foldline" and "Version x", and a **Check for updates** button. The check runs only on a click (no timer, nothing at start-up): `updates/` (`mod.rs` holds the state and runs one check on a background thread, `fetch.rs` sends a GET to `links::LATEST_RELEASE_API_URL` with a `Foldline/<version> (+https://github.com/hibatillah/foldline)` User-Agent and an Accept header and reads `tag_name`, `version_order.rs` compares the tag with the running version). `about/update_card.rs` maps each state to what the card shows:

  | State | Note beside the button | Button |
  | --- | --- | --- |
  | before a check | none | **Check for updates** |
  | checking | "Checking..." | **Check for updates**; a click does nothing |
  | up to date | "Foldline is up to date" | **Check for updates** |
  | newer release | "Version X is available" | **Download**, which opens `links::LATEST_RELEASE_URL` (a fixed address, never one from the response) |
  | any failure (no network, status 403, 404, 429 or other, no tag in the body) | "Could not check for updates" | **Check for updates** |

  A pre-release version (`0.6.0-alpha.2`) counts the stable release with the same numbers as newer; a tag with a suffix is never an update. The app downloads and installs nothing. The request is the settings window's only network use; the notch makes none.
- **What's new:** opens `links::RELEASES_URL` (https://fold.hibatillah.com/releases) in the browser.
- **Source code:** a **GitHub** button with the GitHub icon (the component library's `IconName::GitHub`) that opens `links::REPOSITORY_URL` (https://github.com/hibatillah/foldline). The Pages page links to the same constant. The addresses live in `crates/settings/src/links.rs`.
- **Licenses:** opens `THIRD-PARTY-LICENSES.txt`, looked for in the folder of `foldline-settings.exe` and then in up to six
  parent folders (so `run/` and `target/.../release` find the repository-root file; the installer puts it beside the
  program, and `scripts/reload.sh` copies it into `run/`). When it is not found, a warning notification in the window's top
  right corner says so (`about/licenses.rs`).
- **Settings file and data folder** (a block with no title): **Settings file** opens `%APPDATA%\Foldline\settings.txt` (the folder when the file does not exist yet);
  **Data folder** opens `%APPDATA%\Foldline`, which holds `todo.txt`, `shelf.txt` and `calendar.ics`. When the folder is missing,
  a warning says the notch has not saved anything yet (`about/paths.rs`).

The window's memory and CPU figures are in [performance baseline](performance-baseline.md#settings-window). To add a setting or a
section, see [add a setting](../how-to/add-a-setting.md).

The window's layers, row layout and option cards follow the "Settings window" rules in [design-rules.md](design-rules.md#settings-window).
