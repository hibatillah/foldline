# Source map

> **Reference.** Read when: you look for where something lives, or which module may use which. Skip if: you know the file; read the file.

```
src/
  main.rs         entry point: starts the host with the registered pages and activities
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
    shalat/       prayer times: the calculation (times.rs), the city list for the search (cities.rs), the page
    todo/         todo list: the list and its todo.txt file (list.rs), the page and its add field
store/            notch-store: the settings file code shared by the notch and the settings window (per-key atomic writes)
settings/         notch-settings: the settings window, a separate GPUI executable (needs the MSVC host toolchain);
                  sections/ holds one file per section
vendor/salah/     the prayer-time crate with one crash fix (see docs/reference/dependencies-and-licenses.md)
installer/        the Inno Setup script the release workflow builds the installer from
assets/fonts/     Geist, the bundled font, and its license
```

Dependencies point one way. Pages use `page`, `activity` (for `Presence` and `Flash`), `gfx`, `ui` and `platform`;
activities use `activity`, `page` (for `Context`), `gfx`, `ui` and `platform`. The host uses `page`, `activity` and `gfx`. Nothing in
`gfx`, `ui`, `page` or `activity` knows about a particular page or activity, and pages and activities never use each
other.
