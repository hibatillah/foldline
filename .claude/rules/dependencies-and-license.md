---
paths:
  - "Cargo.toml"
  - "Cargo.lock"
  - "LICENSE"
  - "crates/settings/Cargo.toml"
  - "crates/store/Cargo.toml"
---

# Dependencies and license

- The project is open source (GPL-3.0-or-later). Every dependency must have a license compatible with GPLv3 distribution: permissive
  licenses (MIT, Apache-2.0, BSD, ISC, Zlib, Unicode, CC0; Apache-2.0 fits GPLv3 only, so no GPL-2.0-only crate). The one MPL-2.0 crate (`option-ext`, unmodified) is
  accepted because its source notice ships in `THIRD-PARTY-LICENSES.txt`.
- A new data set, font or icon needs its license checked the same way, and its credit added to `scripts/third_party_extra.txt`.
- GPUI and the crates it brings are allowed in `crates/settings/` only. `foldline.exe` and `crates/store/` stay free of a UI framework.
- After any change to `Cargo.lock`, run `cargo license` (install with `cargo install cargo-license --locked`).
- After any change to `Cargo.lock`, run `scripts/third_party_licenses.sh` and commit `THIRD-PARTY-LICENSES.txt` (the installer ships it).
- Ask before adding a dependency, and before accepting a GPL, LGPL, AGPL or MPL one. The notch is small on purpose.
