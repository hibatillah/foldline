# Settings keys, environment variables and defaults

> **Reference.** Read when: you need a key, its default, an environment variable, or where a tunable lives in the code. Skip if: you are adding a key; the steps are in [add a setting](../how-to/add-a-setting.md).

## Settings file

`%APPDATA%\notch\settings.txt` is a plain text file with one `key=value` per line. The notch and the settings window
write it one key at a time through a temporary file, so keys another program wrote are kept.

Settings are edited in the settings window (`notch-settings.exe`), which the gear in the open header and the tray's **Settings** item open; the notch has no settings panel of its own.

The todo page keeps its list in `%APPDATA%\notch\todo.txt`, one task per line as a markdown checkbox (`- [ ] task`,
`- [x] done`). Any other line loads as an open task, so the file can be edited by hand while the notch is closed.

After a write, the settings window posts `WM_SETTINGS_CHANGED` (`WM_APP + 30`) to the window of class `NotchWindow`. The
notch then re-reads the file and applies the `notch.*` keys except `notch.top`, `navigation.order`, `navigation.disabled`,
`activity.notifications`, `activity.device_battery`, `media.lyrics`, `timer.*_minutes`, `calendar.*` and `notifications.show_text` keys without a
restart, keeping the page that is open. A missing or unreadable value keeps the current one. A hand edit is read at the
next start, or at once if something posts that message.

| Key | Values | Meaning |
| --- | --- | --- |
| `pages` | comma-separated page ids | which pages are on (see above) |
| `media.lyrics` | `on`, `off` | show lyrics on the media page; only the speech-bubble button on the media page changes it (the settings window has no field for it) |
| `timer.focus_minutes` | 5 to 120 | length of a focus session (default 25); the timer page changes it in steps of 5 |
| `timer.short_break_minutes`, `timer.long_break_minutes` | minutes | break lengths (default 5 and 15) |
| `timer.state`, `timer.completed` | written by the notch | the running timer and the focus sessions finished in this round, so a restart keeps them |
| `navigation.order`, `navigation.disabled` | page ids, comma-separated | the order of the pages and the ones switched off; the settings window's Pages section writes them |
| `notifications.show_text` | `on`, `off` | show what a notification says in its banner (off is private mode). Switch the banner itself off with `activity.notifications` |
| `calendar.ics_url` | a web address | the calendar address; the settings window sets it. It is a secret: anyone with it can read the calendar |
| `advanced.debug_log` | a file path | the file the notch appends its timestamped log to; empty means no log. Read once at start, and `NOTCH_DEBUG` overrides it |
| `notch.top` | pixels, 0 to 2000 | how far the notch sits below the top edge of the monitor it is on (default 0). Read once at start, and `NOTCH_TOP` overrides it |
| `notch.monitor` | `main`, `focused` | which monitor the notch sits on: the primary one (`main`, the default) or the one with the window in front (`focused`). A missing key keeps the current choice and an unknown word reads as `main`. Applies without a restart; the settings window labels it **Show the notch on**. See [the layout](layout.md#where-the-notch-sits-on-several-monitors) |
| `calendar.view` | `month`, `week` | the calendar page's left column: the whole month (default) or the current week; the week page is the default 86 px high and its list scrolls. The settings window's Calendar section writes it |
| `calendar.refresh_minutes` | minutes | how often the calendar is downloaded again (default 15) |
| `location.city`, `location.lat`, `location.lon`, `location.country` | text, degrees, degrees, a two-letter country code | the one place the prayer times and the weather are for, chosen with the location search in the settings window's General section. Older versions saved these as `shalat.city`, `shalat.lat`, `shalat.lon` and `shalat.country`; those are read until the shared keys are saved. |
| `shalat.method` | `auto`, `singapore` (Kemenag), `mwl`, `northamerica`, `karachi`, `ummalqura`, `egyptian` | the calculation method; `auto` (default) picks one from `shalat.country` |
| `shalat.hanafi` | `on`, `off` | off by default (Shafi'i, Maliki and Hanbali Asr); on uses the Hanafi Asr |
| `shalat.offset_fajr`, `_sunrise`, `_dhuhr`, `_asr`, `_maghrib`, `_isha` | minutes, -30 to 30 | added to that time (default 0) |
| `shalat.message`, `shalat.badge` | `on`, `off` | on by default: a message when a prayer starts, and the next prayer in the collapsed pill for the last 30 minutes |
| `location.track` | `on`, `off` (default `on`) | the saved `location.*` city follows the device: Windows' position is read once at start, when the switch goes on, after sleep and 10 s after the last network change, and the nearest listed city replaces the saved one when it is another city and the device is more than 25 km from the saved point (`places::city_to_save`). Off reads nothing and keeps the city. Needs Windows location access for desktop apps. The **Track location** switch in General sets it (`host/location_tracker.rs`) |
| `weather.rain_alert` | `on`, `off` (default `on`) | a banner and then a countdown badge in the collapsed pill from 30 minutes before the next rain; the weather page itself is off until you add it. The place is the shared `location.*`. The **Rain alert** switch under Weather in the settings window sets it |
| (none) | | the clipboard page has no settings key of its own: it is switched on and off in the Pages list (`navigation.disabled`), and its history is never saved |
| `stats.disks` | drive names, comma-separated (`C:,G:`) | the disks the stats page shows, in that order, up to 3 (more are dropped); chosen with the checkboxes under System stats in the settings window, which writes the chosen drives in the known order. Without the key the page shows the first 3 fixed drives; an empty value shows none. A name that is no longer present is skipped and the others keep their order; if none is present the Disks card is empty |
| `stats.gpus` | GPU names separated by `\|` (`RTX 4050\|Radeon 760M`); a list without `\|` splits at commas | the GPUs the stats page shows, in that order, up to 2; chosen under System stats in the settings window, which writes them joined with `\|`. Without the key the page shows the first 2; an empty value shows none and drops the GPU card. A name that is no longer present is skipped; with none present the GPU card is dropped |
| `stats.known_disks` | drive names, comma-separated (`C:,G:`) | written by the notch when the stats page starts sampling (it is shown), only when the list changed: every fixed drive, in drive-letter order. The settings window offers these for `stats.disks`. Absent until the stats page has been shown once; leave it alone: the notch owns it |
| `stats.known_gpus` | GPU names separated by `\|` (`RTX 4050\|Radeon 760M`) | written like `stats.known_disks`: every hardware GPU, in the order Windows lists them, by the short names `stats.gpus` uses. `\|` cannot appear in a name (the notch replaces it and line breaks with a space). An empty value means the machine has no hardware GPU; absent means the stats page was never shown |
| `general.language` | `auto`, `en`, `id` | `auto` by default: the language of the notch and the settings window. `auto` is Indonesian when the first Windows preferred UI language starts with `id`, English otherwise; an unknown word reads as `auto`. The texts are in `crates/notch/locales/app.yml` and `crates/settings/locales/app.yml` |
| `notch.hide_when_idle` | `on`, `off` | off by default: hide the notch over regular windows until you point at the top of the screen; only playing music, a short message or a banner shows it, a lasting badge or dot (Caps Lock, a countdown, the camera or microphone dot) does not; the muted speaker does while `notch.hide_windows_volume_popup` is on ([priority activities](#what-the-notch-shows-over-another-window)) |
| `notch.hide_windows_volume_popup` | `on`, `off` | on by default: keep the volume and brightness flyout Windows shows above the taskbar from drawing, while the notch runs (`host/volume_flyout.rs`). Applies without a restart; the flyout returns when the notch exits or the switch goes off, and after a forced end once the notch has been started and closed again. While it is on, a volume or brightness change and the muted speaker show over every `notch.maximized` and `notch.fullscreen` word, `hidden` included. See [hide Windows' own popups](../how-to/hide-windows-popups.md) |
| `notch.hide_windows_notifications` | `on`, `off` | on by default: turn off Windows' notification banner for every app while the notch runs, by writing `ShowBanner=0` per app and saving the previous values in `%APPDATA%\notch\notification-banners.txt` (`host/notification_banners.rs`). Notifications stay in the notification center, so the notch still shows them. Applies without a restart; the values go back when it is switched off, when the notch quits, and at the next start after a crash. See [hide Windows' own popups](../how-to/hide-windows-popups.md) |
| `notch.paused_until` | empty, `manual`, Unix seconds | empty (not paused) by default: while paused the notch is hidden completely and Windows' own volume flyout and notification banners return. `manual` lasts until Resume; a number is the Unix second a timed pause ends. The settings window (**Pause Notch** on General), the tray (**Pause Notch**, then **Resume Notch**) and the notch itself (when a timed pause ends) write it. The notch clears it at every start, so a quit or a crash never leaves it paused; any other text, or a time not after now, reads as not paused and is cleared. Applies without a restart. See [what a pause hides](#pausing-the-notch) |
| `notch.maximized`, `notch.fullscreen` | `hidden`, `activities`, `lyrics`, `music` | what the notch shows over a maximized window (default `lyrics`) and over a full-screen app (default `hidden`); the words are in [the next table](#what-the-notch-shows-over-another-window). A missing or unknown word uses the default |
| `notch.hover_maximized`, `notch.hover_fullscreen` | `on`, `off` | on by default: pointing at the top edge opens the notch over a maximized window and over a full-screen app (the settings window labels them **Open by pointing**; the key names keep the older word). Off stops the pointer from opening it there; a pinned notch stays open whatever they say; short messages still follow `notch.maximized` and `notch.fullscreen`, and dragging files onto the shelf still opens it. Neither key applies over a screenshot overlay (Win+Shift+S, Snipping Tool, Game Bar): the notch is hidden there and the pointer never opens it |
| `pages.known` | `id:Label,...`, with `:opt_in` after the label of an opt-in page | written by the notch at start, only when it changed: the pages the settings window lists (label is `Page::name`; an opt-in page shows as off until `navigation.disabled` is saved). Leave it alone: the notch owns it |
| `navigation.page` | a page id | the page that was open last; the notch opens on it and writes this when you change page |
| `activity.notifications` | `on`, `off` | on by default: show Windows notification banners in the notch |
| `activity.device_battery` | `on`, `off` | on by default: a message when a connected Bluetooth device's battery falls to 20 % or 10 %. Read at start; switching it off while the notch runs hides the message but the watcher keeps running until exit |
| `activity.downloads` | `on`, `off` | on by default: a banner when a browser download starts and finishes, and an icon while one runs; the settings window labels it **Browser downloads**. Reads file names in the Downloads folder only (see [activities](activities.md)). Applies without a restart: switching it on starts the folder watcher, switching it off hides the banners and the icon, but the watcher thread runs until the notch exits |

Pages add their own keys, always starting with the page id. A page reads and writes them through
`Context::settings`.

### What the notch shows over another window

`notch.maximized` and `notch.fullscreen` take one word each. A word lists what the notch still shows; it hides the strip
for anything the word leaves out.

| Word | The notch shows |
| --- | --- |
| `hidden` | nothing, not even activities (except volume and brightness, below) |
| `activities` | short messages and notification banners (a volume change, a timer alert); a lasting badge or dot (Caps Lock, a countdown, the camera or microphone dot) does not keep the pill up |
| `lyrics` | a message, or music that is playing with lyrics |
| `music` | the same, plus the idle pill with nothing playing |

Volume and brightness are priority activities (`Activity::is_priority`): while `notch.hide_windows_volume_popup` is on,
the notch is the only feedback for them, so their level message and the muted speaker show over every word, `hidden`
included, and with `notch.hide_when_idle`. The muted speaker is a lasting badge, so it keeps the pill up for as long as
the sound is muted, and the whole collapsed pill shows then (the clock, Caps Lock, countdowns, the camera and
microphone dots), not only the muted icon. Pointing at the top edge still follows the word and `notch.hover_maximized` or
`notch.hover_fullscreen`. With `notch.hide_windows_volume_popup` off, Windows shows its own flyout and the words apply
to volume and brightness as to any activity. A screenshot overlay hides them too.

A pinned notch (the pin in the open header, `host/notch/pin.rs`) overrides every word, `notch.hide_when_idle` and
`notch.hover_maximized` / `notch.hover_fullscreen`: it stays visible and open over maximized and full-screen windows
until it is unpinned. Only a screenshot overlay hides it, and it returns pinned. The pin has no key: it is not saved,
and the notch starts unpinned.

### Pausing the notch

A pause (`notch.paused_until`, `host/notch/pause.rs`) wins over everything above: it hides the window and draws nothing,
whatever the pin, `notch.maximized`, `notch.fullscreen`, `notch.hide_when_idle` and the priority activities say. A pinned
notch closes and returns pinned when the pause ends. The pause also gives Windows' popups back: the volume and
brightness flyout and the notification banners show as if the notch were not running, and the notch hides them again
(from `notch.hide_windows_volume_popup` and `notch.hide_windows_notifications`) when the pause ends. Changing any other
setting during a pause leaves it unchanged. A timed pause ends from a one-shot host timer (id 12), at wake from sleep and
at unlock after its end time, whichever comes first; a pause until resumed arms no timer. The tray menu is described in
[host and pages](../explanation/host-and-pages.md#input-and-timers) and the window's group in
[the settings window](settings-window.md#sections).

Three older keys (`notch.hide_for_fullscreen`, `notch.show_over_maximized`, `notch.lyrics_over_maximized`) have no
tray item. The notch reads them only when the new key is missing, at start and on a reload:

| Old key and value | New key and word |
| --- | --- |
| `hide_for_fullscreen=on` | `notch.fullscreen=hidden` |
| `hide_for_fullscreen=off` | `notch.fullscreen=music` |
| `show_over_maximized=on` | `notch.maximized=music` |
| `lyrics_over_maximized=off`, with `show_over_maximized` not `on` | `notch.maximized=activities` |
| none of those | `notch.maximized=lyrics` |

## Environment variables

| Variable | Effect |
| --- | --- |
| `NOTCH_PAGES` | comma-separated page ids to switch on (see above) |
| `NOTCH_LYRICS=off` | never look up lyrics, so nothing about your music leaves the computer |
| `NOTCH_SETTINGS_SOLID=1` | the settings window draws its frame opaque instead of translucent and blurred |
| `NOTCH_DEBUG=<file>` | append a timestamped log of what the notch is doing to this file; overrides `advanced.debug_log` |
| `NOTCH_TOP=<pixels>` | move the notch down by this many pixels, to compare it with another notch app; overrides `notch.top` |

## In the code

Things a developer tunes by editing source:

| What | Where |
| --- | --- |
| which pages exist and their order (the order breaks priority ties) | `crates/notch/src/pages/mod.rs` |
| which activities exist and their order (badges) | `crates/notch/src/activities/mod.rs` |
| when a page is shown | the page's `priority()` |
| how big a page is | the page's `sizes()` |
| corner radius, spring stiffness, window size | `crates/notch/src/host/notch.rs` and `crates/notch/src/host/notch/` |
| text styles | `const FontStyle`s in each page |
| the limits of the performance test | `crates/notch/examples/perf.rs` |

## Start with Windows

The stable installer starts the notch at sign-in unless that is unticked in its options; a pre-release installer has no such option and writes no entry. The tray menu's **Start with
Windows** item and the settings window's General section show whether it does, and switch it: each adds or removes a
`Notch` value under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` that points at the copy of `notch.exe` that is
running. The settings window points it at the `notch.exe` beside `notch-settings.exe`.
