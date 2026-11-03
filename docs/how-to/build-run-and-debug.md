# Build, run and debug

> **How-to.** Read when: you build the notch, run a change, or need to see why something does not show. Skip if: you only need a command's name: the list is in [AGENTS.md](../../AGENTS.md).

## Requirements for building

These are needed only to build from source. To run an already-built `notch.exe` you need just Windows 10 or 11.

- Windows 10 or 11 (developed and tested on Windows 11; Windows 10 is untested)
- [Rust](https://rustup.rs) 1.88 or newer (tested with 1.97); `Cargo.toml` sets `rust-version = "1.88"`
- Visual Studio Build Tools with the **Desktop development with C++** workload, which provides the MSVC linker and
  the Windows SDK
- [Git for Windows](https://git-scm.com/download/win), only for the `.sh` scripts (`scripts/reload.sh`,
  `scripts/watch.sh`, `scripts/perf.sh`), which run in Git Bash
- [Inno Setup](https://jrsoftware.org/isinfo.php), only to build the installer by hand; the release workflow installs it

`.cargo/config.toml` makes `x86_64-pc-windows-msvc` the default target. If your Rust install uses the GNU toolchain,
add the target once:

```
rustup target add x86_64-pc-windows-msvc
```

The settings window (`settings/`, GPUI) builds only with the MSVC host toolchain: install it with
`rustup toolchain install stable-x86_64-pc-windows-msvc` and run its commands as `cargo +stable-x86_64-pc-windows-msvc`.

## Build

```
git clone https://github.com/hibatillah/notch.git
cd notch
cargo build --release
```

The executable is `target\x86_64-pc-windows-msvc\release\notch.exe`. The workspace's default member is the notch, so
this does not build the settings window; `cargo +stable-x86_64-pc-windows-msvc build --release -p notch-settings` builds
`notch-settings.exe` beside it, and `--workspace` builds both. A debug build (`cargo build`) works too, but
animation is noticeably slower, so use release for anything you look at.

## App icon

`assets/branding/logo.png` is the orange master logo; the shaded black and white variants live beside it.
`assets/branding/logo.ico` is the Windows export, with transparent 16, 20, 24, 32, 40, 48, 64, 96, 128 and 256 px frames.
After changing the master, regenerate the export from Git Bash with `scripts/export-icon.sh` (requires `uv`; Pillow
is used only for this export, not by the app or normal builds).

`build.rs` uses the Windows SDK resource compiler to embed `logo.rc` and `logo.ico` in the executable, and `settings/build.rs` embeds `settings.rc` (the same icon as resource 1) in `notch-settings.exe` for its window and taskbar button; both share `scripts/icon_resource.rs`. The window and
tray load this same resource at the current display scale. The installer uses the same ICO, and its Start menu
shortcut and installed-app entry use the executable's embedded icon. No image files are needed beside the installed
executable. Rebuild and restart Notch to see an icon change; updating an installed copy requires the newer installer.
Windows may cache an old shortcut icon until its icon cache refreshes.

## Run

```
target\x86_64-pc-windows-msvc\release\notch.exe
```

The notch has no window to close. It adds a tray icon (on Windows 11 it may be under the `^` overflow menu).
Right-click the tray icon and choose **Quit**.

## Development loop

A running `.exe` is locked by Windows, so you cannot rebuild over it. `scripts/reload.sh` (run from Git Bash) works
around that:

```
scripts/reload.sh
```

It builds the notch and the settings program, copies the notch build to `run/notch-<time>.exe` and the settings program
to `run/notch-settings.exe` (the name the tray's **Settings…** item looks for beside the running notch), starts the
notch copy, and asks the previous instance to close once the new one is showing. The notch never disappears between
versions. `run/` is git-ignored. A settings window that is still open keeps the old `notch-settings.exe` until you close
it and reload.

To see a change as soon as you save it, leave this running in a terminal:

```
scripts/watch.sh
```

It does the same with a debug build (`scripts/reload.sh --debug`, about 2 s after a small edit, against about 20 s for
release) every time a file in `src/`, `store/` or `Cargo.toml` changes (a saved file in `settings/` does not trigger it; the next rebuild picks the change up), and checks for saved files every 2 s. A debug build is
slower and larger, so use the release build and `scripts/perf.sh` for performance numbers. If a build fails, the
error shows and the previous copy keeps running until the next save.

To see what the notch is doing, write a timestamped log:

```
NOTCH_DEBUG=C:\path\to\notch.log scripts/reload.sh
```

All settings and environment variables are in [configuration.md](../reference/settings-keys.md).

## Checks

Run these before considering a change finished, in this order:

```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
scripts/perf.sh          # when you touched the host, animation, timers or drawing
```

The checks above cover only the notch; add `--workspace` (with the MSVC toolchain) to include `store/` and `settings/`.
CI runs the first three on every push to the default branch ([releasing.md](cut-a-release.md)); run them yourself first. Where the tests live and what they cover: [testing.md](test-and-snapshot.md).
`scripts/perf.sh` moves the mouse for about two minutes, so leave the mouse alone while it runs.

## A page does not show up

- Another page has a higher priority, or the same priority and an earlier place in `all()`.
- `priority` returns `None` for the mode you are looking at.
- `NOTCH_PAGES` or the `pages` setting leaves it out.
- `sizes()` returns zero.
- Set `NOTCH_DEBUG=<file>`: the log lists the registered pages and any that are switched off.
