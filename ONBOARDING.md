# Onboarding

For anyone working on Foldline, team or outside contributor. Foldline is open source under the GNU GPL version 3 or later ([LICENSE](LICENSE)) and takes pull requests under the [contributor agreement](CONTRIBUTING.md). Read this once, then use [AGENTS.md](AGENTS.md) and [docs/README.md](docs/README.md) as the day-to-day index.

## What the project is

Foldline is a Windows "dynamic island" written in Rust: a black pill at the top of the screen, drawn with Direct2D and no UI framework. Two programs ship together:

- `foldline.exe` (`crates/notch/`) is the pill. It idles at 0 % CPU and stays under 50 MB of memory, and every change to it is held to that budget.
- `foldline-settings.exe` (`crates/settings/`, built with GPUI) is the settings window. The gear in the open notch and the tray's **Settings** item start it, and settings are edited only there. `crates/store/` holds the settings-file code both programs share.

## Set up

You need:

- Windows 10 or 11, 64-bit (developed on Windows 11).
- [Rust](https://rustup.rs) 1.88 or newer, plus the MSVC toolchain: `rustup toolchain install stable-x86_64-pc-windows-msvc`. Every crate builds with it, because GPUI and the notch's text lookup (rust-i18n) need it.
- Visual Studio Build Tools with the **Desktop development with C++** workload.
- [Git for Windows](https://git-scm.com/download/win). The `scripts/*.sh` helpers run in Git Bash.
- [Inno Setup](https://jrsoftware.org/isinfo.php), only to build the installer by hand.
- [Node.js](https://nodejs.org), for the agent skills and [CodeGraph](https://www.npmjs.com/package/@colbymchenry/codegraph) (code search).

Then, in the repository:

```
npm install -g @colbymchenry/codegraph    # once per machine
codegraph init                            # builds the index in .codegraph/ (git-ignored)
npx skills experimental_install           # restores the skills in skills-lock.json
```

`codegraph sync` catches the index up after edits made outside a Claude Code session. If one skill fails to install, run the install with `--dry-run`, remove that skill's entry from `skills-lock.json` and its name from the `skills:` list in `.claude/agents/*.md`, and tell the team before you commit the lock file.

## Build and run

```
cargo +stable-x86_64-pc-windows-msvc build --release    # target\x86_64-pc-windows-msvc\release\foldline.exe
scripts/watch.sh                                        # debug build, relaunched on every saved file
scripts/reload.sh                                       # release build; starts it and closes the old one
```

Commands without `-p` or `--workspace` cover only the notch. [Build, run and debug](docs/how-to/build-run-and-debug.md) has the rest, and `scripts/test-build.sh` makes an installer of your working tree for a teammate.

## How the code is organized

- `crates/notch/src/host/` is the notch itself: the window, opening and closing, the pin, animation, input, the tray and the launcher of the settings window. It names no page.
- `pages/` holds one folder per page and `activities/` one file per pill message. Each is one line in a registry, so adding or removing one touches no other file.
- `gfx/` is the Direct2D and DirectWrite toolkit, `ui/` the widgets on top of it, and `platform/` small Windows actions.
- Pages, activities and widgets draw in device-independent units. Only `host/notch/units.rs` converts to device pixels ([drawing model](docs/explanation/drawing-model.md)).
- Dependencies point one way. Pages never use each other, and nothing below the host knows a particular page.

The layout rules are in [source map](docs/reference/source-map.md), the reasons in [host and pages](docs/explanation/host-and-pages.md), and the overview in [ARCHITECTURE.md](ARCHITECTURE.md).

## Rules the code follows

- **Performance.** No polling. A timer or animation stops when its output stops changing. Draw shapes into small cached images, not Direct2D geometry. Numbers are in [performance baseline](docs/reference/performance-baseline.md).
- **Design.** Order, spacing, color and font follow [design rules](docs/reference/design-rules.md). When you decide something they don't cover, add a rule with its reason in the same change.
- **Style.** Write names in full (`present_track_ids`, not `ptids`). Comments say only what the code can't: a reason, a unit, a Windows workaround.
- **Tests.** Logic that takes and returns plain values gets a unit test named as a sentence about behavior. Drawing is checked with `cargo test snapshots -- --ignored` at display scales 1.0, 1.25 and 1.5 ([test and snapshot](docs/how-to/test-and-snapshot.md)).
- **Docs.** When a user can see a change, update `README.md` and the doc that covers it in the same change.

## Branches and pull requests

Everything goes through a pull request that passes the **Check** job ([CONTRIBUTING.md](CONTRIBUTING.md)).

## Before you finish a change

```
cargo +stable-x86_64-pc-windows-msvc fmt --check
cargo +stable-x86_64-pc-windows-msvc clippy --workspace --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-msvc test --workspace
scripts/perf.sh          # after host, animation, timer or drawing changes
```

`scripts/perf.sh` moves the mouse for about three minutes and replaces the running notch, so leave the mouse alone while it runs ([run the perf check](docs/how-to/run-the-perf-check.md)).

## Working with Claude Code

[AGENTS.md](AGENTS.md) and `.claude/rules/` carry the project rules for the agent, and CodeGraph gives it the code index. Six subagents in `.claude/agents/` each own one area: `notch-host`, `notch-pages`, `notch-render`, `settings-gpui`, `perf-auditor` (read-only) and `docs-writer`. The agent commits only when asked, and always with `/group-commit`. `/release` prepares a release locally and asks before it pushes ([cut a release](docs/how-to/cut-a-release.md)).
