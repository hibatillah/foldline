# Add an activity

> **How-to.** Read when: you are adding something that shows in the collapsed pill: a volume change, a plug, a device turning on. Skip if: it belongs in the open notch; that is a page ([add a page](add-a-page.md)).

An activity is something that happens while the notch is closed: a volume change, a charger being plugged in, the
microphone turning on. A page is what you see when you open the notch; an activity is what you see in the collapsed
pill. Activities are required unless their implementation opts into a user toggle; see [switch off or remove](switch-off-or-remove.md).

What an activity can show, and the built-in ones, are in [the activities reference](../reference/activities.md).

## Steps

### 1. Create the file

Make `crates/notch/src/activities/<name>.rs` (or a folder when it has several parts, like `notifications/`). An activity depends on
`activity`, `page` (for `Context`), `gfx`, `ui` and `platform`, never on a page or another activity.

### 2. Implement `Activity`

Three methods are required (`id`, `presence`, `draw`); the rest are hooks. `crates/notch/src/activities/brightness.rs` is a complete small activity: it
listens to one Windows power setting and flashes a level bar when it changes.

| Method | Job |
| --- | --- |
| `id()` | stable name; the settings key is `activity.<id>` |
| `is_optional()` | false by default: required activities are always enabled; return true to expose a saved settings toggle |
| `is_priority()` | false by default: true shows the transient level (not a badge) over a window that hides the notch (any `notch.maximized` or `notch.fullscreen` word, Hide when idle), only while `notch.hide_windows_volume_popup` is on; volume and brightness answer true |
| `presence()` | what to show right now |
| `is_notice(presence)` | whether `presence`, what `presence()` just returned (the host asks it once per refresh), is a notice a snooze (`notch.snoozed_until`) drops: a message or a banner. Default: true for a `Banner`, false otherwise. A message answers true while it shows (battery, Bluetooth, device battery, "Copied"); an icon-only indicator keeps false, even its own short message (volume, brightness, Caps Lock, camera, microphone). The host drops a notice in one place and dismisses a dropped banner through `dismiss_banner()` |
| `draw()` | draw into the area the host gives, in device-independent units from zero on a canvas whose `scale()` is 1; a companion's `draw` also serves the open header (see below) |
| `header_status_width()`, `draw_header_status()`, `header_status_rank()` | optional: a small mark kept at the right edge of the header, icon only, while something lasts (muted speakers, Caps Lock on); `None` for no mark. The rank orders the group in the pill and the header |
| `header_status_page()` | optional: the id of the page a click on the header mark opens; `None` (the default) opens nothing, as for muted and Caps Lock |
| `on_start()` | begin listening; called once, and only if the activity is switched on |
| `on_timer()`, `on_notify()` | a timer fired; a background thread used the `Notifier` |
| `on_open_changed()` | the notch opened or closed (by the pointer, a page or the pin) |
| `dismiss_banner()` | the host calls it on an activity that shows a `Banner` while a priority activity (volume, brightness; a transient, not the muted badge) shows a level: hide the banner now, as if it had timed out, and do not bring it back; keep the data behind it (a dismissed notification stays in the Action Center). Default: nothing |
| `on_pointer()` | pointer input over this activity's banner while the notch is collapsed, in the unit `area` `draw()` got, with points in units from its top-left; `PointerEvent::Left` means the pointer left the banner. `ui::BannerButtons` draws and hit-tests the Dismiss and Open pair (the notification card) |
| `on_key()`, `on_keyboard_lost()` | typing, after `context.take_keyboard()` from a click (the notification banner's reply field); the keyboard went elsewhere |
| `on_system_message()` | a power setting or the clipboard changed (see below) |
| `on_language_changed()` | on a started activity, once per change of `general.language` and after the notch moves to a monitor of another scale; rebuild any message or width cached from `tr!` |
| `on_settings_changed()` | on a started activity, after the notch received `WM_SETTINGS_CHANGED` and re-read `settings.txt`; re-read the activity's own keys, falling back to the current value |


Use `Flash` for a transient that shows and then goes: `flash.show(context, milliseconds)` shows it, again while
visible keeps it up for a fresh duration, and `flash.on_timer(timer, context)` hides it. It uses `TimerId(0)`; an
activity that needs more timers starts from `TimerId(1)`. Timer ids and wake-up messages are separate from the pages',
so an activity and a page never collide.

A mark that lasts at the right end of the group (the battery ring, a camera or microphone dot) answers
`Presence::Companion { slot, width }` instead of a transient or a badge. `Slot::Ring` puts it before the dots (one
item; the battery activity answers it only on a machine with a battery); `Slot::Dots` puts it in the dot column at the very end. Neither is ranked, and both sit right of every badge. Three rules follow from how the host uses it:

- The same `draw` paints the companion in the collapsed pill and in the open header, in areas of different heights, so draw
  centered on `area` (`area.center_x()`, `area.center_y()`) and never from its top or bottom edge. The ring draws its
  diameter around the center; a dot draws in a row offset from the center (`DotRow` in `activities/privacy.rs`).
- Every `Slot::Dots` companion draws into one shared area as wide as the widest of them, so give each its own
  row: camera on top, microphone below. `Slot::Ring` holds one item. The dot column reserves no width, so a dot that
  appears or leaves slides the group.
- A companion has no timer. It repaints on the events its activity already handles, so `presence()` is the only place its
  state shows.

Only activities place companions; a page's `Companion` is ignored. While a transient shows, the activity answers the
transient instead, and the companion returns after it.

A state that lasts and also announces itself, like muted speakers or Caps Lock, uses a badge and a transient together:
`presence()` answers `Presence::Transient` while the `Flash` is visible and `Presence::Badge { width }` while the state is on.
`draw` paints the message while the flash shows and the icon alone otherwise (`volume.rs`, `caps_lock.rs`), and
`header_status_width` and `draw_header_status` give the same icon to the open header. Set `header_status_rank` to place
it in the group: lower ranks sit nearer the clock, in the pill and in the header (music wave 0, Caps Lock 1, muted 2, timer 3, event 4, prayer 5, rain 6, downloads 7; the default is 0).

Draw messages with `ui::draw_level` (icon, bar, value) or `ui::draw_message` (icon, text, optional value) so every
transient looks alike, and add an `Icon` to `crates/notch/src/ui/icons/` (the enum in `mod.rs`, the shape in `paths.rs`) when you need a new glyph.

### 3. Get events without polling

The notch must use no CPU while nothing happens, so an activity never polls. Pick the source that calls you:

| Source | How |
| --- | --- |
| a Windows power setting (brightness, charger, battery level) | `context.listen_to_power_setting(&GUID)` in `on_start`; changes arrive in `on_system_message`. The first message after registering is the current value, so use `is_change` to skip it. |
| the clipboard | `context.listen_to_clipboard()`; arrives as `SystemMessage::ClipboardChanged` |
| a COM callback (Core Audio) | store the change in an `Arc<Mutex<..>>` and call `context.notifier().notify()`; read it in `on_notify` |
| a Windows event log channel | `EvtSubscribe` with a callback, which runs on a system thread and calls the `Notifier`; see `camera.rs` |
| one key | a low-level keyboard hook on a thread of its own that wakes the activity for that key only, then read the state from Windows; see `caps_lock.rs` |
| a Windows device watcher | a WinRT `DeviceWatcher` on a thread of its own, which Windows pushes property changes into; it calls the `Notifier`; see `device_battery/` |
| a Windows event a WinRT API raises | subscribe to it and call the `Notifier` from the handler; see `notifications/listener.rs` (`NotificationChanged`) |
| a folder changing | `FindFirstChangeNotificationW` and a blocking wait on a thread of its own; see `notifications/listener.rs` (the fallback when the event cannot be subscribed) |
| a folder's file names changing | `ReadDirectoryChangesW` on a thread of its own, which calls the `Notifier`; see `downloads/watcher.rs` |

### 4. Register it

Add the module and an entry to `all()` in `crates/notch/src/activities/mod.rs`. Registration order breaks ties between badges of equal `header_status_rank`.
