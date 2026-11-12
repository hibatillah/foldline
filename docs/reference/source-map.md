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
      i18n.rs         the `tr!` text lookup; the texts are in `crates/notch/locales/app.yml` (the settings window has its own `crates/settings/locales/app.yml`)
      host/           the notch itself: window, animation, opening and closing, the pin, tray, the launcher of the settings window. Knows no page.
      gfx/            drawing toolkit: Canvas and its unit space (`Canvas::in_units` in `canvas.rs`), fonts (a device set for the host and a unit set, `Fonts::unit`, in `fonts.rs`), the window's bitmap (`surface.rs`; `Surface::resize` changes its size and keeps the render target and uploaded images), colors, image pixels (`image_data.rs`) and their uploaded brushes (`image_brush.rs`), springs, geometry, text clamping, the bounded cache of measured text (`text_cache.rs`), SVG path data read into a `Path` (`svg_path.rs`), the whole-pixel square for cached shapes (`pixel_square.rs`, `Canvas::pixel_square`)
      platform/       small Windows actions for pages and activities: an app's icon, a shell bitmap's pixels (`bitmap.rs`), a window's class and its executable (`window_info.rs`), open a file, read and write the clipboard (`clipboard.rs`), paste into the window in front (`paste.rs`), drag a file
                      out, the picture Explorer shows, whether the machine has a battery (`power.rs`), each app's notification banner value and a watch on those values (`notification_settings.rs`), the Downloads folder (`downloads_folder.rs`), the icon Windows shows for a file extension (`file_icon.rs`), open a file or show it in Explorer (`shell_open.rs`), paired Bluetooth devices, their battery and the radio (`bluetooth_devices.rs`), one read of the device position (`geolocation.rs`), devices behind a 2.4 GHz USB receiver and a Logitech receiver's HID++ battery (`dongle_devices.rs`; `dongle_devices/` holds the tree walk, the HID channel, the receiver rules and the HID++ messages), opening a notification's app as a click on Windows' banner does (`toast_activation.rs`; `toast_activation/` holds the toast XML rules, the read of Windows' notification database, the toast activator lookup, the click's deadline and cap on stuck activator threads, and the foreground hook that tells whether the app came forward)
      ui/             reusable widgets: IconButton (with its hover wash, `draw_hover_wash`, also behind the header's pin and gear), PillButton, Ring, TextField (the todo page's add field), icons (`icons/`: the shapes in `paths.rs`; the gear and the pin included), line icons from SVG path data in several colors (`svg_icon.rs`), the dashed outline, the shared list scroll offset (`scroll.rs`), the one hovered thing on a page (`hover.rs`), the HUD layouts, the shared colors (`palette.rs`) and text styles (`type_styles.rs`), a banner's Dismiss and Open buttons (`banner_buttons.rs`, labels passed in), the input types `PointerEvent` and `KeyInput` (`input.rs`, re-exported by `page`)
      activities/     the registry (mod.rs) and one file or folder per activity: volume, brightness, battery, Bluetooth,
                      microphone, camera, clipboard, Caps Lock, notifications (`notifications/`: the card, the item and
                      its heading, the listener thread and its rounds of asking by id, `logos.rs` the icon cache that retries failures), browser downloads (`downloads/`: the tracker of partial files and the size text, both pure; `watcher.rs` the `ReadDirectoryChangesW` thread; the card)
      pages/
        mod.rs        the registry: the only place pages are listed
        media/        media player: session, controls, lyrics, artwork, app icon, the lyric glow
        shelf/        file shelf: the list of files (items.rs, with the thumbnail rule and tag colors), the drawn documents (document.rs), the empty row (empty.rs), the places (layout.rs), drawing (draw.rs), pointer input (input.rs), the example files for snapshots (sample.rs), drag in and out
        calendar/     calendar: dates, the iCal reader, the month grid and the day's agenda, the download, the month grid's text (`labels.rs`), when the page wakes and fetches (`wake.rs`), pointer input (`input.rs`), the collapsed badge (`badge.rs`)
        timer/        focus timer: the rules, what is said in each state, the layout, the page's tests (`tests.rs`)
        clipboard/    clipboard history: what a copy becomes (capture.rs, item.rs with the kind rules), the bounded list, the pins and undo (history.rs, tests in history/tests.rs), the bitmap reader and writer (dib.rs, no Windows calls), the preview lines (preview.rs), places and double click (layout.rs), clicks and pasting (actions.rs), drawing (draw.rs, style.rs), the row pictures held per copy and device size (thumbnails.rs), the page
        stats/        system stats: the readings (GPU load in gpu.rs), how they are written (readout.rs), the load charts and disk bars (charts.rs), card areas (layout.rs), the CPU, memory and network cards (cards.rs), the GPU and disk cards (rows.rs) and the known disk and GPU lists for the settings window (publish.rs)
        shalat/       prayer times: the calculation (times.rs), settings (config.rs), the open page (day.rs) and its sun arc (arc.rs)
        devices/      connected devices: the list and its rules (model.rs), Bluetooth plus receiver devices and the messages (merge.rs), the worker that reads Windows (source.rs), tiles (layout.rs, draw.rs, ring.rs)
        weather/      weather: the MET Norway request (fetch.rs), the forecast (forecast.rs), the sky icons (picture.rs), the message states and their width (message.rs), the page's drawing and the Today/Week pill (draw.rs), the rain alert (alert.rs: the banner, badge.rs: the countdown badge), the weather.com addresses (link.rs) the clickable cells (click.rs), fetch and rain-alert scheduling (refresh.rs), pointer input (input.rs) and the snapshot samples (sample.rs)
        todo/         todo list: the list, its todo.txt file and the undo of the last removal (list.rs), which rows show and where (layout.rs), presses: add, clear, undo, copy, delete, tick (actions.rs), the green check after a copy (copy_check.rs), drawing (draw.rs), the snapshot scenarios (sample.rs), the page tests (tests.rs), the page
  places/                the city list and the shared `location.*` keys, used by the notch and the settings window
  store/                 the settings file code shared by the notch and the settings window (per-key atomic writes) `language.rs`, the language both programs read, and `pause.rs`, the `notch.paused_until` value (its parse, text, end time and clock text) that the notch, the tray and the settings window share
  settings/              the settings window, a separate GPUI executable named notch-settings.exe (needs the MSVC host
                         toolchain); src/sections/ holds one file per section (shalat/ one file per card group, about/ the
                         About page: app card, links, licenses file, data paths; pause.rs the Pause item of General; sections/mod.rs lists them and writes
                         every key through `apply`); main.rs the window (frame, brand strip, sidebar and content card,
                         in the system's light or dark theme), field.rs the switch and the text and number fields bound
                         to a key and their save rules, notify.rs posts WM_SETTINGS_CHANGED to the notch,
                         single_instance.rs allows one window and focuses the open one, startup.rs the Run registry
                         value behind Start with Windows, palette.rs the colors (dark and light, with contrast tests), sidebar.rs the groups and icons, header.rs the breadcrumb header and window
                         buttons, icons.rs the bundled SVG icons (assets/icons) and their asset source, layout.rs the page
                         description, bordered sections and rows, location.rs the city search, card.rs and picture.rs the
                         picture cards
vendor/salah/     the prayer-time crate with one crash fix (see docs/reference/dependencies-and-licenses.md)
installer/        the Inno Setup script the release workflow builds the installer from
assets/fonts/     Geist, the bundled font, and its license
```

## Host files


`host/` owns everything that is not specific to a page.

| File | Job |
| --- | --- |
| `window.rs` | creates the window, runs the message loop, turns Windows messages into calls on the notch; `window/focus.rs` keeps the window topmost and hands the keyboard to the notch and back |
| `notch.rs` + `notch/` | the `Notch`; `notch.rs` holds the struct and the timer ids, one file per job in `notch/` (see [The notch's files](#the-notchs-files)) |
| `registry.rs` | decides which registered pages are switched on (`NOTCH_PAGES`, the `pages` setting) |
| `navigation.rs` | the user's page order and which pages are on, which page opens, the spring of the dots' long mark (pure) |
| `page_swap.rs` | the page swap in the open notch: when the leaving page fades out and how the arriving one fades and shifts in (pure) |
| `scroll_gate.rs` | when scrolling switches a page: one per wheel notch or gesture, with a cooldown (pure) |
| `motion.rs` | every tuning number of the size springs, the page swap and scroll switching |
| `drop_target.rs` | a second thread with an almost invisible window that takes dragged files (OLE needs a single-threaded apartment) and tells the notch to open |
| `chrome.rs` | the padding and the page dots (they start at the header's left edge) around every open page |
| `pill_row.rs` | where the idle row's items and the open header's items (the pin and the gear included) sit, the gaps, the empty pill and the hit areas: `idle_row`, `header_row`, `hit_item` (pure) |
| `clock.rs` | the time: in the header of the open notch, and in the collapsed pill's idle row, and its width |
| `screen_mode.rs` | whether the window in front is maximized or full screen, from Windows' event hooks (no polling) |
| `screen_mode/classify.rs` | the screen mode the facts about the window in front give: full screen, maximized, a screenshot overlay (pure) |
| `screen_mode/location_hook.rs` | the hook on the foreground window's thread; pauses it for 250 ms when caret or pointer moves come less than 100 ms apart |
| `visibility.rs` | what the notch shows over a maximized or full-screen window: the four words, `suppresses`, `hides_to_strip`, the priority exception (`priority_shows`) and the one-time migration of the retired keys (pure) |
| `startup.rs` | the "Start with Windows" entry under the user's `Run` key |
| `activity_layout.rs` | which transient activities show and where each one starts (pure functions) |
| `claimed_row.rs` | the collapsed pill while a page claims it: lead, dots, content, group (pure) |
| `settings.rs` | re-exports `Settings` from the `store` crate, the persistent key-value store pages and activities share |
| `tray.rs` | the tray icon and its menu: Start with Windows (checked when on), Settings, Pause Notch (a submenu, or the status line and Resume Notch while paused), Quit Notch; the notch has no menu of its own |
| `settings_launcher.rs` | starts `notch-settings.exe` from the folder of the running `notch.exe`, without waiting; the gear in the header and the tray's Settings item both call it |
| `display.rs` | display scale and refresh rate |
| `monitor.rs` | the monitors (`list`), the foreground window's monitor, the `notch.monitor` word and the start placement (Windows calls) |
| `monitor/choose.rs` | which monitor the notch sits on (`target_monitor`) and the window's position and size there (`placement`) (pure) |
| `location_tracker.rs` | keeps the saved city where the device is (`location.track`): a worker thread that reads Windows' position once, the answer's message, and what an answer changes (pure `update_for`); `location_tracker/network.rs` registers a callback on network interface changes while tracking is on |
| `volume_flyout.rs` | keeps Windows' volume and brightness flyout from drawing by giving its window an empty region, found by a show hook on the flyout's thread (`notch.hide_windows_volume_popup`); `volume_flyout/identity.rs` tells the flyout from Explorer's other island windows (pure); `volume_flyout/hook_thread.rs` runs the hook on a thread of its own |
| `notification_banners.rs` | turns off Windows' notification banner for every app while the notch runs and puts the values back (`notch.hide_windows_notifications`), from a thread that waits on registry changes; `notification_banners/backup.rs` is the saved values and what a pass changes (pure), `notification_banners/passes.rs` reads and writes them |

### The notch's files

`Notch` is one struct whose methods live in one file per job under `host/notch/`. Each job that keeps its own state
owns a small struct for it, defined beside its logic and held by `Notch` (`springs`, `row_motion`, `open`, `hiding`, `keyboard`,
`resting_row`, `presences`).

| File | Job | State it owns |
| --- | --- | --- |
| `notch.rs` | the `Notch` struct, `new`, `start`, the timer ids and the constants several files share | `PageSwap` (from `host/page_swap.rs`) |
| `input.rs` | pointer, wheel and touchpad scroll, a page's right click and drag-and-drop input | |
| `timers.rs` | what each timer does when it fires | |
| `pages.rs` | page and activity lifecycle, choosing and moving between pages | |
| `layout.rs` | page frames, the pages a swap draws, starting a swap, refreshing sizes and which page shows | |
| `presences.rs` | collecting what activities and pages show in the collapsed pill: transients, the banner, badges, the ring and the dots | `ShownPresences` |
| `open_state.rs` | opening and closing (the open state), the delays, the hover watch | `OpenState` |
| `pin.rs` | the pin: pinning opens and holds the notch (no hover watch, a raise over a maximized or full-screen window), unpinning starts the usual close | |
| `hiding.rs` | what shows over a maximized or full-screen window, suppression, hide when idle | `HidingState` |
| `pause.rs` | pausing the notch: the stored `notch.paused_until`, the end timer, giving Windows' popups back and hiding them again | `PauseState` |
| `animation.rs` | the springs, the frame the springs describe, the animation timer | `Springs`, `FrameTimer` |
| `keyboard.rs` | typing: who owns the keyboard, text keys, pasted text, the keyboard lost to another window | `KeyboardState` |
| `settings_reload.rs` | applying a changed settings file after `WM_SETTINGS_CHANGED` | |
| `location.rs` | following `location.track`: start and stop watching, the 10 s wait after a network change, saving the city a position gives | |
| `resting_row.rs` | the idle row and the claimed pill's row: the group, the battery ring, the dot column, short badges, their drawing | `RestingRow` |
| `row_motion.rs` | one spring per row item and for the clocks, so items slide when the row changes | `RowMotion` |
| `header_row.rs` | the open header: the pin, the gear, the icons, the group, what a click on them does, the hover wash on the pin and the gear (`HeaderMarks::hovered`) | |
| `units.rs` | opening the device-independent unit space around every page and activity draw and pointer hook (all but `draw_backdrop`), and the conversion of frames, areas and points | |
| `paint.rs` | drawing the pill, the header, the banner, the badges and the shadow | |
| `relocate.rs` | moving the notch to the monitor `notch.monitor` picks: when a move waits, the one-message scale change (`Surface::resize`, new fonts, rebuilt cached text), the window move | |
| `monitor_state.rs` | what the notch knows about its monitor: the mode, the current monitor, the last placement, a move that waits, and the seam that lets tests inject monitors | `MonitorState` |
| `tests/` | behavior tests (`hiding_rules.rs`, `screen_mode.rs`, `relocate.rs`, `settings_reload.rs`, `page_navigation.rs`, `page_states.rs`, `clipboard_states.rs`, `resting_row.rs`, `row_motion.rs`, `header_row.rs`, `header_hover.rs`, `pin.rs`, `pause.rs`) and the pictures (`images.rs`) on an off-screen `Notch` built in `tests/mod.rs` | |

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
activities use `activity`, `page` (for `Context`), `gfx`, `ui` and `platform`. The host uses `page`, `activity`, `gfx`
and `ui` (for icons). Nothing in `gfx`, `ui`, `page` or `activity` knows about a particular page or activity, and pages
and activities never use each other. `gfx` and `ui` import nothing from `page`, `i18n`, `host`, `pages`, `activity` or
`platform`; the test `the_drawing_layers_import_nothing_above_them` in `ui/mod.rs` enforces it, and the reason is in
[the drawing model](../explanation/drawing-model.md#the-three-layers).
