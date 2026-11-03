# Add an activity

> **How-to.** Read when: you are adding something that shows in the collapsed pill: a volume change, a plug, a device turning on. Skip if: it belongs in the open notch; that is a page ([tutorial](../tutorials/add-a-page.md)).

An activity is something that happens while the notch is closed: a volume change, a charger being plugged in, the
microphone turning on. A page is what you see when you open the notch; an activity is what you see in the collapsed
pill. Activities are required unless their implementation opts into a user toggle; see [switch off or remove](switch-off-or-remove.md).

What an activity can show, and the built-in ones, are in [the activities reference](../reference/activities.md).

## Steps

### 1. Create the file

Make `src/activities/<name>.rs` (or a folder when it has several parts, like `notifications/`). An activity depends on
`activity`, `page` (for `Context`), `gfx`, `ui` and `platform`, never on a page or another activity.

### 2. Implement `Activity`

Four methods are required; the rest are hooks. `src/activities/brightness.rs` is a complete small activity: it
listens to one Windows power setting and flashes a level bar when it changes.

| Method | Job |
| --- | --- |
| `id()` | stable name; the settings key is `activity.<id>` |
| `label()` | the name in the tray menu for an optional activity |
| `is_optional()` | false by default: required activities are always enabled; return true to expose a saved tray toggle |
| `presence()` | what to show right now |
| `draw()` | draw into the area the host gives |
| `header_status_width()`, `draw_header_status()` | optional: a small mark kept in the open notch's header beside the time while something lasts (muted speakers, Caps Lock on); `None` for no mark |
| `on_start()` | begin listening; called once, and only if the activity is switched on |
| `on_timer()`, `on_notify()` | a timer fired; a background thread used the `Notifier` |
| `on_hover_changed()` | the pointer entered or left the notch |
| `on_system_message()` | a power setting or the clipboard changed (see below) |
| `on_settings_changed()` | on a started activity, after the notch received `WM_SETTINGS_CHANGED` and re-read `settings.txt`; re-read the activity's own keys, falling back to the current value |

Use `Flash` for a transient that shows and then goes: `flash.show(context, milliseconds)` shows it, again while
visible keeps it up for a fresh duration, and `flash.on_timer(timer, context)` hides it. It uses `TimerId(0)`; an
activity that needs more timers starts from `TimerId(1)`. Timer ids and wake-up messages are separate from the pages',
so an activity and a page never collide.

Draw messages with `ui::draw_level` (icon, bar, value) or `ui::draw_message` (icon, text, optional value) so every
transient looks alike, and add an `Icon` to `src/ui/icons.rs` when you need a new glyph.

### 3. Get events without polling

The notch must use no CPU while nothing happens, so an activity never polls. Pick the source that calls you:

| Source | How |
| --- | --- |
| a Windows power setting (brightness, charger, battery level) | `context.listen_to_power_setting(&GUID)` in `on_start`; changes arrive in `on_system_message`. The first message after registering is the current value, so use `is_change` to skip it. |
| the clipboard | `context.listen_to_clipboard()`; arrives as `SystemMessage::ClipboardChanged` |
| a COM callback (Core Audio) | store the change in an `Arc<Mutex<..>>` and call `context.notifier().notify()`; read it in `on_notify` |
| a Windows event log channel | `EvtSubscribe` with a callback, which runs on a system thread and calls the `Notifier`; see `camera.rs` |
| one key | a low-level keyboard hook on a thread of its own that wakes the activity for that key only, then read the state from Windows; see `caps_lock.rs` |
| a folder changing | `FindFirstChangeNotificationW` and a blocking wait on a thread of its own; see `notifications/listener.rs` |

### 4. Register it

Add the module and an entry to `all()` in `src/activities/mod.rs`. Registration order controls badges and optional
tray entries; filtering the tray must preserve the registered activity indices.
