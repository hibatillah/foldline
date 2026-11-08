---
paths:
  - "scripts/**"
---

# Scripts

- Write entry scripts as `.sh` for Git Bash and keep them thin: parse options, build, start.
- Put logic that needs Windows APIs (moving the mouse, reading process counters) in a Rust example: a `.rs` file in
  `crates/notch/examples/` (Cargo finds it without an entry), run with `cargo run --release -p notch --example <name>`. The project
  has no PowerShell files.
- Begin every script with `cd "$(dirname "$0")/.."` so it works from any directory.
- Redirect the output of a detached `cmd start` (`>/dev/null 2>&1 </dev/null`) so the caller's pipe can close.
- Asset generation (images) may be a Python file in `scripts/` run by a `.sh` wrapper through
  `uv run --no-project --with <package>` (`scripts/build-cities.sh`). Commit the generated output; the build and the
  checks never run Python.
- List every file a build reads beside `crates/` (`scripts/icon_resource.rs`, `scripts/release_version.rs`, `assets/branding/*`) in `scripts/watch.sh`'s `watched` array.
