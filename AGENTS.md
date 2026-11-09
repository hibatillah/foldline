# Notch

A Windows "dynamic island" written in Rust: a black pill at the top of the screen, drawn with Direct2D, no UI
framework. `notch.exe` idles at 0 % CPU and stays under 50 MB of memory. Windows only.

The settings window is a second program, `notch-settings.exe` (`crates/settings/`, built with GPUI), that the gear in the open notch and the tray's
**Settings** item start. It has a higher footprint while open and is outside that budget. `crates/store/` holds the settings
file code both programs share. Settings are edited only in the settings window; the notch has no settings panel.

## Commands

Root commands cover only the notch (the workspace's default member). GPUI builds only with the MSVC host toolchain, so
run `crates/settings/` and `crates/store/` commands as `cargo +stable-x86_64-pc-windows-msvc ... -p settings` (or `--workspace`).

```
cargo build --release                     # target\x86_64-pc-windows-msvc\release\notch.exe
cargo test                                # unit tests, no window needed
cargo test snapshots -- --ignored         # writes pictures of the pages to target/snapshots
cargo fmt --check
cargo clippy --all-targets -- -D warnings
scripts/reload.sh [--debug]               # build, start the new build, close the old one (Git Bash)
scripts/watch.sh                          # debug build of the notch, relaunched on every saved source file (Git Bash)
scripts/test-build.sh                     # installer (zip without Inno Setup) of the current code in dist/ for a teammate (Git Bash)
scripts/perf.sh [--no-build]              # memory and CPU checks; moves the mouse for about three minutes
```

Builds and checks:

- Each agent or session that builds sets its own `CARGO_TARGET_DIR`, so a build never overwrites the `notch.exe` that
  runs (a Windows file lock) or another agent's artifacts. A target directory takes about 3 to 9 GB: delete the stale
  ones when the disk fills.
- A `tree-sitter` build that ends incomplete in a stale target directory is fixed by `cargo clean -p tree-sitter` in
  that directory and a new build.
- `cargo test snapshots -- --ignored` writes the pictures of the notch's pages; the settings window has no snapshots.
- The settings tests cannot pass while a `notch-settings.exe` window is open: its single-instance mutex makes the
  `single_instance` test fail. Close the window first.
- `scripts/test-build.sh` builds the current tree into `dist/` (git ignores it) as an installer, or a zip without Inno Setup.

## Find code

`.codegraph/` indexes this repository. Ask `codegraph_explore` (pass `projectPath`) for a symbol, a caller or a call
path before grepping; it returns the current source with line numbers. The index follows edits while the MCP server
runs; `codegraph sync` catches up after changes made outside a session.

## Rules

`.claude/rules/` holds the constraints, loaded by the paths they cover. Three always apply: `workflow.md` (checks before
done, no commits unless asked, never replace the running notch except through the scripts, specs and plans go to
`.superpowers/`), `performance.md` (no polling, memory budget) and `contributions.md` (closed source, no outside
contributions). `module-layout.md` sets how Rust files are cut (one job per file, about 400 lines) and applies to every
source file; `code-style.md` and `testing.md` cover the notch's Rust code and scripts.

Privacy: the clipboard page, when the user switches it on, keeps the last seven copies in memory only. Nothing is
written to disk or sent anywhere, and the history is dropped when the page is switched off or the notch quits. The
clipboard activity ("Copied") never reads what was copied. Keep both as they are, and keep every statement about them in
the docs true.

## Commits

Commit only when the user asks. When they do, in any wording, run `/group-commit` (the Skill tool, `group-commit`) for
it: never `git commit` by hand and never another commit skill. It makes one commit when the changes share a scope, a few
when they do not, and never pushes.

Push only when the user asks. Until the tag `v0.5.0` exists, a push to `master` is allowed on request. From then on every
change goes through a branch and a pull request (`docs/how-to/work-with-branches.md`): never push to `master`, and when
asked to, stop, tell the user it breaks the workflow, and offer a branch and a pull request instead. Check with
`git tag --list v0.5.0`.

## Subagents

**A change under an area's paths MUST be dispatched to that area's subagent through the Agent tool; do not edit it
inline.** The main session plans, orders the areas, runs commits and answers questions. Claude Code picks the agent from
its description; an agent without a subagent mechanism reads the matching file in `.claude/agents/` and follows it.

| Agent | Area |
| --- | --- |
| `notch-render` | `crates/notch/src/gfx`, `crates/notch/src/ui` |
| `notch-host` | `crates/notch/src/host`, `crates/notch/src/platform` |
| `notch-pages` | `crates/notch/src/pages`, `activities`, `page.rs`, `activity.rs`, `crates/places` |
| `settings-gpui` | `crates/settings`, `crates/store` |
| `perf-auditor` | read-only budget review after timers, drawing, threads or caching change |
| `docs-writer` | `docs/**`, `README.md`, `ONBOARDING.md` |

A task across areas goes in this order: `notch-render`, then `notch-host` and `notch-pages`, then `settings-gpui`, then
`perf-auditor`, then `docs-writer`. Give each agent a self-contained brief: the goal, the files or boards to read (design
boards are in `.superpowers/design/`) and what to report. Agents that build run one at a time per source tree, each with
its own `CARGO_TARGET_DIR`.

The agent files, `.claude/rules/` and this file belong to the main session; `docs-writer` does not edit them. After
editing an agent file run `claude plugin validate .claude/agents`: one unescaped `'` in a description (write `''` inside
a single-quoted one) stops that agent from loading.

## Code layout

The repository is a Cargo workspace: `crates/notch` (the pill), `crates/settings` (the settings window, built as
`notch-settings.exe`), `crates/store` (the shared settings file code) and `crates/places` (the city list and the shared
location), with the patched `salah` crate in
`vendor/salah`. The layout of the code is defined in `docs/explanation/architecture.md` (the reasons) and
`docs/reference/source-map.md` (where each module lives, and how a module is laid out); keep new code to it.

## Docs

`docs/README.md` indexes every document by type. Open the one that matches the work in front of you before you start:

| Work | Read |
| --- | --- |
| a page: first one, or looking up a hook or rule | `docs/how-to/add-a-page.md`, `docs/reference/page-api.md` |
| an activity (volume, Caps Lock, notification banner) | `docs/how-to/add-an-activity.md`, `docs/reference/activities.md` |
| a setting, a key, a control in the settings window | `docs/how-to/add-a-setting.md`, `docs/reference/settings-keys.md`, `docs/reference/settings-window.md` |
| hiding Windows' own volume flyout or notification banners | `docs/how-to/hide-windows-popups.md` |
| switching pages or activities off, or removing them | `docs/how-to/switch-off-or-remove.md` |
| build, run, debug, a page that does not show | `docs/how-to/build-run-and-debug.md` |
| a test, a snapshot, the manual checklist | `docs/how-to/test-and-snapshot.md`, `docs/reference/test-map.md` |
| `perf.sh`, a CPU or memory number | `docs/how-to/run-the-perf-check.md`, `docs/reference/performance-baseline.md`, `docs/explanation/performance-budget.md` |
| padding, the header, the open and collapsed shapes | `docs/reference/layout.md` |
| order, spacing, color or font of anything visible, or a new visual decision | `docs/reference/design-rules.md` |
| restructuring, or where something lives | `docs/explanation/architecture.md`, `docs/reference/source-map.md` |
| a dependency, a license | `docs/reference/dependencies-and-licenses.md` |
| the installer, a release, updating | `docs/how-to/cut-a-release.md` |

## Logo

Two forms: a **mark** is the logo without a square, a **tile** is the logo inside the rounded square. `assets/branding/`
holds the logo files and nothing else of the logo:

| File | What it is | Used for |
| --- | --- | --- |
| `tile-stable.svg` / `.png` (256 px) / `.ico` | blue tile | executables, taskbar, installer and shortcuts of a stable build; the settings sidebar and About card of a stable build |
| `tile-dev.svg` / `.png` / `.ico` | orange tile | the same for a pre-release build (alpha, beta, rc) |
| `mark-cream.png`, `mark-charcoal.png`, `mark-orange.png` | marks without a square | archive only; nothing uses them |

The settings sidebar and the About card show the tile of the build's channel (`crates/settings/src/brand.rs`), in both themes.
The channel comes from the version (a suffix such as `-alpha.1` means dev), chosen at build time by
`scripts/release_version.rs`. There is no nightly channel. The SVG files are the masters; the PNG and `.ico` files come from the
designer's logo set and are committed as supplied. No script exports them.

## Design rules

The "Notch Design" canvas and `docs/reference/design-rules.md` set what the user sees and why. Follow them for every visual
change. When the user decides something they do not cover (an order, a gap, a color, a font), or a change would break
one, say so and propose a rule with its reason for that doc; write it in the same change once the user agrees.

## Debugging

`NOTCH_DEBUG=<file>` writes a timestamped log (Unix milliseconds). `NOTCH_PAGES=shelf,media` picks pages,
`NOTCH_LYRICS=off` stops lyric lookups, `NOTCH_TOP=<px>` moves the pill down, and `NOTCH_SETTINGS_SOLID=1` gives the
settings window an opaque frame instead of the blurred translucent one. Settings are in
`%APPDATA%\notch\settings.txt`; every key is in `docs/reference/settings-keys.md`.
