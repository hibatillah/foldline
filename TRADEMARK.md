# The Foldline name and logo

The source code of Foldline is free software under the GNU General Public License, version 3 or later ([LICENSE](LICENSE)). You may use, study, modify and share it under that license.

The name **Foldline**, the Foldline logo, the tile and the other marks in `assets/branding/` (the *Foldline marks*) are not part of that license. As section 7(e) of the GNU GPL allows, no right to use them comes with the code.

- **You may** build Foldline for yourself, modify it, and redistribute the code or a build of it under the GNU GPL, with its source.
- **A copy you redistribute must not carry the Foldline marks**, changed or not. Give it another name and logo everywhere a person sees or a system registers them:
  - window titles and the About page
  - the executable names (`[[bin]]` in `crates/notch/Cargo.toml` and `crates/settings/Cargo.toml`)
  - the installer (`installer/foldline.iss`) and the package identity and display name (`installer/msix/AppxManifest.xml`)
  - the version information (`crates/notch/build.rs`, `crates/settings/build.rs`)
  - the texts in `crates/*/locales` and the images in `assets/branding/`
  - any store listing, website or message that offers it
- **You may say, truthfully,** that your software is based on Foldline or built from its source, and link to this repository, as long as that doesn't suggest the maintainer made, approved or supports it.
- The builds the maintainer publishes, which are the Microsoft Store listing of Foldline and the releases of this repository, are the official Foldline.

Questions about using the name: open an issue in this repository.
