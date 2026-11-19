# Dependencies and licenses

> **Reference.** Read when: you add, update or audit a dependency. Skip if: you do not touch `Cargo.toml` or `Cargo.lock`.

The project is licensed GPL-3.0-or-later (see `LICENSE`), so every dependency must allow redistribution under it with its notices; Apache-2.0 fits GPL version 3 only, so no GPL-2.0-only crate is allowed. Audit of 2026-11-03 (`cargo license`, Windows target):

- `foldline.exe` links 22 packages, all `MIT`, `Apache-2.0` or both; `unicode-ident` adds Unicode-3.0.
- `foldline-settings.exe` (GPUI, pinned `gpui-component` `=0.5.1`) links 552 packages. Almost all are permissive: MIT, Apache-2.0, BSD, ISC, Zlib, 0BSD, Unicode-3.0, CC0 and Unlicense.
- One weak-copyleft crate: `option-ext` 0.2.0 (MPL-2.0), reached through `gpui-component-assets`, `rust-embed` and `dirs`. Unmodified, it may be linked if the MPL text ships and recipients are told where its source is; both are in `THIRD-PARTY-LICENSES.txt`. Modifying its files would require publishing the changed files. It disappears if the settings window does.
- No GPL, LGPL or AGPL crate was in the build at the audit.
- The package counts above predate the current `Cargo.lock` (for example `rust-i18n` 3.1.5, MIT, in both programs, with its macro and support crates). The current counts and the licenses of the transitive crates are not confirmed until `cargo license` is run again.

Permissive licenses still require their notices: the license texts ship in `THIRD-PARTY-LICENSES.txt` ([cut a release](../how-to/cut-a-release.md#third-party-licenses)), which also carries the credits that are not crates (`scripts/third_party_extra.txt`: GeoNames CC BY 4.0, the Lucide icons (the settings window bundles its own as SVG files in `crates/settings/assets/icons`; the names are listed in that file), the Geist font, the MPL source notice). Re-check after changing `Cargo.lock`:

```
cargo install cargo-license --locked    # once
cargo license
```

This groups the packages by license. Do not add a dependency with a copyleft license (GPL, LGPL, AGPL, MPL)
without checking what it requires. The notch is small on purpose, so prefer not to add dependencies at all.

## Vendored: salah

`vendor/salah` is `salah` 0.7.6 (MIT) with one change, listed in `vendor/salah/VENDORED.md`; the root `Cargo.toml` points `[patch.crates-io]` at it and excludes it from the workspace. Upstream panics when a time falls in the last half minute of a UTC day, and the release build aborts on panic, so the notch would close and close again on every start that day. To update it, copy the new release over `vendor/salah`, reapply the change to `SolarTime::setting_hour`, and run `cargo test shalat` (it scans every latitude, many dates and several longitudes for panics).
