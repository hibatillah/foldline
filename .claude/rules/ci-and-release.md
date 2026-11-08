---
paths:
  - ".github/**"
  - "Cargo.toml"
  - "crates/settings/Cargo.toml"
  - "installer/**"
  - "THIRD-PARTY-LICENSES.txt"
---

# CI and release

`docs/how-to/cut-a-release.md` describes the release workflow.

- `ci.yml` runs on pull requests into `master` only, with one job named `Check` (the required status check of branch protection); it never releases and has `contents: read`.
- `release.yml` runs on `v*` tag pushes only, and fails unless the tag, without any `-suffix` (a pre-release such as `v0.5.0-alpha.1`), equals `v` plus the workspace version in `Cargo.toml`, which every crate inherits.
- The workflow uses the MSVC toolchain (`stable-x86_64-pc-windows-msvc`) and the `--workspace` flag; the settings crate (GPUI) does not build with the GNU toolchain.
- Read the executable names from `crates/notch/Cargo.toml` and the `[[bin]]` of `crates/settings/Cargo.toml` in workflows; never write `notch` into them.
- The installer ships `notch-settings.exe` beside `notch.exe` and `THIRD-PARTY-LICENSES.txt`; regenerate the licenses file with `scripts/third_party_licenses.sh` after any `Cargo.lock` change.
- Keep the `AppId` in `installer/notch.iss` fixed; changing it hides earlier installs from the installer.
- Use the preinstalled `gh` CLI for releases rather than a third-party action.
- Give a workflow the least permission it needs (`contents: read`; `contents: write` only for releases).
- The release workflow's **Check the icons** step runs `scripts/check-icon.sh` on both executables: a tag with a suffix must carry the orange `tile-dev`, any other the blue `tile-stable`.
- `scripts/test-build.sh` is the local counterpart for a teammate's installer (`dist/`); it is not part of CI.
