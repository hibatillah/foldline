# Activities

> **Reference.** Read when: you need what an activity can show (`Presence`), or what each built-in activity watches. Skip if: you are writing one; the steps are in [add an activity](../how-to/add-an-activity.md).

## What an activity can show

An activity answers `presence()` after every event, and the host does the layout:

| Presence | Looks like | Example |
| --- | --- | --- |
| `Transient { width }` | takes over the pill for a moment, then goes | volume bar, "Copied", "Charging 56%" |
| `Badge { width }` | a small mark in the group at the right edge for as long as something lasts, ordered by `header_status_rank` | the timer ring, the prayer or event countdown, the muted speaker, the Caps Lock icon, the music wave |
| `Companion { slot, width }` | a mark at the fixed right end of the group for as long as something lasts: `Slot::Ring` is the battery ring, the item before the dots, `Slot::Dots` the dot column at the very end | the battery ring (`Ring`), the camera and microphone dots (`Dots`) |
| `Banner { width, height }` | opens the pill into a card, taller than the idle pill and smaller than the open notch, and draws into all of it; it has the pill to itself, so transients wait. Pointing at the notch does not open it while a banner shows; the pointer over the banner arrives in `on_pointer` (`on_presence_pointer` for a page), so the activity can hold it, answer a taller `presence()` with buttons, and take their clicks | a notification, the rain alert |
| `None` | nothing | |

- Up to two transients show at once, side by side with a thin divider; the newest two win.
- While a transient shows, the page's collapsed content fades out and the pill resizes to the transient. Badges are
  hidden then and come back after.
- Badges shrink the room the page draws in, so the page never sits under a badge or the dots. Where each item sits
  is in [layout.md](layout.md).
- Ranks order the group in the pill and the icons in the open header; the lowest is nearest the clock, and equal ranks
  keep list order. `Activity::header_status_rank` (default 0) and `Page::header_status_rank` share one scale: music wave 0,
  Caps Lock 1, muted 2, timer 3, event 4, prayer 5.
- A state that lasts and also announces itself (muted, Caps Lock) answers a `Badge` while the state is on and a `Transient`
  while its message shows; the badge is an icon alone, drawn by `draw` and `draw_header_status` alike.
- Companions are drawn by `draw` in the collapsed pill and in the open header, so a companion centers its drawing on the
  area it gets. A `Ring` companion is one item; all `Dots` companions share one area as wide as the widest, and each
  picks its own row. The dot column reserves no width: with no dot the rest of the group fills its place, and a dot
  appearing slides the group left. Companions are not ranked and always sit right of the badges. Only activities place companions.
- Opening the notch fades every activity out, as it does the page's collapsed content.
- `width` is in device-independent pixels. `draw` gets the exact area for it, in device pixels.

## Built-in activities

| Id | Shows | Source |
| --- | --- | --- |
| `volume` | volume bar, or "Muted"; a crossed speaker (rank 2) in the pill group and the header while muted | Core Audio endpoint volume callback; follows the default speakers; the mute state is read when listening starts |
| `brightness` | brightness bar | power setting `GUID_DEVICE_POWER_POLICY_VIDEO_BRIGHTNESS`; laptop panel only |
| `battery` | "Charging", "On battery", "Low battery" at 20 % and 10 %; between messages a 14 px ring at the right end of the group, before the dots (`Slot::Ring`): the arc is the charge, `#2f9e44` idle, `#30d158` while charging, `#ff453a` at 20 % or less on battery; no ring on a PC without a battery | power settings for the power source and battery level; `GetSystemPowerStatus` once at start for whether a battery exists (`platform/power.rs`, true when Windows cannot say) |
| `bluetooth` | "<device> connected" with its battery when known, or "disconnected" | `ConnectionStatusChanged` of every paired device, on a thread of its own; the battery is read two seconds after a connection from the device node that reports it (`platform/bluetooth_devices.rs`, which the connected devices page reads too) |
| `device_battery` | "<device> · Low battery" with the percentage, in red, when a connected Bluetooth device falls to 20 % or 10 % (once per crossing; a lower crossing warns again, a recovery above the level re-arms it), or when a device connects already at 20 % or less; the first reading of a device at start is only a starting point. Optional: `activity.device_battery`, on by default | a WinRT `DeviceWatcher` over the Bluetooth battery nodes (`platform/bluetooth_devices.rs` has the filter and property), on a thread of its own; Windows pushes property changes into it and nothing polls. Before it announces, the device is checked with its connection status, so a stale level on a switched-off device stays quiet. Logitech receiver devices (`platform/dongle_devices`) are not included: reading them sends HID++ requests to the hardware |
| `microphone` | "Microphone on/off", then an orange dot in the lower row of the dot column at the right end (`Slot::Dots`): 6 px beside the camera's dot, 5 px when it is alone | audio sessions of the default recording device; shows that an app has the microphone open, so an app that keeps it open while you are muted (Google Meet in a browser) keeps the dot |
| `camera` | "Camera on/off", then a green 6 px dot, in the upper row of the dot column at the right end (`Slot::Dots`) | stream start and stop events in the `Microsoft-Windows-MF-FrameServer/Camera_FrameServer` event log; misses apps that bypass the Frame Server and a camera already streaming when the notch starts |
| `clipboard` | "Copied" | clipboard change message; never reads the clipboard itself (only the opt-in [Clipboard page](design-rules.md#clipboard-page) reads it, and only while it is on), skips copies that ask not to be monitored and copies the notch made itself (`platform::clipboard::is_excluded`, `is_own_write`) |
| `notifications` | a 400 DIP card with the app's own icon (the Apps folder's, else its packaged logo, else a colored letter), the sender, the app name and two lines, for 5 s. The pointer on the card holds it and adds Dismiss and Open *app* under the same two lines; it goes 1.5 s after the pointer leaves. Dismiss removes the toast from the Action Center; Open opens the app the way a click on Windows' banner does (see below) and removes the toast | `UserNotificationListener`, on a thread of its own when the notification folder changes: the new toast is asked for by id (`GetNotification` for the ids above the highest seen) at the first change and again once the folder is still, and the whole list (about 80 ms per notification in the Action Center) is read only as a fallback. The same thread removes dismissed toasts (`RemoveNotification`) |
| `caps_lock` | "Caps Lock on/off"; an arrow (rank 1) in the pill group and the header while it is on | a low-level keyboard hook on a thread of its own that only wakes the activity when the Caps Lock key goes down; the lock state is then read from Windows, and no other key is looked at |

## Opening a notification's app

`UserNotificationListener` gives no launch arguments, so Open (`platform/toast_activation.rs`) reads the toast's XML
from Windows' notification database (`%LOCALAPPDATA%\Microsoft\Windows\Notifications\wpndatabase.db`, opened
read-only through Windows' own `winsqlite3.dll`; the listener's id is the database's `Notification.Id`) and tries, in
order, until one works:

| `activationType` of the toast | Tried |
| --- | --- |
| foreground (or none) | the app's toast activator (`INotificationActivationCallback::Activate` with the toast's `launch` text, which is what Windows itself does on a click), the `launch` text as a URI when it is one, the `launch` text as launch arguments of a packaged app, then starting the app |
| protocol | the `launch` URI, then starting the app |
| background | starting the app |

The activator's class comes from the package manifest (`ToastActivatorCLSID`), else
`HKCU\Software\Classes\AppUserModelId\<id>\CustomActivator`, else the app's Start menu shortcut. Starting the app opens
`shell:AppsFolder\<id>`, which brings a running single-instance app forward.

- WhatsApp (the Store app) declares an activator, so Open hands it the toast's own `launch` text; the conversation
  opens when that text names one, as with Windows' banner. A WhatsApp toast that names no chat
  (`action=LaunchApp`) opens WhatsApp.
- A toast without `launch` (Discord, Warp) opens the app, not the message.
- A toast the database no longer holds (the app or the user cleared it) only starts the app.
- Open also removes the toast from the Action Center; the app may already have removed it.
