# Notch

A Windows "dynamic island" written in Rust: a black pill at the top of the screen, drawn with Direct2D, no UI
framework. `notch.exe` idles at 0 % CPU and stays under 50 MB of memory. Windows only.

The settings window is a second program, `notch-settings.exe` (`settings/`, built with GPUI), that the tray's
**Settings…** item starts. It has a higher footprint while open and is outside that budget. `store/` holds the settings
file code both programs share. Most settings also live inside the notch, in the panel behind the header gear.

## Commands

Root commands cover only the notch (the workspace's default member). GPUI builds only with the MSVC host toolchain, so
run `settings/` and `store/` commands as `cargo +stable-x86_64-pc-windows-msvc ... -p notch-settings` (or `--workspace`).

```
cargo build --release                     # target\x86_64-pc-windows-msvc\release\notch.exe
cargo test                                # unit tests, no window needed
cargo test snapshots -- --ignored         # writes pictures of the pages to target/snapshots
cargo fmt --check
cargo clippy --all-targets -- -D warnings
scripts/reload.sh [--debug]               # build, start the new build, close the old one (Git Bash)
scripts/watch.sh                          # debug build of the notch, relaunched on every saved source file (Git Bash)
scripts/perf.sh [--no-build]              # memory and CPU checks; moves the mouse for about two minutes
```

## Find code

`.codegraph/` indexes this repository. Ask `codegraph_explore` (pass `projectPath`) for a symbol, a caller or a call
path before grepping; it returns the current source with line numbers. The index follows edits while the MCP server
runs; `codegraph sync` catches up after changes made outside a session.

## Rules

`.claude/rules/` holds the constraints, loaded by the paths they cover. Three always apply: `workflow.md` (checks before
done, no commits unless asked, never kill the notch, specs and plans go to `.superpowers/`), `performance.md` (no
polling, memory budget) and `contributions.md` (no outside contributions).

## Docs

`docs/README.md` indexes every document by type. Open the one that matches the work in front of you before you start:

| Work | Read |
| --- | --- |
| a page: first one, or looking up a hook or rule | `docs/tutorials/add-a-page.md`, `docs/reference/page-api.md` |
| an activity (volume, Caps Lock, notification banner) | `docs/how-to/add-an-activity.md`, `docs/reference/activities.md` |
| a setting, a key, a control in the panel or the window | `docs/how-to/add-a-setting.md`, `docs/reference/settings-keys.md` |
| switching pages or activities off, or removing them | `docs/how-to/switch-off-or-remove.md` |
| build, run, debug, a page that does not show | `docs/how-to/build-run-and-debug.md` |
| a test, a snapshot, the manual checklist | `docs/how-to/test-and-snapshot.md`, `docs/reference/test-map.md` |
| `perf.sh`, a CPU or memory number | `docs/how-to/run-the-perf-check.md`, `docs/reference/performance-baseline.md` |
| padding, the header, the open and collapsed shapes | `docs/reference/layout.md` |
| restructuring, or where something lives | `docs/explanation/architecture.md`, `docs/reference/source-map.md` |
| a dependency, a license | `docs/reference/dependencies-and-licenses.md` |
| the installer, a release, updating | `docs/how-to/cut-a-release.md` |

## Debugging

`NOTCH_DEBUG=<file>` writes a timestamped log (Unix milliseconds). `NOTCH_PAGES=shelf,media` picks pages,
`NOTCH_LYRICS=off` stops lyric lookups, `NOTCH_TOP=<px>` moves the pill down. Settings are in
`%APPDATA%\notch\settings.txt`; every key is in `docs/reference/settings-keys.md`.
