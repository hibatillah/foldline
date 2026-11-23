# Build and test the MSIX

> **How-to.** Read when: you build the MSIX package of Foldline, install it on your own PC to test it, or find where a packaged copy keeps its data. Skip if: you build the Inno Setup installer or publish a release (see [cut a release](cut-a-release.md)), or want the reasons for the package's settings (see [MSIX packaging](../explanation/msix-packaging.md)).

There is no Store release and no release ships the MSIX; the files are kept as history. See [the Microsoft Store attempt](../explanation/microsoft-store-attempt.md).

The MSIX is built unsigned, only for local testing (it was built this way for the Microsoft Store, which would have signed it). You do not need a certificate to test it: Developer Mode lets Windows register the package from its staging folder. Signing and Store submission are out of scope here.

## Prerequisites

| Need | Why | Check |
| --- | --- | --- |
| Windows 11, or Windows 10 version 2004 or later (`MinVersion` 10.0.19041.0 in the manifest) | the package targets `Windows.Desktop` | `winver` |
| Developer Mode on (Settings > System > For developers) | `Add-AppxPackage -Register` needs it for an unsigned, loose-file package | the toggle in Settings |
| Windows SDK with `makeappx.exe` and `makepri.exe` | the script packs the package and indexes its logos | `command -v makeappx.exe`, or a `Windows Kits\10\bin\<version>\x64` folder |
| the MSVC Rust toolchain (`stable-x86_64-pc-windows-msvc`) | GPUI and the notch's text lookup need it | see [build, run and debug](build-run-and-debug.md) |
| Git Bash | both scripts are `.sh` | |
| `uv`, only to regenerate the logos | `scripts/build-msix-assets.sh` runs `uv run --no-project --with pillow` | `uv --version` |

`scripts/build-msix.sh` looks for the two SDK tools on `PATH` first, then under `%WindowsSdkDir%` or `Windows Kits\10` (it takes the newest `bin\<version>\x64`). When neither is found it stops with "`<tool>` not found; install the Windows SDK".

## Build the package

```
scripts/build-msix.sh [--no-build]
```

It does these steps (read the script for the exact commands):

1. Reads the workspace version from `Cargo.toml` and refuses anything but `major.minor.patch`. The package version is that number plus `.0` (`1.0.0` becomes `1.0.0.0`); see [the version rule](../explanation/msix-packaging.md#version-rule).
2. Builds `foldline.exe` and `foldline-settings.exe` in release mode with the MSVC toolchain, passing `FOLDLINE_RELEASE_VERSION=<version>`, which gives both the blue stable tile whatever the latest tag is. `--no-build` skips this and packs what is already in the target directory.
3. Runs `scripts/check-icon.sh` on both executables. It fails when one embeds the indigo `tile-dev` icon instead of the blue `tile-stable` one, which an older `--no-build` target can leave behind.
4. Stages the executables, `THIRD-PARTY-LICENSES.txt`, `Geist-LICENSE.txt`, `LICENSE` (as `LICENSE.txt`), the logos from `installer/msix/Assets/` and the manifest (`installer/msix/AppxManifest.xml` with `{{VERSION}}` replaced) in `dist/msix-<version>/package/`.
5. Builds one `resources.pri` with `makepri`, so the scale and target-size variants of the logos resolve, then packs with `makeappx pack`.

The output is `dist/<name>-<version>-x64.msix` (`<name>` is the `[[bin]]` name in `crates/notch/Cargo.toml`), and the script prints its path and the package version. `dist/` is ignored by git.

The script reads `CARGO_TARGET_DIR`: when it is set, the binaries come from `$CARGO_TARGET_DIR/x86_64-pc-windows-msvc/release`, otherwise from `target/`. Set your own when the notch runs from the default one (a running `foldline.exe` locks its file); see [build, run and debug](build-run-and-debug.md).

The script builds from the working tree, committed or not. A version with a suffix (`1.1.0-alpha.1`) stops it at once; set the workspace version to the plain number first.

### Regenerate the logos

`installer/msix/Assets/` holds generated PNGs. After the blue tile (`assets/branding/tile-stable.png`, 256 px) changes, run:

```
scripts/build-msix-assets.sh
```

and commit the output. The build never runs Python.

## Install it on this PC

```
scripts/install-msix-local.sh --build
```

`--build` first removes an installed copy, runs `scripts/build-msix.sh`, then registers the package. Without `--build` the script registers the package already staged in `dist/msix-<version>/package/` and stops with "no staged package; run with --build" when there is none. After registering, it prints the package full name and starts the app (`shell:AppsFolder\<package family name>!Foldline`).

What the script does is `Add-AppxPackage -Register` on the staged `AppxManifest.xml`. That has two consequences:

- The installed package runs from `dist/`, not from a copy. Rebuilding replaces files the package uses, so remove it before you rebuild; `--build` does that for you.
- Registration needs Developer Mode. Without it, Windows refuses (see [troubleshooting](#troubleshooting)).

The package identity is `Hibatillah.foldline` (from Partner Center) and its package family name is `Hibatillah.foldline_5wtw3aqek2sww`.

## Launch it

The install script starts the notch. Later, start **Foldline** from the Start menu, or from a shell:

```
explorer.exe shell:AppsFolder\Hibatillah.foldline_5wtw3aqek2sww!Foldline
```

`foldline-settings.exe` is a plain file beside `foldline.exe` (the manifest has one `Application`, `Foldline`). The gear in the open notch and the tray's **Settings** item start it, as in the Inno install.

## Remove and reinstall

```
scripts/install-msix-local.sh --remove
```

This runs `Get-AppxPackage Hibatillah.foldline | Remove-AppxPackage` and stops. To reinstall after code changes, run `scripts/install-msix-local.sh --build`.

Quit the notch first when you can: a running packaged `foldline.exe` can keep the registration from being replaced (see [troubleshooting](#troubleshooting)).

## Check where data lands

Run a command with the package's identity, so you see what the packaged notch sees:

```
Invoke-CommandInDesktopPackage -PackageFamilyName Hibatillah.foldline_5wtw3aqek2sww -AppId Foldline -Command powershell.exe -Args '-NoProfile -Command "<your command>"'
```

Run it in PowerShell. `-AppId Foldline` is the `Application Id` in the manifest. The packaged process can then be asked to create a file or a registry value, and you compare the result with the real locations:

| Check | Where to look | Expected |
| --- | --- | --- |
| a file written under `%APPDATA%\Foldline` | `%APPDATA%\Foldline` (real) against `%LOCALAPPDATA%\Packages\Hibatillah.foldline_5wtw3aqek2sww\LocalCache\Roaming\Foldline` (redirected, the usual package location; not seen in the test) | the real folder; `settings.txt` is shared with an Inno install |
| an `HKCU` value, such as `ShowBanner` under `HKCU\Software\Microsoft\Windows\CurrentVersion\Notifications\Settings\...` | `reg query` on the real key | the real key, so [hiding Windows' notification banners](hide-windows-popups.md) reaches Windows |
| the startup entry | **Settings > Apps > Startup** | a **Foldline** entry whose switch follows the General page of the settings window (not verified, see below) |

The first two rows are the findings of a local install test on Windows 11 (build 26300); the reasons are in [MSIX packaging](../explanation/msix-packaging.md#what-the-install-test-found). Repeat them after you change the manifest's capabilities or virtualization elements.

## What is not verified yet

Treat these as open when you test; none is confirmed by a packaged run.

- **Start with Windows from the settings window.** Whether `foldline-settings.exe`, started by `foldline.exe` with `CreateProcess`, inherits the package identity. Without identity the window falls back to the `Run` key and shows "Classic startup entry; the packaged app may ignore it" (`crates/settings/src/startup/mod.rs`). Open the General page from the packaged notch and switch the setting on and off, then read **Settings > Apps > Startup**.
- **Notification listener access.** Whether the `userNotificationListener` capability gives the packaged notch the access prompt and the notifications.
- **Uninstall leftovers.** What stays in `%APPDATA%\Foldline` and `HKCU` after `Remove-AppxPackage`. Uninstall runs none of the notch's code.
- **The Store version rule.** The first number must not be 0; see [the version rule](../explanation/msix-packaging.md#version-rule).
- **Coexistence with an Inno install.** The two have separate identities but share `%APPDATA%\Foldline` and the `Run` value name `Foldline`. Do not run both at once until this is checked.

## Troubleshooting

| Symptom | Cause and fix |
| --- | --- |
| `build-msix.sh` ends with "`makeappx.exe` not found" or "`makepri.exe` not found" | the Windows SDK is not installed or is outside `PATH` and `Windows Kits\10`. Install the SDK (the MSIX packaging tools come with it), or put its `bin\<version>\x64` folder on `PATH`. |
| "workspace version '...' is not major.minor.patch" | the version in `Cargo.toml` has a suffix. Use the plain number. |
| "... embeds tile-dev, expected tile-stable" | the binaries in the target directory come from a pre-release build. Run without `--no-build`. |
| `Add-AppxPackage` fails with an error about Developer Mode or about an unsigned package | turn Developer Mode on and run the script again. |
| the registration fails with a "deployment" error that names a file in use, or the old package keeps running | a packaged or Inno `foldline.exe` is running. Quit it (tray menu, or end `foldline.exe` in Task Manager), run `--remove`, then install again. |
| two notches at once | an Inno copy and the packaged one run together, or a second copy was started from `target\`. Quit all but one; they share one settings file and one window slot. |
| the build fails after the first install | the package runs from `dist/msix-<version>/package`; `--remove` it, or use `--build`, which removes it first. |
| `no staged package; run with --build` | `dist/msix-<version>/package/AppxManifest.xml` does not exist for the current workspace version. Run with `--build`. |

When none of this explains the failure, `Get-AppxLog` (PowerShell, newest entries last) and Event Viewer's **AppXDeployment-Server** log hold Windows' own message.

## Out of scope

- **Signing.** The package is unsigned on purpose; the Store signs it. A self-signed test certificate is not part of this workflow.
- **Store submission.** Partner Center steps, listing text, and the App Certification Kit are not covered.
- **The Inno installer.** See [cut a release](cut-a-release.md).
