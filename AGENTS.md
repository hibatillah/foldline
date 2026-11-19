# Foldline

A Windows "dynamic island" written in Rust: a black pill at the top of the screen, drawn with Direct2D, no UI framework. `foldline.exe` idles at 0 % CPU and stays under 50 MB of memory. Windows only, open source under GPL-3.0-or-later.

The settings window is a second program, `foldline-settings.exe` (`crates/settings/`, GPUI), started by the gear in the open notch and the tray's **Settings** item. It has a higher footprint and is outside the budget. Settings are edited only there, and `crates/store/` holds the settings file code both programs share.

## Code layout

A Cargo workspace: `crates/notch` (the pill: `host/`, `pages/`, `activities/`, `gfx/`, `ui/`, `platform/`), `crates/settings`, `crates/store`, `crates/places` (city list and shared location) and `vendor/salah` (the prayer-time crate with one fix). The package and directory stay `notch`; the binary is `foldline`. Overview: `ARCHITECTURE.md`. Reasons: `docs/explanation/host-and-pages.md`. Where each module lives and how one is laid out: `docs/reference/source-map.md`. Keep new code to them.

## Commands

Build every crate with the MSVC toolchain (GPUI and the notch's rust-i18n need it). Root commands cover only the notch; add `-p settings` or `--workspace` for the other crates.

```
cargo +stable-x86_64-pc-windows-msvc build --release              # target\x86_64-pc-windows-msvc\release\foldline.exe
cargo +stable-x86_64-pc-windows-msvc test                         # unit tests, no window needed
cargo +stable-x86_64-pc-windows-msvc test snapshots -- --ignored  # page pictures in target/snapshots
cargo +stable-x86_64-pc-windows-msvc fmt --check
cargo +stable-x86_64-pc-windows-msvc clippy --all-targets -- -D warnings
scripts/reload.sh [--debug]    # build, start the new build, close the old one (Git Bash)
scripts/watch.sh               # debug build, relaunched on every saved source file
scripts/test-build.sh          # installer of the current tree in dist/ for a teammate
scripts/perf.sh [--no-build]   # memory and CPU checks; moves the mouse for about three minutes
```

- Set your own `CARGO_TARGET_DIR` per agent or session, so a build never overwrites the running `foldline.exe` or another agent's artifacts. A target directory takes 3 to 9 GB, so delete stale ones.
- An incomplete `tree-sitter` build in a stale target directory: `cargo clean -p tree-sitter` there, then build again.
- The settings tests fail while a `foldline-settings.exe` window is open (single-instance mutex). Close it first.

## Find code

`.codegraph/` indexes the repository. Ask `codegraph_explore` (pass `projectPath`) for a symbol, a caller or a call path before grepping. `codegraph sync` catches the index up after edits made outside a session.

## Rules

`.claude/rules/` holds the constraints, loaded by the paths they cover. Always in force: `workflow.md` (checks before done, no commits unless asked, never replace the running notch except through the scripts, specs and plans go to `.superpowers/`), `performance.md` (no polling, the memory budget) and `contributions.md` (open source, no accounts or paid tiers). `module-layout.md` cuts Rust files (one job per file, about 400 lines), and `code-style.md` and `testing.md` cover the Rust code and scripts.

Privacy: the clipboard page, when switched on, keeps the last seven copies in memory only. Nothing is written to disk or sent anywhere, and the history is dropped when the page is switched off or the notch quits. The "Copied" activity never reads what was copied. Keep both as they are, and keep every statement about them in the docs true.

## Commits and pushes

Commit only when the user asks, and then with `/group-commit` (Skill tool): never `git commit` by hand and never another commit skill. It makes one commit per scope and never pushes.

Push only when the user asks. The tag `v0.5.0` exists, so every change goes through a branch and a pull request (`CONTRIBUTING.md`). Never push to `master`. When asked to, stop, say it breaks the workflow and offer a branch and pull request instead.

## Subagents

**A change under an area's paths MUST be dispatched to that area's subagent through the Agent tool, not edited inline.** The main session plans, orders the areas, runs commits and answers questions. An agent without a subagent mechanism reads the matching file in `.claude/agents/` and follows it.

| Agent | Area |
| --- | --- |
| `notch-render` | `crates/notch/src/gfx`, `crates/notch/src/ui` |
| `notch-host` | `crates/notch/src/host`, `crates/notch/src/platform` |
| `notch-pages` | `crates/notch/src/pages`, `activities`, `page.rs`, `activity.rs`, `crates/places` |
| `settings-gpui` | `crates/settings`, `crates/store` |
| `perf-auditor` | read-only budget review after timers, drawing, threads or caching change |
| `docs-writer` | `docs/**`, `README.md`, `ONBOARDING.md` |

A task across areas goes in this order: `notch-render`, then `notch-host` and `notch-pages`, then `settings-gpui`, then `perf-auditor`, then `docs-writer`. Give each agent a self-contained brief with the goal, the files or boards to read (design boards are in `.superpowers/design/`) and what to report. Agents that build run one at a time per source tree, each with its own `CARGO_TARGET_DIR`.

The agent files, `.claude/rules/` and this file belong to the main session, and `docs-writer` does not edit them. After editing an agent file, run `claude plugin validate .claude/agents`: one unescaped `'` in a description (write `''` inside a single-quoted one) stops that agent from loading.

## Docs

`docs/README.md` indexes every document by type. Open the one that matches the work before you start:

| Work | Read |
| --- | --- |
| a page, or a hook or rule of the `Page` trait | `docs/how-to/add-a-page.md`, `docs/reference/page-api.md` |
| an activity | `docs/how-to/add-an-activity.md`, `docs/reference/activities.md` |
| a setting, a key, a control in the settings window | `docs/how-to/add-a-setting.md`, `docs/reference/settings-keys.md`, `docs/reference/settings-window.md` |
| hiding Windows' volume flyout or notification banners | `docs/how-to/hide-windows-popups.md` |
| switching pages or activities off, or removing them | `docs/how-to/switch-off-or-remove.md` |
| build, run, debug, a page that doesn't show | `docs/how-to/build-run-and-debug.md` |
| a test, a snapshot, the manual checklist | `docs/how-to/test-and-snapshot.md`, `docs/reference/test-map.md` |
| `perf.sh`, a CPU or memory number | `docs/how-to/run-the-perf-check.md`, `docs/reference/performance-baseline.md`, `docs/explanation/performance-budget.md` |
| padding, the header, the open and collapsed shapes | `docs/reference/layout.md` |
| order, spacing, color or font, or a new visual decision | `docs/reference/design-rules.md` |
| restructuring, or where something lives | `ARCHITECTURE.md`, `docs/explanation/host-and-pages.md`, `docs/reference/source-map.md` |
| a dependency or a license | `docs/reference/dependencies-and-licenses.md` |
| the installer, the MSIX, a release, updating | `docs/how-to/cut-a-release.md`, `docs/how-to/build-and-test-the-msix.md` |
| how notifications reach the notch | `docs/explanation/notification-delivery.md` |

## Logo

A **mark** is the logo without a square and a **tile** is the logo inside the rounded square. `assets/branding/` holds `tile-stable.*` (blue: executables, taskbar, installer and shortcuts of a stable build, and the settings sidebar and About card) and `tile-dev.*` (indigo: the same for a pre-release). The three `mark-*.png` files are archive and nothing uses them. The version picks the tile (a suffix such as `-alpha.1` means dev), chosen at build time by `scripts/release_version.rs`. The SVG files are the masters, and the PNG and `.ico` files come from the designer's set and are committed as supplied. The name and logo are not under the code license ([TRADEMARK.md](TRADEMARK.md)).

## Design rules

The "Foldline Design" canvas and `docs/reference/design-rules.md` set what the user sees and why. Follow them for every visual change. When the user decides something they don't cover, or a change would break one, say so and propose a rule with its reason for that doc, and write it in the same change once the user agrees.

## Debugging

`FOLDLINE_DEBUG=<file>` writes a timestamped log (Unix milliseconds). `NOTCH_PAGES=shelf,media` picks pages, `FOLDLINE_LYRICS=off` stops lyric lookups, and `FOLDLINE_SETTINGS_SOLID=1` gives the settings window an opaque frame. Settings are in `%APPDATA%\Foldline\settings.txt`, and every key is in `docs/reference/settings-keys.md`.
