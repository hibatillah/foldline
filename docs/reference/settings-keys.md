# Settings keys, environment variables and defaults

> **Reference.** Read when: you need a key, its default, an environment variable, or where a tunable lives in the code. Skip if: you are adding a key; the steps are in [add a setting](../how-to/add-a-setting.md).

## Settings file

`%APPDATA%\notch\settings.txt` is a plain text file with one `key=value` per line. The notch and the settings window
write it one key at a time through a temporary file, so keys another program wrote are kept.

The todo page keeps its list in `%APPDATA%\notch\todo.txt`, one task per line as a markdown checkbox (`- [ ] task`,
`- [x] done`). Any other line loads as an open task, so the file can be edited by hand while the notch is closed.

After a write, the settings window posts `WM_SETTINGS_CHANGED` (`WM_APP + 30`) to the window of class `NotchWindow`. The
notch then re-reads the file and applies the `notch.*` keys except `notch.top`, `navigation.order`, `navigation.disabled`,
`activity.notifications`, `media.lyrics`, `timer.*_minutes`, `calendar.*` and `notifications.show_text` keys without a
restart, keeping the page that is open. A missing or unreadable value keeps the current one. A hand edit is read at the
next start, or at once if something posts that message.

| Key | Values | Meaning |
| --- | --- | --- |
| `pages` | comma-separated page ids | which pages are on (see above) |
| `media.lyrics` | `on`, `off` | show lyrics on the media page (the speech-bubble button changes this too) |
| `timer.focus_minutes` | 5 to 120 | length of a focus session (default 25); the timer page changes it in steps of 5 |
| `timer.short_break_minutes`, `timer.long_break_minutes` | minutes | break lengths (default 5 and 15) |
| `timer.state`, `timer.completed` | written by the notch | the running timer and the focus sessions finished in this round, so a restart keeps them |
| `navigation.order`, `navigation.disabled` | page ids, comma-separated | the order of the pages and the ones switched off; the settings window's Pages section and the notch's Pages tab write them |
| `notifications.show_text` | `on`, `off` | show what a notification says in its banner (off is private mode). Switch the banner itself off with `activity.notifications` |
| `calendar.ics_url` | a web address | the calendar address; the settings window sets it. It is a secret: anyone with it can read the calendar |
| `advanced.debug_log` | a file path | the file the notch appends its timestamped log to; empty means no log. Read once at start, and `NOTCH_DEBUG` overrides it |
| `notch.top` | pixels, 0 to 2000 | how far the notch sits below the top edge of the screen (default 0). Read once at start, and `NOTCH_TOP` overrides it |
| `calendar.refresh_minutes` | minutes | how often the calendar is downloaded again (default 15) |
| `shalat.city`, `shalat.lat`, `shalat.lon`, `shalat.country` | text, degrees, degrees, a two-letter country code | the chosen city's name, position and country; the shalat settings' city search writes all four. Without a position the page asks for a city |
| `shalat.method` | `auto`, `singapore` (Kemenag), `mwl`, `northamerica`, `karachi`, `ummalqura`, `egyptian` | the calculation method; `auto` (default) picks one from `shalat.country` |
| `shalat.hanafi` | `on`, `off` | off by default (Shafi'i, Maliki and Hanbali Asr); on uses the Hanafi Asr |
| `shalat.offset_fajr`, `_sunrise`, `_dhuhr`, `_asr`, `_maghrib`, `_isha` | minutes, -30 to 30 | added to that time (default 0) |
| `shalat.message`, `shalat.badge` | `on`, `off` | on by default: a message when a prayer starts, and the next prayer in the collapsed pill for the last 30 minutes |
| `stats.disks`, `stats.gpus` | names, comma-separated | the disks (`C:`) and GPUs the stats page shows, in that order: up to 3 disks and 2 GPUs. Written by the stats settings in the notch's panel; without the key the page shows the first ones |
| `notch.hide_when_idle` | `on`, `off` | off by default: hide the notch when there is nothing to show until you point at the top of the screen |
| `notch.maximized`, `notch.fullscreen` | `hidden`, `activities`, `lyrics`, `music` | what the notch shows over a maximized window (default `lyrics`) and over a full-screen app (default `hidden`); the words are in [the next table](#what-the-notch-shows-over-another-window). A missing or unknown word uses the default |
| `notch.hover_maximized`, `notch.hover_fullscreen` | `on`, `off` | on by default: pointing at the top edge opens the notch over a maximized window and over a full-screen app. Off stops the pointer from opening it there; short messages still follow `notch.maximized` and `notch.fullscreen`, and dragging files onto the shelf still opens it |
| `pages.known` | `id:Label,...`, with `:opt_in` after the label of an opt-in page | written by the notch at start, only when it changed: the pages the settings window lists (label is `Page::name`; an opt-in page shows as off until `navigation.disabled` is saved). Leave it alone: the notch owns it |
| `navigation.page` | a page id | the page that was open last; the notch opens on it and writes this when you change page |
| `activity.notifications` | `on`, `off` | on by default: show Windows notification banners in the notch |

Pages add their own keys, always starting with the page id. A page reads and writes them through
`Context::settings`.

### What the notch shows over another window

`notch.maximized` and `notch.fullscreen` take one word each. A word lists what the notch still shows; it hides the strip
for anything the word leaves out.

| Word | The notch shows |
| --- | --- |
| `hidden` | nothing, not even activities |
| `activities` | notification banners, volume changes and badges |
| `lyrics` | an activity, or music that is playing with lyrics |
| `music` | the same, plus the idle pill with nothing playing |

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
| `NOTCH_DEBUG=<file>` | append a timestamped log of what the notch is doing to this file; overrides `advanced.debug_log` |
| `NOTCH_TOP=<pixels>` | move the notch down by this many pixels, to compare it with another notch app; overrides `notch.top` |

## In the code

Things a developer tunes by editing source:

| What | Where |
| --- | --- |
| which pages exist and their order (the order breaks priority ties) | `crates/notch/src/pages/mod.rs` |
| which activities exist and their order (optional tray entries, badges) | `crates/notch/src/activities/mod.rs` |
| when a page is shown | the page's `priority()` |
| how big a page is | the page's `sizes()` |
| corner radius, spring stiffness, window size | `crates/notch/src/host/notch.rs` and `crates/notch/src/host/notch/` |
| text styles | `const FontStyle`s in each page |
| the limits of the performance test | `crates/notch/examples/perf.rs` |

## Start with Windows

The installer starts the notch at sign-in unless that is unticked in its options. The tray menu's **Start with
Windows** item and the settings window's General section show whether it does, and switch it: each adds or removes a
`Notch` value under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` that points at the copy of `notch.exe` that is
running. The settings window points it at the `notch.exe` beside `notch-settings.exe`.
