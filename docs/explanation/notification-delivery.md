# Notification delivery

> **Explanation.** Read when: you change the notification listener or its icon lookup, or you explain to a tester why the installer version shows notifications later than the Store version. Skip if: you only use the notch (see the Limitations in the [README](../../README.md)).

Both builds read Windows' notifications through `UserNotificationListener`. They differ in how they learn that a new one arrived (`crates/notch/src/activities/notifications/listener.rs`).

| | Store (MSIX) build | Installer (`-setup.exe`) build |
| --- | --- | --- |
| How a new notification is found | Windows calls the `NotificationChanged` event | the listener watches `%LOCALAPPDATA%\Microsoft\Windows\Notifications` for writes, then asks for the next ids |
| Needs package identity | yes | not possible: subscribing fails with `0x80070490` |
| Measured delay after Windows' banner | 123 to 293 ms; 20 toasts in a burst all arrived, the last in about 1 s | about 0.3 s per toast; the first toast after start-up a few seconds later (the first full read of the list takes about 3 s) |
| Polling or reading the whole list | none | the list is read only when asking by id finds nothing |

`run` tries the event first in every build and falls back to the folder watch when the subscription fails, so one code path serves both.

## Why the installer build can be late

Windows writes a toast into its database when it chooses to. The folder watch only wakes when that write happens, so a toast that Windows shows but stores late reaches the notch late too. Testers have reported delays of up to a minute with the installer build; this has not been reproduced with test toasts, which arrive within a second. The Store build does not wait for the database: the event comes from Windows itself.

A package identity for the installer build would remove the delay but needs a signed sparse package, which the project does not have. Until then the README says where to expect the delay, and a tester who sees one should check the same notification in the Store build before reporting a bug.

## The icon of the app

A card shows the icon of the app that sent the toast. It is looked up once per app, in this order, and the first that is not Windows' generic file icon wins (`platform/app_icon.rs`, `notifications/package_logo.rs`):

1. the shell icon of `shell:AppsFolder\<app id>`;
2. the icon of a running program with that file name (classic apps);
3. the icon file named by `IconUri` under `AppUserModelId\<app id>` in the registry, per user first, then for the machine (an app that registers its toasts under its own id, Firefox);
4. the icon of the largest `.exe` in the folder the app id's package is installed in (Edge installs as a classic app with a stub package, so its package identity has no icon of its own);
5. the app's logo from Windows (`AppInfo.DisplayInfo.GetLogo`), else the logo file named in the package manifest.

With the debug log on (Settings, Advanced), a lookup writes `notifications: icon for <app id> (<name>) came from <source>`, or `no icon for …` when every step failed; send that line when an app shows the placeholder.

A test toast for an app is sent from PowerShell with `ToastNotificationManager.CreateToastNotifier('<app id>')`; the Edge id is `Microsoft.MicrosoftEdge.Stable_8wekyb3d8bbwe!App` and Chrome's is `Chrome` (`Get-StartApps` lists the others).
