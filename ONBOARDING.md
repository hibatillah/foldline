# Onboarding

This is the guide for people joining the Notch team. Notch is closed-source and takes no outside contributions; see [LICENSE](LICENSE). Read this once, then use [AGENTS.md](AGENTS.md) and [docs/README.md](docs/README.md) as the day-to-day index.

## What the project is

Notch is a Windows "dynamic island" written in Rust: a black pill at the top of the screen, drawn with Direct2D and no UI framework. Two programs ship together:

- `notch.exe` (`crates/notch/`) is the pill. It idles at 0 % CPU and stays under 50 MB of memory. Every change to it is held to that budget.
- `notch-settings.exe` (`crates/settings/`, built with GPUI) is the settings window that the gear in the open notch and the tray's **Settings** item start; settings are edited only there, and its About page shows the version and opens the licenses and the data files. `crates/store/` holds the settings-file code both programs share.

## Set up your machine

You need:

- Windows 10 or 11, 64-bit. The project is developed and tested on Windows 11.
- [Rust](https://rustup.rs) 1.88 or newer, and the MSVC Rust toolchain for the settings window: `rustup toolchain install stable-x86_64-pc-windows-msvc`.
- Visual Studio Build Tools with the **Desktop development with C++** workload (MSVC linker and Windows SDK).
- [Git for Windows](https://git-scm.com/download/win). The `scripts/*.sh` helpers run in Git Bash.
- [Inno Setup](https://jrsoftware.org/isinfo.php), only if you build the installer by hand. The release workflow installs it itself.
- [Node.js](https://nodejs.org) for `npx` (skills) and [CodeGraph](https://www.npmjs.com/package/@colbymchenry/codegraph) (code search).

Then, in the repository:

```
npm install -g @colbymchenry/codegraph    # once per machine
codegraph init                            # builds the index in .codegraph/ (git-ignored)
npx skills experimental_install           # restores the skills listed in skills-lock.json
```

`codegraph sync` catches the index up after edits made outside a Claude Code session.

### If a skill will not install

`skills-lock.json` pins each skill to its source. Sources move or disappear. When `npx skills experimental_install` fails on one skill:

1. Run `npx skills experimental_install --dry-run` to see which skills it would restore.
2. Remove the failing skill's entry from `skills-lock.json` and run the install again.
3. Remove the skill's name from the `skills:` list in the front matter of any `.claude/agents/*.md` file that uses it.
4. Tell the team before you commit the lock file, because the removal changes it for everyone.

Skills install into `.agents/skills/` and are linked into `.claude/skills/`. Both folders are git-ignored; only the lock file is committed.

## Build and run

```
cargo +stable-x86_64-pc-windows-msvc build --release                # target\x86_64-pc-windows-msvc\release\notch.exe
scripts/watch.sh                       # debug build, relaunched on every saved file
scripts/reload.sh                      # one release build; starts it and closes the old one
cargo +stable-x86_64-pc-windows-msvc build --release -p settings    # the settings window
```

Use the MSVC toolchain for every crate (the notch's text lookup, rust-i18n, needs it as GPUI does). Commands without `-p` or `--workspace` cover only the notch; add `--workspace` to include `crates/settings/`, `crates/store/` and `crates/places/`. [docs/how-to/build-run-and-debug.md](docs/how-to/build-run-and-debug.md) covers debugging, and `scripts/test-build.sh` makes an installer of your working tree for a teammate.

## How the code is organized

- `crates/notch/src/host/` is the notch itself: the window, hover, animation, input, the tray and the launcher of the settings window. It names no page. `Notch` is one struct whose methods live one job per file in `crates/notch/src/host/notch/`.
- `crates/notch/src/pages/` holds one folder per page (media, shelf, calendar, timer, todo, shalat, stats, weather, devices, clipboard) and `crates/notch/src/activities/` one file per pill message (volume, battery, notifications…). Each is listed in one registry line, so adding or removing one touches no other file.
- `crates/notch/src/gfx/` is the Direct2D and DirectWrite drawing toolkit and `crates/notch/src/ui/` the widgets and input helpers built on it; neither imports from `page`, `i18n`, `host`, `pages`, `activity` or `platform` (a test enforces it). `crates/notch/src/platform/` holds small Windows actions.
- Pages, activities and widgets draw and receive pointer input in device-independent units starting at zero; `crates/notch/src/host/notch/units.rs` is the only place that converts to device pixels. [docs/explanation/drawing-model.md](docs/explanation/drawing-model.md) gives the reasons.
- Dependencies point one way: pages and activities use `page`, `activity`, `gfx`, `ui` and `platform`; nothing in those knows a particular page; pages never use each other.

The layout rules (one job per file, about 400 lines, state in a small struct beside its logic) are in [docs/reference/source-map.md](docs/reference/source-map.md), and the reasons are in [docs/explanation/host-and-pages.md](docs/explanation/host-and-pages.md). [ARCHITECTURE.md](ARCHITECTURE.md) gives the overview of the programs, crates, files and services.

## Rules the code follows

- **Performance.** No polling. A timer or animation stops as soon as its output stops changing. Draw shapes into small cached images, not Direct2D geometry. See [docs/reference/performance-baseline.md](docs/reference/performance-baseline.md) for the numbers.
- **Design.** Order, spacing, color and font of anything visible follow [docs/reference/design-rules.md](docs/reference/design-rules.md). When you decide something it does not cover, propose a rule with its reason and add it in the same change.
- **Style.** Names are written in full (`present_track_ids`, not `ptids`). Comments say only what the code cannot: a reason, a unit, a Windows workaround. Pages and activities write sizes in device-independent units and do not multiply by a scale; only the host and `Page::draw_backdrop` work in device pixels.
- **Tests.** Logic that takes and returns plain values gets a unit test named as a sentence about behavior. Tests sit with the code they check. Drawing is checked with `cargo test snapshots -- --ignored`, then by looking at `target/snapshots`, at the display scales 1.0, 1.25 and 1.5 ([docs/how-to/test-and-snapshot.md](docs/how-to/test-and-snapshot.md)).
- **Docs.** When behavior a user can see changes, update `README.md` and the doc that covers it in the same change.

## Branches and pull requests

Until `v0.5.0` is released, changes may go straight to `master`. From that release on, everything goes through a pull
request that passes the **CI** check, yours included; see [docs/how-to/work-with-branches.md](docs/how-to/work-with-branches.md).

## Before you finish a change

```
cargo +stable-x86_64-pc-windows-msvc fmt --check
cargo +stable-x86_64-pc-windows-msvc clippy --workspace --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-msvc test --workspace
scripts/perf.sh          # after host, animation, timer or drawing changes
```

When you touched `crates/settings/` or `crates/store/`, run the first three as `cargo +stable-x86_64-pc-windows-msvc ... --workspace`. `scripts/perf.sh` moves the mouse for about three minutes and replaces the running notch; leave the mouse alone while it runs. [docs/how-to/test-and-snapshot.md](docs/how-to/test-and-snapshot.md) and [docs/how-to/run-the-perf-check.md](docs/how-to/run-the-perf-check.md) explain the gates.

## Working with Claude Code

[AGENTS.md](AGENTS.md) and `.claude/rules/` carry the project rules for the agent, and CodeGraph gives it the code index. Six subagents in `.claude/agents/` each own one area and load their own docs and skills:

| Agent | Owns |
| --- | --- |
| `notch-host` | `crates/notch/src/host/**`, `crates/notch/src/platform/**`: Win32 behavior, hover, timers, the tray, the header gear |
| `notch-pages` | `crates/notch/src/pages/**`, `crates/notch/src/activities/**`, `page.rs`, `activity.rs` |
| `notch-render` | `crates/notch/src/gfx/**`, `crates/notch/src/ui/**`: Direct2D and DirectWrite drawing |
| `settings-gpui` | `crates/settings/**`, `crates/store/**` |
| `perf-auditor` | read-only check of the performance budget |
| `docs-writer` | `docs/**` and `README.md`, in the Diátaxis structure |

The agent does not commit unless you ask. `/group-commit` commits the working tree grouped by scope (one commit for one scope), with `type(scope): message` subjects. The agent uses it for every commit, whether or not you name it. `/release <version|major|minor|patch|alpha|beta|rc|stable>` prepares a release locally and asks before it pushes; [docs/how-to/cut-a-release.md](docs/how-to/cut-a-release.md) describes the flow.

## Where to read next

| Doc | What it covers |
| --- | --- |
| [docs/how-to/build-run-and-debug.md](docs/how-to/build-run-and-debug.md) | requirements, build, run, debug |
| [docs/how-to/test-and-snapshot.md](docs/how-to/test-and-snapshot.md) | where the tests live, how to write one |
| [docs/how-to/add-a-page.md](docs/how-to/add-a-page.md) | write, register and remove a page |
| [docs/how-to/add-an-activity.md](docs/how-to/add-an-activity.md) | write, register and remove an activity |
| [docs/reference/settings-keys.md](docs/reference/settings-keys.md) | every setting, its default and its environment variable |
| [ARCHITECTURE.md](ARCHITECTURE.md) | the overview: programs, crates, files, services and terms |
| [docs/explanation/host-and-pages.md](docs/explanation/host-and-pages.md) | how the host, pages and activities work inside the notch, and why |
| [docs/how-to/cut-a-release.md](docs/how-to/cut-a-release.md) | the release workflow and the installer |
