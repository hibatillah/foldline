# Architecture

> **Explanation.** Read when: you are new to the code, or you need to know which program, crate, file or service a change touches. Skip if: you need a file's location ([source map](docs/reference/source-map.md)), a step to follow ([how-to guides](docs/README.md#how-to-guides-do-a-task)) or how the host and pages work inside ([host and pages](docs/explanation/host-and-pages.md)).

## The app in two states

The notch is one window at the top of the screen that shows one page at a time, in one of two states:

- **collapsed**: a slim pill, shown while the pointer is away (`Mode::Resting` in `crates/notch/src/page.rs`);
- **open**: the full notch with its header and the page's content (`Mode::Expanded`), either opened by the pointer
  (it closes when the pointer leaves) or pinned open by the pin in the header (it stays until unpinned).

The app is two programs. `notch.exe` is the notch, drawn with Direct2D and no UI framework, and it alone is held to
the [performance budget](docs/explanation/performance-budget.md). `notch-settings.exe` is the settings window, built
with GPUI, started from the gear in the open header or the tray's **Settings** item. What each shows the user is in
[README.md](README.md).

## Project structure

```text
crates/notch/      notch.exe: host/, pages/, activities/, gfx/, ui/, platform/
crates/settings/   notch-settings.exe, the settings window (GPUI)
crates/store/      the settings file code both programs share, and the language both read
crates/places/     the bundled city list and the location.* keys
crates/account/    the account: permission catalog, statuses, grants, the signed token and policy, account.txt, the calls to the account site
vendor/salah/      the prayer-time crate with one fix
scripts/           reload, watch, perf, test-build, release notes, licenses, the city list
installer/         the Inno Setup script
assets/            branding, fonts, images, cities.tsv
docs/              how-to, reference and explanation, indexed in docs/README.md
```

Every file inside these folders, and which module may use which, is in the [source map](docs/reference/source-map.md).

## System diagram

```text
  Windows: messages, event hooks, WinRT watchers           LRCLIB, calendar address, MET Norway
          |                                                          ^
          v                                                          | HttpClient, on worker threads
 +---------------------------- notch.exe ---------------------------------------------+
 |  host/window.rs  -->  Notch (host/)  -->  pages: priority(mode), draw                |
 |                          |                activities: presence()                     |
 |  worker threads ---------+ Notifier wakes the UI thread                              |
 |                          v                                                           |
 |                 layered window, drawn with Direct2D                                  |
 +--------------------------------------------------------------------------------------+
          |  reads keys; writes pages.known      ^  WM_SETTINGS_CHANGED
          v                                      |  (posted to the NotchWindow class)
 +--------------------------------+              |
 | %APPDATA%\notch\settings.txt   |              |
 +--------------------------------+              |
          ^  writes keys through store           |
          |                                      |
 +--------------------- notch-settings.exe ------+-----------+
 |  sections/  -->  store  -->  notify.rs                     |
 +------------------------------------------------------------+
```

The two programs never call each other's code. The settings file, `account.txt` (see [Data stores](#data-stores)) and one window message are the whole link: the
settings window writes a key through `crates/store` and posts `WM_SETTINGS_CHANGED` (`crates/settings/src/notify.rs`),
and the notch reads the keys again. The notch writes `pages.known` at start, so the settings window can list pages it
does not link. The message, the keys and their defaults are in
[settings keys](docs/reference/settings-keys.md#settings-file).

## Core components

| Component | Job | Detail |
| --- | --- | --- |
| `notch.exe` (`crates/notch`), shown in Task Manager as "Notch" | `host/` owns the window, the monitor it sits on (`notch.monitor`, [layout](docs/reference/layout.md#where-the-notch-sits-on-several-monitors)), opening and closing, the pin, the pause, the springs, input, the tray and the launcher of the settings window, and names no page. Pages and activities are each listed in one registry (`pages/mod.rs`, `activities/mod.rs`). `gfx/` and `ui/` draw with Direct2D and DirectWrite; pages, activities and widgets draw in a unit space, converted to device pixels only in `host/notch/units.rs`. | [host and pages](docs/explanation/host-and-pages.md), [drawing model](docs/explanation/drawing-model.md), [page API](docs/reference/page-api.md), [activities](docs/reference/activities.md) |
| `notch-settings.exe` (`crates/settings`), shown in Task Manager as "Notch Settings" | the settings window: GPUI and gpui-component, one file per section, one window at a time, writes keys and tells the notch | [settings window](docs/reference/settings-window.md), [add a setting](docs/how-to/add-a-setting.md) |
| `store` (`crates/store`) | the settings file as `key=value` lines with per-key atomic writes, and `language.rs`, which resolves `general.language` for both programs | [settings keys](docs/reference/settings-keys.md) |
| `places` (`crates/places`) | the bundled city list behind the location search and the `location.*` keys that prayer times and weather share | [settings keys](docs/reference/settings-keys.md) |
| `account` (`crates/account`) | the permission catalog, the statuses and grants, the signed token and policy (verified with Windows CNG), the DPAPI-protected `account.txt`, and the `Windows.Web.Http` calls to the account site; the notch decides access with it and the settings window signs in with it | [accounts and permissions](docs/explanation/accounts-and-permissions.md), [permissions](docs/reference/permissions.md) |

`crates/notch` and `crates/settings` depend on `crates/store`, `crates/places` and `crates/account` and never on each other, so the
notch links no UI framework. Both programs look up their texts in English and Indonesian with `tr!`; how a language
change reaches cached text is in [host and pages](docs/explanation/host-and-pages.md#text-in-two-languages).

## Data stores

| Store | Where | Owner |
| --- | --- | --- |
| settings | `%APPDATA%\notch\settings.txt` | `crates/store` |
| account: device identity, the last signed token and policy, the last check time | `%APPDATA%\notch\account.txt`, written atomically (a `.tmp` file renamed over it) with the device secret DPAPI-protected, by the settings window's sign-in and sign-out and by the notch's daily check | `crates/account/src/stored.rs` |
| todo list | `%APPDATA%\notch\todo.txt` | `pages/todo/list.rs` |
| shelf | `%APPDATA%\notch\shelf.txt` | `pages/shelf/` |
| calendar cache, the last download | `%APPDATA%\notch\calendar.ics` | `pages/calendar/` |
| the `ShowBanner` values the notch replaced, put back after a crash | `%APPDATA%\notch\notification-banners.txt` | `host/notification_banners/` |
| clipboard history | memory only | `pages/clipboard/history.rs` |
| Start with Windows | the current user's `Run` registry value, written by the settings window and the installer | `crates/settings/src/startup.rs` |
| banners off | each app's `ShowBanner` registry value | `host/notification_banners.rs` |

What the user is told about each of these is in [README privacy](README.md#privacy); keys and defaults are in
[settings keys](docs/reference/settings-keys.md).

## External integrations

Network requests use WinRT `Windows.Web.Http.HttpClient` on a worker thread:

| Service | Used for | Code |
| --- | --- | --- |
| LRCLIB | synced lyrics, the only lyrics provider | `pages/media/lyrics/client.rs` |
| the calendar address the user sets | the ICS download | `pages/calendar/fetch.rs` |
| MET Norway Locationforecast | the forecast and the rain alert | `pages/weather/fetch.rs` |
| weather.com | opened in the browser on a click, never fetched | `pages/weather/link.rs` |
| the account site | the sign-in claim, sign-out and the daily check of the token and policy; a release build makes no request to it until accounts are enforced (`account::keys::is_enforced`) ([why](docs/explanation/accounts-and-permissions.md#why-a-release-build-locks-nothing-yet)) | `crates/account/src/client.rs`, called from `host/notch/daily_check.rs`, `host/account_check.rs` and `crates/settings/src/sign_in/` |

The city search reads the bundled `assets/cities.tsv` and sends nothing. The device position comes from Windows
(`Geolocator`, `platform/geolocation.rs`), asked again after a network change (`host/location_tracker/`). What each
request sends is in [README privacy](README.md#privacy).

Windows APIs by area:

| Area | API | Code |
| --- | --- | --- |
| window and surface | layered window, `UpdateLayeredWindow` | `host/window.rs`, `gfx/surface.rs` |
| monitors | `EnumDisplayMonitors`, `GetDpiForMonitor` | `host/monitor.rs`, `host/notch/relocate.rs` |
| window in front, volume flyout | `SetWinEventHook`, one thread at a time | `host/screen_mode.rs`, `host/volume_flyout.rs` |
| session lock and unlock | `WTSRegisterSessionNotification` | `host/window.rs` |
| tray, dropped files | `Shell_NotifyIcon`, `RegisterDragDrop` | `host/tray.rs`, `host/drop_target.rs` |
| media | `GlobalSystemMediaTransportControlsSessionManager` | `pages/media/session.rs` |
| notifications | `UserNotificationListener` | `activities/notifications/listener.rs` |
| Downloads folder | `ReadDirectoryChangesW` on file names, one thread | `activities/downloads/watcher.rs` |
| volume | `IAudioEndpointVolume` | `activities/volume.rs` |
| brightness, power, clipboard | power-setting and clipboard window messages | `page.rs`, `activities/brightness.rs` |
| devices | `DeviceWatcher` | `activities/device_battery/` |
| Caps Lock | a keyboard hook, `SetWindowsHookEx` | `activities/caps_lock.rs` |
| GPU load | PDH counters | `pages/stats/gpu.rs` |

## Build, test and release

- Build, run and debug: [build, run and debug](docs/how-to/build-run-and-debug.md).
- Tests and page pictures: [test and snapshot](docs/how-to/test-and-snapshot.md), [test map](docs/reference/test-map.md).
- CPU and memory: [run the performance check](docs/how-to/run-the-perf-check.md).
- Branches and pull requests: [work with branches](docs/how-to/work-with-branches.md). `.github/workflows/ci.yml` runs on
  pull requests into `master`.
- Releases: [cut a release](docs/how-to/cut-a-release.md). `.github/workflows/release.yml` runs on a pushed `v*` tag.

## Glossary

| Term | Meaning |
| --- | --- |
| collapsed | the slim pill shown while the notch is not open; `Mode::Resting` |
| open | the full notch with its header and a page's content; `Mode::Expanded`. Opened by the pointer, it closes when the pointer leaves; pinned open, it stays (`host/notch/open_state.rs`) |
| pinned | open until the user presses the header pin again, over maximized and full-screen windows too; hidden only by a screenshot overlay, never saved (`host/notch/pin.rs`) |
| paused | hidden completely, with Windows' own volume flyout and banners back, until a chosen time or until resumed; set from the tray or the settings window through `notch.paused_until`, cleared at every start, and stronger than the pin (`host/notch/pause.rs`, [settings keys](docs/reference/settings-keys.md#pausing-the-notch)) |
| host | `crates/notch/src/host/`: everything not specific to one page |
| page | a view that fills the collapsed pill, the open notch or both; implements `Page` ([page API](docs/reference/page-api.md)) |
| activity | something that reports a change while the notch is collapsed (volume, battery, a notification); implements `Activity` ([activities](docs/reference/activities.md)) |
| presence | what an activity or a page shows while collapsed: a `Transient`, a `Badge` or a `Banner` (`activity.rs`) |
| idle pill | the collapsed pill when no page claims it: the time and the group of status marks (`host/pill_row.rs`) |
| claimed pill | the collapsed pill while a page fills it with its own content, as the media page does with a lyric line (`host/claimed_row.rs`) |
| status mark | the icon a page or an activity shows in the group at the right of the collapsed pill and of the open header, ordered by `header_status_rank` |
| header | the top row of the open notch: page dots, time, status marks, the pin and the gear ([layout](docs/reference/layout.md)) |
| unit space | the device-independent coordinates, starting at zero, in which pages, activities and widgets draw and receive the pointer ([drawing model](docs/explanation/drawing-model.md)) |
| `Notifier` | how a worker thread wakes the UI thread |
| settings file | `settings.txt`, the state both programs read and write; `account.txt` is the other file both touch |

## Where to go next

- Setting up and the rules the code follows: [ONBOARDING.md](ONBOARDING.md).
- Where a file lives: [source map](docs/reference/source-map.md).
- Who may use which page or activity, and why a release build locks nothing yet: [accounts and permissions](docs/explanation/accounts-and-permissions.md); the catalog, statuses and `account.txt` fields: [permissions](docs/reference/permissions.md).
- How the host chooses, sizes, animates and hides pages: [host and pages](docs/explanation/host-and-pages.md).
- Every document by type: [docs/README.md](docs/README.md).
