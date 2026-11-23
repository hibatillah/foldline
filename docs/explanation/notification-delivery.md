# Notification delivery

> **Explanation.** Read when: you change the notification listener or its icon lookup, or you explain to a tester why the installer version shows notifications later than a packaged (MSIX) build. Skip if: you only use the notch (see the Limitations in the [README](../../README.md)).

Both builds read Windows' notifications through `UserNotificationListener`. They differ in how they learn that a new one arrived (`crates/notch/src/activities/notifications/listener.rs`).

| | Packaged (MSIX) build | Installer (`-setup.exe`) build |
| --- | --- | --- |
| How a new notification is found | Windows calls the `NotificationChanged` event | the listener watches `%LOCALAPPDATA%\Microsoft\Windows\Notifications` for writes, then asks for the next ids |
| Needs package identity | yes | has none until a signed sparse package ships (below): subscribing fails with `0x80070490` |
| Measured delay after Windows' banner | 123 to 293 ms; 20 toasts in a burst all arrived, the last in about 1 s | about 0.3 s per toast; the first toast after start-up a few seconds later (the first full read of the list takes about 3 s) |
| Polling or reading the whole list | none | the list is read only when asking by id finds nothing |

`run` tries the event first in every build and falls back to the folder watch when the subscription fails, so one code path serves both.

## Why the installer build can be late

Windows writes a toast into its database when it chooses to. The folder watch only wakes when that write happens, so a toast that Windows shows but stores late reaches the notch late too. Testers have reported delays of up to a minute with the installer build; this has not been reproduced with test toasts, which arrive within a second. A packaged build does not wait for the database: the event comes from Windows itself.

A package identity for the installer build removes the wait for the database. The pieces for it are in place (next section), but the package must be signed and the project has no certificate yet, so every release stays on the folder watch. Until then the README says where to expect the delay, and a tester who sees one should compare with Windows' own banner for the same notification before reporting a bug. No release is packaged.

## The sparse package

A sparse package gives a program outside any package a package identity: Windows registers a manifest with no program inside it and points it at the install folder (`Add-AppxPackage -ExternalLocation`), and the program claims that package through an `<msix>` element in its own embedded manifest.

| Piece | Where |
| --- | --- |
| package manifest (`Foldline.Desktop`, `uap10:AllowExternalContent`, the `userNotificationListener` capability) | `installer/sparse/AppxManifest.xml` |
| `<msix>` element in `foldline.exe`, built from that manifest's `Identity Name`, `Publisher` and `Application Id` | `crates/notch/build.rs`, `scripts/sparse_identity.rs` |
| packing it (unsigned) to `dist/foldline-sparse-<version>.msix` | `scripts/build-sparse-package.sh` |
| shipping it to the install folder and registering it after install, removing it on uninstall | `installer/foldline.iss`, only when `installer/sparse/foldline-sparse.msix` exists and the build is stable |

The installer step is switched on by the file alone: with no `installer/sparse/foldline-sparse.msix`, the installer is the same as before. A failed registration is only written to the setup log, and `foldline.exe` then runs without identity on the folder watch.

### Signing it

1. Set `Publisher` in `installer/sparse/AppxManifest.xml` to the subject of the signing certificate, character for character (the placeholder is `CN=Foldline`). It is the only copy: the build script copies it into `foldline.exe`.
2. Build `foldline.exe` again, so its `<msix>` element carries the new publisher.
3. Run `scripts/build-sparse-package.sh` and sign `dist/foldline-sparse-<version>.msix` with that certificate (`signtool sign /fd SHA256 ...`).
4. Copy the signed file to `installer/sparse/foldline-sparse.msix` before the installer is built.

### What was measured (2026-11-23, Windows 11 build 26300, Developer Mode on)

- Registered unsigned from a folder (`Add-AppxPackage -Register <manifest> -ExternalLocation <folder>`, which Developer Mode allows), `foldline.exe` logs `notifications: listening for change events` instead of the `0x80070490` fallback, and test toasts arrive through the event.
- Time from `Show()` of a PowerShell test toast to the card: 167 to 694 ms with the event (6 toasts), 183 to 602 ms with the folder watch on the same machine (4 toasts). Test toasts are written to the database at once, so they show no gain; the gains are the late writes described above (not reproduced) and the first read at start-up, which the event path skips (7.7 s in this run).
- Windows gives no identity, and the program runs on the folder watch, when the package is not registered or when the `<msix>` publisher differs from the package's.
- An unsigned `.msix` file does not install: `0x800B0100` ("must be digitally signed"). The unsigned route of Windows 11 (`-AllowUnsigned` with the `OID.2.25.311729368913984317654407730594956997722=1` publisher) refuses it too, because an unsigned package may not declare an `Application` (`0x80073D2B`).
- A program started by a process with the identity does not inherit it, so `foldline-settings.exe` keeps the `Run` key for Start with Windows.
- A `foldline.exe` that carries the `<msix>` element gets the identity from any folder while the package is registered, not only from the install folder.

Not verified: the signed install through the installer (no certificate), and whether the identity changes what Windows asks for location access or media sessions (the manifest declares the same capabilities as the MSIX one).

## The icon of the app

A card shows the icon of the app that sent the toast. It is looked up once per app, in this order, and the first that is not Windows' generic file icon wins (`platform/app_icon.rs`, `notifications/package_logo.rs`):

1. the shell icon of `shell:AppsFolder\<app id>`;
2. the icon of a running program with that file name (classic apps);
3. the icon file named by `IconUri` under `AppUserModelId\<app id>` in the registry, per user first, then for the machine (an app that registers its toasts under its own id, Firefox);
4. the icon of the largest `.exe` in the folder the app id's package is installed in (Edge installs as a classic app with a stub package, so its package identity has no icon of its own);
5. the app's logo from Windows (`AppInfo.DisplayInfo.GetLogo`), else the logo file named in the package manifest.

With the debug log on (Settings, Advanced), a lookup writes `notifications: icon for <app id> (<name>) came from <source>`, or `no icon for …` when every step failed; send that line when an app shows the placeholder.

A test toast for an app is sent from PowerShell with `ToastNotificationManager.CreateToastNotifier('<app id>')`; the Edge id is `Microsoft.MicrosoftEdge.Stable_8wekyb3d8bbwe!App` and Chrome's is `Chrome` (`Get-StartApps` lists the others).
