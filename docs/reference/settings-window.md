# Settings window

> **Reference.** Read when: you need what each section of `notch-settings.exe` sets, or how it saves. Skip if: you are looking for a key's default; it is in [settings keys](settings-keys.md).

Settings are edited only in this window; the notch has no settings panel. Open it from the gear at the right of the open
header or from the tray menu's **Settings** item. Both start `notch-settings.exe` from the folder of the running
`notch.exe` without waiting for it (`crates/notch/src/host/settings_launcher.rs`) and log `could not start settings` when the program
is missing. The window allows one instance: starting it again focuses the open one.

## Layout

The frame is a neutral color (dark `#101010` at 64 %, light white at 35 %) over a blurred acrylic backdrop
(`palette::window_background`), so the desktop shows through; it carries the sidebar directly, with no card and no solid
fill, and has no tint or glow. The content is the one solid surface, a rounded (14 px), bordered card inset 8 px from the
top, right and bottom edges (`FRAME_PADDING`, `CARD_RADIUS` in `crates/settings/src/main.rs`). The sidebar is 216 px wide. The window follows the Windows display scale; `UI_FONT_SIZE` (14.5, `main.rs`, applied as `theme.font_size` in `palette::apply`) is the one knob that shrinks every rem-based size. The colors are in `crates/settings/src/palette.rs`, with one
blue accent; the font is Geist (`theme.font_family`, set in `palette::apply`), and the reasons are in
[design-rules.md](design-rules.md#settings-window). Where the blur is unwanted, start the window with
`NOTCH_SETTINGS_SOLID=1`: the frame is then a neutral opaque color (`#141414` dark, `#eeeeee` light). gpui 0.2.2 offers only `Opaque`, `Transparent` and
`Blurred` on Windows (no Mica), and cannot report whether blur works, so the frame keeps a tint that keeps the text readable
over most desktops.

| Part | What it holds | Source |
| --- | --- | --- |
| Brand strip | the sidebar's 16 px top padding, then the squared tile logo (28 px: blue `tile-stable.png` on a stable build, indigo `tile-dev.png` on a pre-release, chosen by `release_version`), "Notch" (`text_lg` semibold) and the version (`text_xs`, muted), 8 px apart and 18 px from the left edge, with 4 px below; a drag area | `main.rs`, `brand.rs`, `sidebar.rs` |
| Sidebar | group "Notch": General, Pages, Behavior, Advanced; group "Page settings": Calendar, Timer, Prayer (the Shalat page; "Shalat" in Indonesian), System stats, Weather; **Account** and then **About** at the bottom (`NavGroup::Footer`, no label). Padding 16 px above, 10 px at the sides, 12 px below. Group labels are `text_sm`, muted, with 12 px above and 5 px below. Items are 36 px high, 1 px apart, with a 10 px radius, a blue 17 px icon 11 px from the `text_base` label; the empty sidebar (gutters, space below the items, group labels) is a drag area, the items are not; the selected one has a faint fill and a medium-weight label | `sidebar.rs`, `sections/mod.rs` (`all`) |
| Sidebar icons | SVG files in `crates/settings/assets/icons`, served by `WindowAssets`, all drawn in the accent blue: a hex nut for General (the notch's gear), Lucide layers (Pages), puzzle (Behavior), calendar-days (Calendar), timer (Timer), moon-star (Prayer), chart-column (System stats), cloud-drizzle (Weather), wrench (Advanced), circle-user (Account) and info (About); folder is the icon of the Files section, and lock marks a locked page on Pages | `icons.rs` |
| Card header | 44 px, a little lighter than the card, drawn by the window and not by the component library's title bar: the breadcrumb "Settings › <Page>" (`text_base`, 22 px from the card's left edge, 10 px around the 13 px chevron, the page in blue and medium weight), then minimize, maximize and close at the right end; a drag area. Its top corners are rounded to the card's, and the close button rounds its own, so the red hover follows the corner | `header.rs` |
| Card content | padded 16 px above, 22 px at the sides and 20 px below. General, Pages, Behavior, Advanced, Account and About open straight with their first section (`Page::plain`); the page settings (Calendar, Timer, Prayer, System stats, Weather) open the same way; a section may carry a muted note under its title (`Group::description`). Sections sit 14 px apart. A section opens with a heading row (`text_sm`, muted: its title at the left, with a small blue icon where one fits, then set at 14 px in the full title color (Files); an optional note at the right), then its parts 7 px apart: blocks (12 px radius, one step lighter than the card, `#2a2a2d` dark) of rows with dividers between them, and items drawn outside a block (`Item::outside_block`, the Behavior cards, the hint boxes). A row is at least 50 px high with 9 px by 14 px padding: its title (`text_base`, medium, with an optional tag such as "After restart" after it: `Item::tag`, `tag.rs`) and description (`text_sm`, 17 px lines) at the left, 12 px from the control at the right. A picture row (`Item::picture_row`) holds pictures or a wide control instead, padded 12 px by 14 px. Pictures of the desktop are drawn with GPUI elements, never images (`illustration.rs`: a dark blue scene, a window, the notch, a strip of equal pictures with captions). A hint box (`info_box.rs`) is a blue-bordered box with a blue icon and a sentence. Switches are 42 x 25 (`switch.rs`); buttons are rounded (8 px, 30 px high) a step lighter than the block with a thin border, and an inert one is dimmed with "Coming soon" to its left; picture cards for option pickers (10 px radius, 1.5 px border, 8 px padding, 6 px between parts, 10 px between cards; title 13.5 px semibold, description 12 px on 16 px lines) | `layout.rs`, `field.rs`, `card.rs`, `picture.rs`, `illustration.rs`, `info_box.rs`, `tag.rs`, `sections/about/widgets.rs` |

## Sections

| Section | What you set | Key |
| --- | --- | --- |
| General | Startup: start the notch at sign-in | the `Run` registry value, see [Start with Windows](settings-keys.md#start-with-windows) |
| General | Startup: hide the notch unless music plays or a message or banner shows, with three pictures under it (the notch in the tray, the pill hidden while idle, the pill back when the pointer reaches the top edge) | `notch.hide_when_idle` |
| General | Location: a **City** row, then the search field (up to five matches, the saved city checked; "Prayer times" and "Weather" chips say who uses it) beside a drawn map (street grid, river, a pin, the city's name and its coordinates; a picture, not a real map), then **Track location** | `location.city`, `location.lat`, `location.lon`, `location.country`, `location.track` |
| General | Notifications: show Windows notification banners (its description points to the switch that hides Windows' own banner); show what a notification says, with two banner cards under it (Show the message, Hide the message: "New message") that set the same key; warn when a connected device's battery is low; show browser downloads (**Browser downloads**, on by default; its description says only file names in the Downloads folder are read) | `activity.notifications`, `notifications.show_text`, `activity.device_battery`, `activity.downloads` |
| General | **Show the notch on**: three picture cards, Main screen, Focused screen and A chosen screen, each drawing three screens with the notch on the one that choice picks (default Main screen; a missing or unknown word shows Main screen). While A chosen screen is selected a **Chosen screen** row draws the connected screens where they stand on the desktop, scaled to fit (`store::screens::list()`, read when the page paints, no polling): each shows its number, its size and "Main" on the primary, the chosen one is outlined in blue with the notch on it, pointing at one shows "Screen N, model, 1920x1080 (main)", and a click stores its id. A stored id missing from the list says the notch uses the main screen until it returns; painting never overwrites the stored id. The heading says it applies at once | `notch.monitor`, `notch.monitor_id` |
| General | Windows popups: hide the Windows volume and brightness popups; hide Windows notification banners (both on by default) | `notch.hide_windows_volume_popup`, `notch.hide_windows_notifications` |
| Timer | focus, short break and long break lengths, each a minus and plus stepper, with a picture of the round and its total | `timer.focus_minutes`, `timer.short_break_minutes`, `timer.long_break_minutes` |
| Pages | "How it looks in the notch": the open header at the first, the third (or middle) and the last page shown, one dot per page shown and the long dot on the current one, captioned with its key ("First page (key 1): Media", "Last page (key 0): ..."). Then the list, headed "N pages · M on · drag to reorder": one row per page with its key (1 to 9 in a blue square for the pages shown, a dashed "–" for a page that is off or locked; the last page shown carries an "Also key 0" tag, or 0 when it comes after the ninth), a drag handle, the page's icon, its name and a one-line description, and its switch; a page that is off is dimmed. Drag a row to reorder it. Two hint boxes follow: the digit keys (while the pointer is on the open notch, `host/notch/page_hotkeys.rs`) and that a page switched off keeps its place. The opt-in Clipboard page starts switched off and has no section of its own. A page the account does not unlock shows a blue lock and a pill before its switch: **Sign in** (opens the Account page) for a guest, **Upgrade** (opens `<site>/pricing`) otherwise; display only, the notch enforces it. See [Account](#account) | `navigation.order`, `navigation.disabled` |
| Behavior | one section each for a maximized window and a full-screen app (the heading row says when it applies): four picture cards in one row, each a 78 px picture above the level and its sentence (Hidden, Activities only, Lyrics, Always visible; each level also shows everything below it), the selected one with a blue check after its title, then **Open on hover** (a pinned notch stays open regardless) | `notch.maximized`, `notch.fullscreen`, `notch.hover_maximized`, `notch.hover_fullscreen` |
| Calendar | the calendar address and how often it is downloaded again; the section lists the steps for finding a Google Calendar address | `calendar.ics_url`, `calendar.refresh_minutes` |
| System stats | Disks (one checkbox per known disk, at most 3) and GPUs (one per known GPU, at most 2); unset shows the first 3 disks and first 2 GPUs, and the first click writes the explicit list in the notch's order (GPUs joined with `|`). Until the notch has published a list the block says to open the System stats page in the notch once; an empty GPU list says "No GPU found" | `stats.disks`, `stats.gpus`, read from `stats.known_disks`, `stats.known_gpus` |
| Prayer | a section note that the location is set in General (the page has no city field); each section opens with a bold heading line; the calculation method as picture cards, Auto and six methods, each showing how far below the horizon the sun is at Fajr and at Isha; Standard or Hanafi Asr, drawn as a stick with its shadow; the two reminders with a preview of each; minus 30 to plus 30 minutes for each prayer with a compact minus and plus stepper, under a day bar that shows where each prayer falls | `shalat.method`, `shalat.hanafi`, `shalat.message`, `shalat.badge`, `shalat.offset_fajr`, `_sunrise`, `_dhuhr`, `_asr`, `_maghrib`, `_isha` |
| Advanced | Troubleshooting: the debug log file (a path field with a file icon and **Browse**, which opens Windows' save dialog in the log's folder, else `%APPDATA%\notch`), with three sample lines in the log's own format under it. It carries an "After restart" tag. Language: Automatic, English or Bahasa Indonesia as cards. Pause: a **Pause Notch** row with four buttons while the notch runs (10 min, 30 min, 1 hour, Until I resume); while it is paused the buttons give way to "Paused until 14:30" (24-hour local time) or "Paused until you resume" and a **Resume** button. A pause hides the notch completely and gives Windows' volume flyout and banners back, see [pausing the notch](settings-keys.md#pausing-the-notch) | `advanced.debug_log`, `general.language`, `notch.paused_until` |
| Account | an Account section: sign in with Google, the account and its status, Upgrade, Manage account, Sign out. with the page's sentence (accounts open or not) under the block. See [Account](#account) | none; it writes `%APPDATA%\notch\account.txt` |
| About | the app card in its own block (the build's tile, "Notch", "Version x", and "Updates come from the Microsoft Store" with an **Open in Store** link to the app's Store page); What's new and Licenses rows; under "Files", the settings file and the data folder. See [About](#about) | none; it writes nothing |

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

### Account

The page is `crates/settings/src/sections/account.rs`; the sign-in is `crates/settings/src/sign_in.rs` and its children,
and what the window shows about the account is `crates/settings/src/standing.rs`. It sits in the sidebar's footer, above
About.

- **Signed out:** "Not signed in" and **Sign in with Google**. The click makes a PKCE verifier, challenge and `state`
  (`account::pkce`), listens on `127.0.0.1:0` (`sign_in/listener.rs`) and opens
  `<site>/device/login?port&challenge&state&device_hash&name` in the default browser. `<site>` is `https://fold.encure.id` in
  a release build and `NOTCH_ACCOUNT_URL` (default `http://127.0.0.1:8787`) in a debug build.
- **Waiting for the browser:** a **Cancel** button. The wait ends after 5 minutes ("Sign-in stopped after 5 minutes...").
  The browser's `GET /callback?code&state` or `?error=cancelled&state` is checked against the `state` sent
  (`sign_in::parse_callback`); the browser gets a small "You can close this tab and return to Notch." page and the port
  closes. Any other connection (a browser preconnect, `/favicon.ico`, another `state`) gets a 404 within 2 seconds and the
  wait goes on. A cancel shows "Sign-in cancelled." and writes nothing; a port that stops accepting shows its own line.
- **Finishing:** "Finishing sign-in…" with a spinner and a note that it takes up to 15 seconds; the code is claimed off the UI thread (`account::client::claim`, at most 15 s). While signatures are
  enforced the token must verify for this PC (`read_token`); then `account.txt` is written atomically and
  `WM_SETTINGS_CHANGED` is posted, so the notch unlocks without a restart. A refused code, an unreachable site, a token
  that does not verify or a failed write each show their own line and write nothing.
- **Signed in:** the email, the status under it (Free account, Trial until `YYYY-MM-DD`, Paid, Free during launch, or
  "Your sign-in cannot be verified on this PC, so nothing is unlocked. Sign out, then sign in again." when the token
  no longer verifies here) and **Sign out**; **Upgrade** opens `<site>/pricing`
  (hidden for a paid account), **Manage account** opens `<site>/account`. **Sign out** clears the device and token from
  `account.txt` first (the policy stays), tells the notch, then tells the site in the background (best effort).
- **Refresh:** showing the page (a sidebar click or a lock's **Sign in again**) and clicking **Upgrade** (after the
  browser opens) check the account with the site once (`account_refresh.rs`), so a plan bought on the site shows without
  waiting for the notch's daily check. Only in an enforced build, signed in, when `last_check_at` (or the window's last
  attempt) is 2 minutes old or more, and never two at a time; no timer or loop. The outcome goes through
  `account::schedule::settle`: it is dropped when `account.txt` holds another `device_id` by then (the notch signed out
  or in meanwhile), otherwise it is written atomically and `WM_SETTINGS_CHANGED` tells the notch. A purchase made after
  the click shows when the Account page is shown again.
- **Unenforced build:** a release build with no production key verifies nothing, locks nothing and makes no request to
  the site and opens none of its pages. The page reads "Notch accounts are not open yet, so every page and feature works
  without one." and its one row "Accounts open soon. Everything is free until then.", with no sign-in button, no
  **Upgrade**, no **Manage account** and no lock badges. An account stored by an earlier build shows its email (read from
  the token without a check, for display only) and **Sign out**, which clears `account.txt` and sends nothing to the site.

**Sign in again:** a signed-in PC whose token does not verify here reads as a guest, and its locks say **Sign in again**
(opens the Account page, which explains it under the email) instead of **Sign in**.

**Debug status:** in a debug build `NOTCH_ACCOUNT_STATUS` forces the status the window shows and locks by, parsed by
`account::status::parse_forced_status` exactly as the notch parses it, so both agree.

**Locks on other rows:** the same lock and **Sign in** / **Upgrade** pill sit before the switch of **Rain alert** (Weather,
`weather`/`rain_alert`) and of the activity switches on General that the account refuses (**Show notifications**,
**Warn when a device battery is low**, **Browser downloads**: `activity`/`<id>`); `sections/lock.rs`. The settings window has no lyrics
switch (`media.lyrics` changes only on the media page), so lyrics shows no lock here.

The window reads `account.txt` again when its modified time changes (a sign-in or refresh here, or the notch's daily check) or once a
trial or launch end it read has passed, checked on paint with no timer;
the code, verifier, secret and token are never logged.

### About

The page is `crates/settings/src/sections/about/`. It sits in the sidebar's footer, below Account.

- **App card:** the tile of the build's channel (the dev tile for a pre-release version, the stable tile otherwise, chosen by
  `release_version::icon_for_version` like the executables' icons), "Notch" and "Version x". Updates come from the Microsoft Store only (no in-app update check): the card says "Updates come from the Microsoft Store" and an **Open in Store** link (the constant
  `STORE_PRODUCT_ID` in `about/mod.rs` holds the id `9NGTV96SZDS7`) to `ms-windows-store://pdp/?productid=<id>`;
  while the constant is empty the link is hidden (`open_store()` returns no handler).
- **What's new:** inert until the constant `WHATS_NEW_URL` in `about/mod.rs` is set to the release notes address.
- **Licenses:** opens `THIRD-PARTY-LICENSES.txt`, looked for in the folder of `notch-settings.exe` and then in up to six
  parent folders (so `run/` and `target/.../release` find the repository-root file; the installer puts it beside the
  program, and `scripts/reload.sh` copies it into `run/`). When it is not found, a warning notification in the window's top
  right corner says so (`about/licenses.rs`).
- **Files:** **Settings file** opens `%APPDATA%\notch\settings.txt` (the folder when the file does not exist yet);
  **Data folder** opens `%APPDATA%\notch`, which holds `todo.txt`, `shelf.txt` and `calendar.ics`. When the folder is missing,
  a warning says the notch has not saved anything yet (`about/paths.rs`).

The window's memory and CPU figures are in [performance baseline](performance-baseline.md#settings-window). To add a setting or a
section, see [add a setting](../how-to/add-a-setting.md).

The window's layers, row layout and option cards follow the "Settings window" rules in [design-rules.md](design-rules.md#settings-window).
