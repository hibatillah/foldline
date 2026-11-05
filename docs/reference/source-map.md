# Source map

> **Reference.** Read when: you look for where something lives, or which module may use which. Skip if: you know the file; read the file.

```
Cargo.toml               the workspace: members, shared version, profiles, the salah patch
crates/
  notch/                 the pill, notch.exe
    build.rs             embeds the icon
    examples/            perf.rs (the performance check) and tray_spike.rs
    src/
      page.rs         the whole contract for a page: the Page trait, Context, Frame, events
      activity.rs     the whole contract for an activity: the Activity trait, Presence, SystemMessage, Flash
      log.rs          optional debug log (set NOTCH_DEBUG to a file path)
      host/           the notch itself: window, animation, hover, tray, settings. Knows no page.
      gfx/            drawing toolkit: Canvas, fonts, colors, images, springs, geometry, text clamping
      platform/       small Windows actions for pages and activities: an app's icon, open a file, paste text, drag a file
                      out, the picture Explorer shows
      ui/             reusable widgets: IconButton, PillButton, Ring, TextField, icons, the dashed outline, the HUD layouts
      activities/     the registry (mod.rs) and one file or folder per activity: volume, brightness, battery, Bluetooth,
                      microphone, camera, clipboard, Caps Lock, notifications
      pages/
        mod.rs        the registry: the only place pages are listed
        media/        media player: session, controls, lyrics, artwork, app icon, the lyric glow
        shelf/        file shelf: the list of files, their pictures, drag in and out
        calendar/     calendar: dates, the iCal reader, the day's agenda, the download
        timer/        focus timer: the rules, what is said in each state, the layout
        stats/        system stats: the readings (GPU load in gpu.rs) and how they are written
        shalat/       prayer times: the calculation (times.rs), settings (config.rs), the page
        weather/      weather: the MET Norway request (fetch.rs), the forecast (forecast.rs), the pictures (picture.rs)
        todo/         todo list: the list and its todo.txt file (list.rs), the page and its add field
  places/                the city list and the shared `location.*` keys, used by the notch (and the settings window later)
  store/                 the settings file code shared by the notch and the settings window (per-key atomic writes)
  settings/              the settings window, a separate GPUI executable named notch-settings.exe (needs the MSVC host
                         toolchain); src/sections/ holds one file per section
vendor/salah/     the prayer-time crate with one crash fix (see docs/reference/dependencies-and-licenses.md)
installer/        the Inno Setup script the release workflow builds the installer from
assets/fonts/     Geist, the bundled font, and its license
```

## Host files


`host/` owns everything that is not specific to a page.

| File | Job |
| --- | --- |
| `window.rs` | creates the window, runs the message loop, turns Windows messages into calls on the notch |
| `notch.rs` + `notch/` | the `Notch`; `notch.rs` holds the struct and the timer ids, one file per job in `notch/` (see [The notch's files](#the-notchs-files)) |
| `registry.rs` | decides which registered pages are switched on (`NOTCH_PAGES`, the `pages` setting) |
| `navigation.rs` | the user's page order and which pages are on, which page opens, wheel and swipe rules (pure) |
| `drop_target.rs` | a second thread with an almost invisible window that takes dragged files (OLE needs a single-threaded apartment) and tells the notch to open |
| `chrome.rs` | the padding and the header (page dots, status marks and the time) around every open page |
| `clock.rs` | the time: in the header of the open notch, and in the collapsed pill while no page or activity has anything to show |
| `screen_mode.rs` | whether the window in front is maximized or full screen, from Windows' event hooks (no polling) |
| `visibility.rs` | what the notch shows over a maximized or full-screen window: the four words, `allows`, and the one-time migration of the retired keys (pure) |
| `startup.rs` | the "Start with Windows" entry under the user's `Run` key |
| `page_editor.rs` | the Pages tab of the settings panel: chip layout, drag and double-click rules, drawing |
| `settings_panel.rs` + `settings_panel/` | the settings panel inside the notch: tabs and their controls (`controls.rs`), where they sit (`layout.rs`), the value a press saves (`values.rs`), drawing (`draw.rs`), sizes and colors (`style.rs`) |
| `activity_layout.rs` | which transient activities show and where each one starts (pure functions) |
| `settings.rs` | re-exports `Settings` from the `store` crate, the persistent key-value store pages and activities share |
| `tray.rs` | the tray icon and its menu: a checkable item per activity, the pages' items, the notch's own options, Settings…, Quit |
| `settings_launcher.rs` | starts `notch-settings.exe` from the folder of the running `notch.exe`, without waiting |
| `display.rs` | display scale and refresh rate |

### The notch's files

`Notch` is one struct whose methods live in one file per job under `host/notch/`. Each job that keeps its own state
owns a small struct for it, defined beside its logic and held by `Notch` (`springs`, `hover`, `hiding`, `keyboard`,
`panel`).

| File | Job | State it owns |
| --- | --- | --- |
| `notch.rs` | the `Notch` struct, `new`, `start`, the timer ids and the constants several files share | |
| `input.rs` | pointer, wheel, swipe, right click and drag-and-drop input | |
| `timers.rs` | what each timer does when it fires | |
| `pages.rs` | page and activity lifecycle, the tray menus, choosing and moving between pages | |
| `layout.rs` | page frames, blending between two pages, refreshing sizes and which page shows | |
| `hover.rs` | opening and closing, the delays, the hover watch | `HoverState` |
| `hiding.rs` | what shows over a maximized or full-screen window, suppression, hide when idle | `HidingState` |
| `animation.rs` | the springs, the frame the springs describe, the animation timer | `Springs` |
| `keyboard.rs` | typing: who owns the keyboard, text keys, pasted text | `KeyboardState` |
| `settings_ui.rs` | the settings panel and the Pages editor inside the notch, applying a changed settings file | `PanelState` |
| `paint.rs` | drawing the pill, the header, the banner, the badges and the shadow | |
| `tests/` | behavior tests (`hiding_rules.rs`, `screen_mode.rs`, `settings_reload.rs`, `page_navigation.rs`) and the pictures (`images.rs`) on an off-screen `Notch` built in `tests/mod.rs` | |

## How a module is laid out

The rule file `.claude/rules/module-layout.md` repeats this in short for agents.

Every module follows one shape, so a file's place tells you its job.

- `mod.rs` (or the `name.rs` beside a `name/` folder) wires: it declares the child modules, holds the main type and the
  `Page` or `Activity` impl, and keeps no logic a child could hold.
- One job per file, named for the job (`layout.rs`, `draw.rs`, `storage.rs`, `text.rs`). A file reaches about 400 lines
  before it is split, and 500 is the limit.
- Logic that takes and returns plain values (layout math, rules, wording, parsing) never imports `windows::`, so a
  test runs it without a window. Windows calls stay in the host, `platform` and `ui`.
- State a job owns lives in a small struct beside that job's logic.
- Tests sit with the code they check; a module with many tests moves them to `tests.rs` or `tests/`.

Dependencies point one way. Pages use `page`, `activity` (for `Presence` and `Flash`), `gfx`, `ui` and `platform`;
activities use `activity`, `page` (for `Context`), `gfx`, `ui` and `platform`. The host uses `page`, `activity` and `gfx`. Nothing in
`gfx`, `ui`, `page` or `activity` knows about a particular page or activity, and pages and activities never use each
other.
