# Settings window

> **Reference.** Read when: you need what each section of `notch-settings.exe` sets, or how it saves. Skip if: you want settings inside the notch; they come from each page's `settings()` ([page API](page-api.md)).

Open it from the tray menu's **Settings** item. The tray starts `notch-settings.exe` from the folder of the running
`notch.exe` without waiting for it (`crates/notch/src/host/settings_launcher.rs`) and logs `could not start settings` when the program
is missing. The window allows one instance: starting it again focuses the open one.

| Section | What you set | Key |
| --- | --- | --- |
| General | Startup: start the notch at sign-in | the `Run` registry value, see [Start with Windows](settings-keys.md#start-with-windows) |
| General | Startup: hide the notch when nothing is playing or happening | `notch.hide_when_idle` |
| General | Location: the city, searched by name; up to five matches are listed and the pick is saved | `location.city`, `location.lat`, `location.lon`, `location.country` |
| General | Notifications: show Windows notification banners; show what a notification says | `activity.notifications`, `notifications.show_text` |
| General | Music: show lyrics | `media.lyrics` |
| General | Timer: focus, short break and long break lengths | `timer.focus_minutes`, `timer.short_break_minutes`, `timer.long_break_minutes` |
| Pages | the order of the pages, and which are on; drag a page to reorder it | `navigation.order`, `navigation.disabled` |
| Behavior | what the notch shows over a maximized window and over a full-screen app, and whether pointing at it opens it there | `notch.maximized`, `notch.fullscreen`, `notch.hover_maximized`, `notch.hover_fullscreen` |
| Calendar | the calendar address and how often it is downloaded again; the section lists the steps for finding a Google Calendar address | `calendar.ics_url`, `calendar.refresh_minutes` |
| Shalat | a note that the location is set in General (the page has no city field); the calculation method as picture cards, Auto and six methods, each showing how far below the horizon the sun is at Fajr and at Isha; Standard or Hanafi Asr, drawn as a stick with its shadow; the two reminders with a preview of each; minus 30 to plus 30 minutes for each prayer, with a day bar that shows where each prayer falls | `shalat.method`, `shalat.hanafi`, `shalat.message`, `shalat.badge`, `shalat.offset_fajr`, `_sunrise`, `_dhuhr`, `_asr`, `_maghrib`, `_isha` |
| Advanced | the debug log file; how far the notch moves down from the top edge. Both apply after a restart of the notch | `advanced.debug_log`, `notch.top` |

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

The window opens at 1140 x 720 and resizes down to 720 x 480. The sidebar's bottom **Licenses** item opens `THIRD-PARTY-LICENSES.txt`, looked for in the
folder of `notch-settings.exe` and then in up to six parent folders (so `run/` and `target/.../release` find the repository-root
file; the installer puts it beside the program, and `scripts/reload.sh` copies it into `run/`). When it is not found, a warning
notification in the window's top right corner says so. The
text is Geist, the notch's bundled font.

The window's memory and CPU figures are in [performance.md](performance-baseline.md#settings-window). To add a setting or a
section, see [adding-a-setting.md](../how-to/add-a-setting.md).

The window's layers, section titles and option cards follow the "Settings window" rules in [design-rules.md](design-rules.md#settings-window).
