# Hide Windows' own popups

> **How-to.** Read when: Windows shows its own volume or brightness box or notification banner next to the notch and you want only the notch. Skip if: you want Windows' popups as well.

Both switches are under **Settings > General > Windows popups** and are on by default (an unset key counts as on); switch one off to get that Windows popup back. To get both back for a while, pause the notch from the tray or **Settings > General > Pause Notch**: the notch is hidden completely and Windows' popups show as usual until the pause ends, then are hidden again ([pausing the notch](../reference/settings-keys.md#pausing-the-notch)).

## The volume and brightness popups

**Hide the Windows volume and brightness popups** (`notch.hide_windows_volume_popup`).

- With the switch on, the notch is the only feedback for volume and brightness, so their level message (a volume or brightness change, a mute or unmute; the muted
  speaker that stays afterwards follows the word) shows over a maximized or full-screen window whatever
  `notch.maximized` and `notch.fullscreen` say, `hidden` included, and while Hide when idle is on. Only a screenshot
  overlay hides it. With the switch off, Windows' own box is back and the notch follows those settings
  ([settings keys](../reference/settings-keys.md#what-the-notch-shows-over-another-window)).

- Explorer draws both boxes in one window of class `XamlExplorerHostIslandWindow` (no title, never takes focus,
  centered above the taskbar, extended style `0x08200088` on build 26300). The notch gives that window an empty region
  (`SetWindowRgn`), which Windows accepts from another process and Explorer never resets, so every show draws nothing
  (`crates/notch/src/host/volume_flyout.rs`).
- A show hook on the window's own thread (`volume_flyout/hook_thread.rs`) finds the window at its first show, because it
  does not exist until the first volume or brightness change after Explorer starts. Until then the hook listens to
  Explorer's shows; after Explorer restarts the notch looks for the new window. The hook is only a finder and a backstop
  that empties the region again if it was lost.
- Hiding the window after Explorer showed it (the earlier approach) left one or two frames of the box on screen, mostly
  with quick key presses: the box was already drawn when the hide arrived.
- A test with the real keys on build 26300, with the region emptied, showed no box at all for volume and brightness.
- The region is put back when the notch exits, when the switch is turned off and at the end of a Windows session. A newer
  notch that replaces a running one keeps the flyout concealed. A notch ended by force (Task Manager) cannot put it
  back: the box stays away until the next start, which puts an empty region left by an earlier notch back first,
  whatever the switch says, or until Explorer restarts.
- A Windows update can change the window. The box then simply shows again; no other window is touched, because the
  notch checks the class, the missing title, the extended style and the position.

## Notification banners

**Hide Windows notification banners** (`notch.hide_windows_notifications`).

The notch reads notifications from the notification center list (`UserNotificationListener`), so a notification
reaches it as long as Windows keeps it in the notification center. The switch turns off only the banner, for every
app, while the notch runs:

- It writes `ShowBanner=0` (DWORD) under
  `HKCU\Software\Microsoft\Windows\CurrentVersion\Notifications\Settings\<app id>` for every app listed there, the
  same value **Show notification banners** in Windows Settings > System > Notifications writes. `Enabled` and
  `ShowInActionCenter` are never touched, so notifications stay in the notification center.
- Before changing anything it saves each app's previous value (or that it had none) in
  `%APPDATA%\notch\notification-banners.txt`. An app whose banner was already off is left out and never changed.
- A thread waits on `RegNotifyChangeKeyValue` for that key. An app gets its key when it shows its first notification,
  and the notch turns its banner off then; that first banner may still show.
- The saved values go back when the switch is turned off, when the notch quits, when Windows signs out or shuts down,
  and at the next start if the notch crashed. A value changed since (for example by turning the banner back on in
  Windows Settings, which the notch then turns off again) and an app removed meanwhile are left alone.
- When a newer notch replaces a running one, the old one leaves the values to the new one.

## What quitting guarantees

`crates/notch/src/host/shutdown.rs` holds the exit path:

- **Quit Notch** destroys the window first, so the pill and the tray icon go at once, then both popups are put back.
  Putting them back runs on a short-lived thread and gets 3 seconds; the banner watch gets 1 second to stop. After
  that the process ends anyway, and values not yet restored stay in the backup for a later restore.
- The notch's state is not dropped at exit. Rust runs the main thread's thread-local destructors inside
  `ExitProcess`, after Windows has ended every other thread, so a destructor that waits on one of them never returns.
- A panic in a release build (`panic = "abort"`) puts both popups back from the panic hook before the process aborts.
- Nothing runs when the process is ended by force; see the flyout and banner sections above for what the next start
  puts back.

Tested on build 26300 with a test app:

| Per-app values | Windows banner | Reaches the notch |
| --- | --- | --- |
| none (default) | yes | yes |
| `ShowBanner=0` | no | yes |
| `ShowInActionCenter=0` | yes | yes, while the banner is up |
| `ShowBanner=0` and `ShowInActionCenter=0` | no | listed shortly after it arrives; not kept |
| `Enabled=0` | no | no |

Limits:

- Windows' notification service keeps its own copy of these values. A value removed from the registry can stay in
  force until the service restarts, so a banner may stay off after the notch puts the value back. Whether this
  happens when the notch deletes a value it added is not yet measured.
- Alarms, reminders and incoming calls lose their Windows banner as well.
- Writing `ToastEnabled=0` under `HKCU\Software\Microsoft\Windows\CurrentVersion\PushNotifications` had no effect
  while Windows ran, and turning all notifications off in Windows Settings also stops them reaching the notch, so
  neither is used.
