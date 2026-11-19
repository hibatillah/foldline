# MSIX packaging

> **Explanation.** Read when: you change the MSIX manifest, the version rule, or how Start with Windows works in a packaged build, or you ask why the package declares what it does. Skip if: you only build or install the package (see [build and test the MSIX](../how-to/build-and-test-the-msix.md)).

Foldline ships two ways: the Inno Setup installer (`installer/foldline.iss`) and an unsigned MSIX for the Microsoft Store (`installer/msix/AppxManifest.xml`, built by `scripts/build-msix.sh`). The programs are the same; the package changes how Windows treats their files, registry writes and startup entry. This page gives the reasons for each manifest choice and says what has been tested.

## Shape of the package

- One `Application`, `Foldline`, a full-trust desktop app (`EntryPoint="Windows.FullTrustApplication"`, capability `runFullTrust`).
- `foldline-settings.exe` is a plain file beside `foldline.exe`, not a second `Application`. The tray and the header gear start it with `CreateProcess`, as in the Inno install. If a test shows it lacks the package identity, the fallback is a second `Application` with `AppListEntry="none"`; see [what is not verified](../how-to/build-and-test-the-msix.md#what-is-not-verified-yet).
- The identity (`Name="Hibatillah.foldline"`, the `Publisher` GUID) comes from Partner Center. The package is unsigned: the Store signs it.
- The manifest is a template. The build replaces `{{VERSION}}` and nothing else.

## Capabilities

| Capability | Reason |
| --- | --- |
| `runFullTrust` | the notch is a Win32 program, not a sandboxed app |
| `unvirtualizedResources` | allows the manifest to turn registry write virtualization off (see below) |
| `userNotificationListener` | the notifications activity reads Windows notifications |
| `globalMediaControl` | the media page reads and controls the playing session |
| `location` (device capability) | the weather and Shalat pages can use the device location |
| `internetClient` | states intent in the Store listing; a full-trust process needs it for neither the internet nor a loopback listener |

## Virtualization

A packaged desktop app can have its writes to `HKCU` and to `%APPDATA%` redirected to a private copy under `%LOCALAPPDATA%\Packages\<family>`. Redirected data is invisible to Windows and to unpackaged copies of the notch, so the notch needs it off. The manifest sets `desktop6:RegistryWriteVirtualization` and `desktop6:FileSystemWriteVirtualization` to `disabled` and declares the `unvirtualizedResources` capability that registry setting requires.

### What the install test found

Tested on 2026-11-18 on Windows 11 (build 26300), with commands run through `Invoke-CommandInDesktopPackage`:

- Files created in `%APPDATA%\Foldline` from inside the package were **not** redirected. They landed in the real folder.
- `HKCU` registry writes **were** redirected until `desktop6:RegistryWriteVirtualization` was set to `disabled` and `unvirtualizedResources` was added. After both, they reach the real registry.

The registry result matters for [hiding Windows' notification banners](../how-to/hide-windows-popups.md): that feature writes `HKCU` notification settings that Windows itself reads, and a redirected write would change nothing.

Not tested: whether the Store accepts `unvirtualizedResources` (Microsoft describes it as a capability for specific cases), and what an uninstall leaves behind. The elements are kept in the manifest as tested.

## Start with Windows

A `Run` entry is not read at sign-in for a packaged app, even though its `HKCU` write now reaches the real registry. The manifest's startup task is the packaged way. The package declares a startup task, `FoldlineStartup` (`desktop:StartupTask`, `Enabled="false"`), which stays off until the user turns on **Start with Windows**.

`crates/settings/src/startup/` chooses at run time (`mod.rs`):

| The settings program has | Behavior |
| --- | --- |
| package identity (`GetCurrentPackageFullName` does not answer `APPMODEL_ERROR_NO_PACKAGE`) | reads and sets the `FoldlineStartup` task (`StartupTask.GetAsync`, `RequestEnableAsync`, `Disable`) on a background thread |
| no identity, and an `AppxManifest.xml` beside it | the `Run` key, with the note "Classic startup entry; the packaged app may ignore it" |
| no identity, no manifest beside it | the `Run` key (Inno and development builds) |

The task's states, and what the switch shows for each, are in [settings keys](../reference/settings-keys.md#start-with-windows). The one open question is identity inheritance: the program expects `foldline-settings.exe`, started by `foldline.exe`, to inherit the package identity, and this has not been run in a package.

## Version rule

An MSIX version has four numbers. `scripts/build-msix.sh` takes the workspace version from `Cargo.toml`, accepts only `major.minor.patch`, and appends `.0` (`1.0.0` becomes `1.0.0.0`); a pre-release suffix stops the script. The reason is that the Store reserves the fourth number, and a suffix cannot be written in the format.

The same plain version is passed to the build as `FOLDLINE_RELEASE_VERSION`, so both programs carry the blue stable tile; `scripts/check-icon.sh` then refuses an executable with the indigo `tile-dev` icon, because the package's logos are blue.

The Store documentation says the first number cannot be 0, so the workspace version must start at `1.0.0` or later; the script does not map a `0.x` version.

## Two installs side by side

The Inno install and the package have separate identities (`AppId` against `Hibatillah.foldline`) and install to separate places, but they share `%APPDATA%\Foldline` (settings and data) and use the same `Run` value name, `Foldline` (the package itself starts through its startup task, not that value). Both programs read the same `settings.txt`, so one copy changes what the other shows. Running both at once, or removing one while the other's startup entry exists, has not been tested. Detecting the Inno install on first packaged start is a considered change, not implemented.

## Known gaps

The open items are listed once, with how to check each, in [build and test the MSIX](../how-to/build-and-test-the-msix.md#what-is-not-verified-yet).
