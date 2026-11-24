# Architecture

> **Explanation.** Read when: you are new to the code, or you need to know which program, crate, file or service a change touches. Skip if: you need a file's location ([source map](docs/reference/source-map.md)), a step to follow ([how-to guides](docs/README.md#how-to-guides-do-a-task)) or how the host and pages work inside ([host and pages](docs/explanation/host-and-pages.md)).

## The app in two states

The notch is one window at the top of the screen that shows one page at a time, in one of two states:

- **collapsed**: a slim pill, shown while the pointer is away (`Mode::Resting` in `crates/notch/src/page.rs`).
- **open**: the full notch with its header and the page's content (`Mode::Expanded`). The pointer opens it and it closes when the pointer leaves, or the pin in the header holds it open until unpinned.

The app is two programs, and the repository holds a third thing beside them, the website. `foldline.exe` is the notch, drawn with Direct2D and no UI framework, and it alone is held to the [performance budget](docs/explanation/performance-budget.md). `foldline-settings.exe` is the settings window, built with GPUI and started from the gear in the open header or the tray's **Settings** item. What each shows the user is in [README.md](README.md). The website is a separate Next.js program that never links the Rust code (see [The website](#the-website)).

## Project structure

```text
crates/notch/      foldline.exe: host/, pages/, activities/, gfx/, ui/, platform/
crates/settings/   foldline-settings.exe, the settings window (GPUI)
crates/store/      the settings file code both programs share: language, data folder, pause, snooze, screens
crates/places/     the bundled city list and the location.* keys
vendor/salah/      the prayer-time crate with one fix
website/           the landing, download, releases and policy pages (Next.js, Bun, Vercel); not part of the Cargo workspace
scripts/           reload, watch, perf, test-build, release notes, licenses, the city list
installer/         the Inno Setup script and the MSIX package files
assets/            branding, fonts, images, cities.tsv
docs/              how-to, reference and explanation, indexed in docs/README.md
```

Every file in these folders, and which module may use which, is in the [source map](docs/reference/source-map.md).

## System diagram

```text
  Windows: messages, event hooks, WinRT watchers           LRCLIB, calendar address, MET Norway
          |                                                          ^
          v                                                          | HttpClient, on worker threads
 +---------------------------- foldline.exe ---------------------------------------------+
 |  host/window.rs  -->  Notch (host/)  -->  pages: priority(mode), draw                |
 |                          |                activities: presence()                     |
 |  worker threads ---------+ Notifier wakes the UI thread                              |
 |                          v                                                           |
 |                 layered window, drawn with Direct2D                                  |
 +--------------------------------------------------------------------------------------+
          |  reads keys; writes pages.known      ^  WM_SETTINGS_CHANGED
          v                                      |  (posted to the FoldlineWindow class)
 +---------------------------------+             |
 | %APPDATA%\Foldline\settings.txt |             |
 +---------------------------------+             |
          ^  writes keys through store           |
          |                                      |
 +--------------------- foldline-settings.exe ---+----------+
 |  sections/  -->  store  -->  notify.rs                    |
 +-----------------------------------------------------------+
```

The two programs never call each other's code. The settings file and one window message are the whole link. The settings window writes a key through `crates/store` and posts `WM_SETTINGS_CHANGED` (`crates/settings/src/notify.rs`), and the notch reads the keys again. The notch writes `pages.known` at start so the settings window can list pages it doesn't link. The keys and defaults are in [settings keys](docs/reference/settings-keys.md#settings-file).

## Core components

| Component | Job | Detail |
| --- | --- | --- |
| `foldline.exe` (`crates/notch`) | `host/` owns the window, the monitor, opening and closing, the pin, the pause, the springs, input, the tray and the launcher of the settings window, and names no page. Pages and activities are each listed in one registry (`pages/mod.rs`, `activities/mod.rs`). `gfx/` and `ui/` draw with Direct2D and DirectWrite in a unit space that `host/notch/units.rs` converts to device pixels. | [host and pages](docs/explanation/host-and-pages.md), [drawing model](docs/explanation/drawing-model.md), [page API](docs/reference/page-api.md), [activities](docs/reference/activities.md) |
| `foldline-settings.exe` (`crates/settings`) | GPUI and gpui-component, one file per section, one window at a time. It writes keys and tells the notch. | [settings window](docs/reference/settings-window.md), [add a setting](docs/how-to/add-a-setting.md) |
| `store` (`crates/store`) | the settings file as `key=value` lines with per-key atomic writes, the data folder `%APPDATA%\Foldline`, and what both programs share (language, pause, snooze, countdown, screens) | [settings keys](docs/reference/settings-keys.md) |
| `website/` | the public site `fold.hibatillah.com`: the landing page (hero, feature grid, notch anatomy, performance and privacy numbers, extend guide, the ten pages with hover cards, FAQ), the download, releases, privacy and terms pages, in English and Indonesian. The notch drawings are web components in `components/notch/`, the page sections in `components/landing/`, and the text in `messages/`. | [website/README.md](website/README.md) |
| `places` (`crates/places`) | the city list behind the location search and the `location.*` keys that prayer times and weather share | [settings keys](docs/reference/settings-keys.md) |

`crates/notch` and `crates/settings` depend on `store` and `places` and never on each other, so the notch links no UI framework. Both programs look up their texts in English and Indonesian with `tr!` ([text in two languages](docs/explanation/host-and-pages.md#text-in-two-languages)).

## The website

`website/` is a separate program from the two above: Cargo ignores it, it has its own `bun.lock`, and neither Rust program reads or writes anything of it. It deploys to Vercel from the `website/` root directory, and a commit that touches only the Rust workspace does not redeploy it. The download button sends a visitor to the installer of the latest GitHub release (`website/app/api/download/route.ts`). The site's analytics (PostHog, cookieless, no session replay), its bug-report form, the share image and the SEO files are described in [website/README.md](website/README.md); the privacy page states what the site collects.

## Data stores

| Store | Where | Owner |
| --- | --- | --- |
| settings | `%APPDATA%\Foldline\settings.txt`, copied once from `%APPDATA%\notch` ([settings keys](docs/reference/settings-keys.md#settings-file)) | `crates/store` |
| todo list | `%APPDATA%\Foldline\todo.txt` | `pages/todo/list.rs` |
| shelf | `%APPDATA%\Foldline\shelf.txt` | `pages/shelf/` |
| calendar cache | `%APPDATA%\Foldline\calendar.ics` | `pages/calendar/` |
| `ShowBanner` values the notch replaced | `%APPDATA%\Foldline\notification-banners.txt`, put back after a crash | `host/notification_banners/` |
| clipboard history | memory only | `pages/clipboard/history.rs` |
| Start with Windows | the user's `Run` registry value (stable installer), or the packaged build's `FoldlineStartup` task (the MSIX is not released) | `crates/settings/src/startup/` |

What the user is told about each is in [README privacy](README.md#privacy).

## External integrations

Network requests use WinRT `Windows.Web.Http.HttpClient` on a worker thread. Nothing in either program contacts a Foldline server: there is no account or sign-in, and the only update check is the About page's button, which asks GitHub for the latest release on a click.

| Service | Used for | Code |
| --- | --- | --- |
| LRCLIB | synced lyrics | `pages/media/lyrics/client.rs` |
| the calendar address the user sets | the ICS download | `pages/calendar/fetch.rs` |
| MET Norway Locationforecast | the forecast and the rain alert | `pages/weather/fetch.rs` |
| weather.com | opened in the browser on a click, never fetched | `pages/weather/link.rs` |

The city search reads the bundled `assets/cities.tsv` and sends nothing. The device position comes from Windows (`platform/geolocation.rs`, `host/location_tracker/`).

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
| Downloads folder | `ReadDirectoryChangesW` on file names | `activities/downloads/watcher.rs` |
| volume | `IAudioEndpointVolume` | `activities/volume.rs` |
| devices | `DeviceWatcher` | `activities/device_battery/` |
| Caps Lock | `SetWindowsHookEx` keyboard hook | `activities/caps_lock.rs` |
| GPU load | PDH counters | `pages/stats/gpu.rs` |

## Build, test and release

- [website/README.md](website/README.md) for running, testing and deploying the website (`bun run dev`, `bun run lint`, `bun test`). `.github/workflows/website.yml` runs the type check, format check, lint and build on pull requests into `master` that touch `website/`.
- [Build, run and debug](docs/how-to/build-run-and-debug.md)
- [Test and snapshot](docs/how-to/test-and-snapshot.md), [test map](docs/reference/test-map.md)
- [Run the performance check](docs/how-to/run-the-perf-check.md)
- [CONTRIBUTING.md](CONTRIBUTING.md) for pull requests. `.github/workflows/ci.yml` runs on pull requests into `master`.
- [Cut a release](docs/how-to/cut-a-release.md). `.github/workflows/release.yml` runs on a pushed `v*` tag.

## Glossary

| Term | Meaning |
| --- | --- |
| collapsed | the slim pill while the notch is not open (`Mode::Resting`) |
| open | the full notch with its header and a page's content (`Mode::Expanded`) |
| pinned | open until the header pin is pressed again, also over maximized and full-screen windows. Never saved (`host/notch/pin.rs`) |
| paused | hidden completely, with Windows' own volume flyout and banners back, until a chosen time or until resumed. Stronger than the pin (`host/notch/pause.rs`, [settings keys](docs/reference/settings-keys.md#pausing-the-notch)) |
| host | `crates/notch/src/host/`: everything not specific to one page |
| page | a view that fills the collapsed pill, the open notch or both. Implements `Page` ([page API](docs/reference/page-api.md)) |
| activity | something that reports a change while the notch is collapsed. Implements `Activity` ([activities](docs/reference/activities.md)) |
| presence | what an activity or page shows while collapsed: a `Transient`, a `Badge` or a `Banner` (`activity.rs`) |
| idle pill | the collapsed pill when no page claims it: the time and the status marks (`host/pill_row.rs`) |
| claimed pill | the collapsed pill while a page fills it, as the media page does with a lyric line (`host/claimed_row.rs`) |
| status mark | an icon in the group at the right of the pill and the open header, ordered by `header_status_rank` |
| header | the top row of the open notch: page dots, time, status marks, the pin and the gear ([layout](docs/reference/layout.md)) |
| unit space | the device-independent coordinates in which pages, activities and widgets draw ([drawing model](docs/explanation/drawing-model.md)) |
| `Notifier` | how a worker thread wakes the UI thread |

## Where to go next

[ONBOARDING.md](ONBOARDING.md) for setup and rules, the [source map](docs/reference/source-map.md) for where a file lives, and [docs/README.md](docs/README.md) for every document by type.
