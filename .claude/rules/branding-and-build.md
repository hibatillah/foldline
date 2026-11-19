---
paths:
  - "crates/notch/build.rs"
  - "crates/settings/build.rs"
  - "scripts/icon_resource.rs"
  - "assets/**"
  - "installer/**"
  - "crates/notch/src/host/icon.rs"
---

# Branding and build script

`docs/how-to/build-run-and-debug.md` (branding) and `docs/how-to/cut-a-release.md` describe the icon pipeline.

- `tile-stable` is the blue tile and `tile-dev` the indigo one. The SVG files are the masters; the PNG (256 px) and `.ico`
  files come from the designer's logo set and are committed as supplied (no export script). The settings sidebar and the About card
  show the tile of the build's channel (`crates/settings/src/brand.rs`). The three marks (`mark-cream.png`, `mark-charcoal.png`,
  `mark-orange.png`) are archive; nothing uses them. `AGENTS.md` defines the words mark and tile.
- A pre-release version (`1.0.0-alpha.1`, from `FOLDLINE_RELEASE_VERSION` or the latest pre-release tag) gets `tile-dev`, any
  other `tile-stable`. `scripts/release_version.rs` decides it for both build scripts; `installer/foldline.iss` decides it from
  `AppVersion` by the same rule. Never choose by a flag or a branch name.
- `crates/notch/build.rs` embeds the tile icon as `FOLDLINE_ICON` and `crates/settings/build.rs` as resource id 1 (the one GPUI
  loads for the window and taskbar). Both call `compile_icon_resource` in `scripts/icon_resource.rs`, which writes a resource
  script to `OUT_DIR` and runs the Windows SDK's `rc.exe` (found on `PATH` or under `Windows Kits/10`). Keep it dependency-free and have it fail with a message
  that names the missing tool.
- The same resource script carries a `VERSIONINFO` block from `release_version::version_info_resource`: Task Manager shows its `FileDescription` ("Foldline", "Foldline Settings"), so never ship an exe without one.
- Load the embedded icon through `host/icon.rs` by the resource name `FOLDLINE_ICON`; the installer and shortcuts reuse
  the executable's icon.
- `scripts/check-icon.sh` and the release workflow's **Check the icons** step prove a release executable embeds the tile of its channel.
