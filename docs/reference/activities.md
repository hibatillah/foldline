# Activities

> **Reference.** Read when: you need what an activity can show (`Presence`), or what each built-in activity watches. Skip if: you are writing one; the steps are in [add an activity](../how-to/add-an-activity.md).

## What an activity can show

An activity answers `presence()` after every event, and the host does the layout:

| Presence | Looks like | Example |
| --- | --- | --- |
| `Transient { width }` | takes over the pill for a moment, then goes | volume bar, "Copied", "Charging 56%" |
| `Badge { width }` | a small mark at the right end of the pill for as long as something lasts | the orange microphone dot |
| `Banner { width, height }` | opens the pill into a card, taller than the idle pill and smaller than the open notch, and draws into all of it; it has the pill to itself, so transients wait. Pointing at the notch does not open it while a banner shows, so the activity can hold it with `on_hover_changed` and answer a larger `presence()` while hovered | a notification |
| `None` | nothing | |

- Up to two transients show at once, side by side with a thin divider; the newest two win.
- While a transient shows, the page's collapsed content fades out and the pill resizes to the transient. Badges are
  hidden then and come back after.
- Badges shrink the room the page draws in, so the page never sits under a dot.
- Opening the notch fades every activity out, as it does the page's collapsed content.
- `width` is in device-independent pixels. `draw` gets the exact area for it, in device pixels.

## Built-in activities

| Id | Shows | Source |
| --- | --- | --- |
| `volume` | volume bar, or "Muted", and a crossed speaker in the header while muted | Core Audio endpoint volume callback; follows the default speakers; the mute state is read when listening starts |
| `brightness` | brightness bar | power setting `GUID_DEVICE_POWER_POLICY_VIDEO_BRIGHTNESS`; laptop panel only |
| `battery` | "Charging", "On battery", "Low battery" at 20 % and 10 % | power settings for the power source and battery level |
| `bluetooth` | "<device> connected" with its battery when known, or "disconnected" | `ConnectionStatusChanged` of every paired device, on a thread of its own; the battery is read two seconds after a connection from the device node that reports it |
| `microphone` | "Microphone on/off", then an orange dot | audio sessions of the default recording device; shows that an app has the microphone open, so an app that keeps it open while you are muted (Google Meet in a browser) keeps the dot |
| `camera` | "Camera on/off", then a green dot | stream start and stop events in the `Microsoft-Windows-MF-FrameServer/Camera_FrameServer` event log; misses apps that bypass the Frame Server and a camera already streaming when the notch starts |
| `clipboard` | "Copied" | clipboard change message; never reads the clipboard, skips copies that ask not to be monitored |
| `notifications` | a banner with the app's own icon (the Apps folder's, else its packaged logo, else a colored letter), who it is from and up to two lines; while the pointer is on the notch it stays and, grows taller to show up to 12 lines at the same 400 DIP width; it has no buttons | `UserNotificationListener`, read on a thread of its own when the notification folder changes; arrives a few seconds after Windows' own banner |
| `caps_lock` | "Caps Lock on/off", and an arrow in the header while it is on | a low-level keyboard hook on a thread of its own that only wakes the activity when the Caps Lock key goes down; the lock state is then read from Windows, and no other key is looked at |
