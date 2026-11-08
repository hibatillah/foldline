# Hide Windows' own popups

> **How-to.** Read when: Windows shows its own volume box or notification banner next to the notch and you want only the notch. Skip if: you want Windows' popups as well.

Both switches are under **Settings > General > Windows popups** and are off by default.

## The volume popup

Turn on **Hide the Windows volume popup** (`notch.hide_windows_volume_popup`).

- Explorer draws the volume box in a window of class `XamlExplorerHostIslandWindow` (no title, never takes focus,
  centered above the taskbar). It shows that window again at every volume change, so the notch hides it each time:
  an `EVENT_OBJECT_SHOW` hook on the window's own thread hides it about 2 ms after Explorer shows it
  (`crates/notch/src/host/volume_flyout.rs`).
- The hook runs on a thread of the notch's own (`volume_flyout/hook_thread.rs`), not on the thread that draws the
  notch, because the notch draws its own volume message at the same moment.
- Measured on Windows 11 build 26300: over eight volume changes, one frame of the box was captured on screen once;
  without the setting it stays for about 2.5 s after each change. A stand-in for the hook on an idle thread hid the box
  1 to 4 ms after it was shown, and a check every millisecond never saw it on screen.
- Moving the hidden box off screen does not help: Explorer moves it back before it shows it.
- Nothing in Windows is changed. When the notch closes, is switched off here, or crashes, the hook ends and the box
  appears again at the next volume change.
- Until the first volume change after Explorer starts, the window may not exist; the notch then listens to Explorer's
  shows until it appears. After Explorer restarts, the notch looks for the new window.
- A Windows update can change the window. The box then simply shows again; no other window is hidden, because the
  notch checks the class, the missing title, the extended style and the position.

## The brightness popup

Not confirmed. A brightness change made through WMI (`WmiMonitorBrightnessMethods`) shows no box at all on build
26300, no window appears or changes, and software cannot press the brightness keys. If the keys show their box in the
same window as the volume box, the volume switch already hides it, because the hook hides that window at every show;
if they use another window, it still shows.

## Notification banners

Turn on **Hide Windows notification banners** (`notch.hide_windows_notifications`).

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
