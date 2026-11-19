# Cut a release

> **How-to.** Read when: you publish a version, regenerate the license list, or explain how users update. Skip if: you only build locally; see [build, run and debug](build-run-and-debug.md).

The Microsoft Store package is built separately: see [build and test the MSIX](build-and-test-the-msix.md).

Two GitHub Actions workflows exist. `release.yml` does the release. `ci.yml` only checks pull requests into `master`; nothing runs on a push to `master`, so run the checks yourself before
you tag a commit that did not come through a pull request.

| Workflow | Starts when | Does |
| --- | --- | --- |
| `ci.yml` | a pull request into `master` | `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`; the **Check** job is what a pull request must pass to merge |
| `release.yml` | a push of a tag that starts with `v` | `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, then builds and publishes a GitHub release (a pre-release when the tag has a suffix, such as `v1.0.0-alpha.1`; see [share a pre-release](#share-a-pre-release)) |

It installs the MSVC toolchain, because the settings crate (GPUI) does not build with the GNU one. Locally use `cargo +stable-x86_64-pc-windows-msvc` for the settings crate.

`scripts/perf.sh` is not in the workflow: it moves the mouse and needs a real, unlocked desktop. Run it yourself before a release.

## Release notes

The release notes are written for you when the tag is pushed (`scripts/release-notes.sh`, called by `release.yml`):

- They list the pull requests merged since the previous release, grouped by `type:` label as `.github/release.yml` says
  (Breaking changes, Features, Fixes, Performance and refactors, Other changes; chore, docs, test, style, ci and build
  are left out). Each line is the pull request's title, credited to its **assignee**, or to its author when nobody is
  assigned, with a link to the pull request.
- The `type:` label comes from the pull request's title: the CI workflow sets it from the `type(scope):` prefix.
- A stable tag covers everything since the last stable tag, so `v1.0.0` collects the changes of its pre-releases; a
  pre-release covers what came after the tag before it (`scripts/previous-tag.sh`).
- They end with a **Contributors** section that thanks the people from outside the team (GitHub `author_association` other than owner, member or collaborator) who opened a merged pull request; maintainers and teammates are not listed, and with no outside author there is no section.
- Changes pushed straight to `master` are not pull requests, so a release made only of them lists commit titles instead.
- To see the notes before you push: `bash scripts/release-notes.sh v1.0.0 master` (it reads what is on `origin`).

Assign each pull request to the person who should be credited when you open it.

## Publish a release

1. Set `version` under `[workspace.package]` in `Cargo.toml` (for example `0.2.0`; every crate inherits it) and commit it to the default branch.
2. Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` (MSVC toolchain); the workflow runs them again and a failure there leaves a tag to delete.
3. Tag the commit and push the tag:

   ```
   git tag v0.2.0
   git push origin v0.2.0
   ```

The release workflow stops at once if the tag does not match the workspace version in `Cargo.toml`, so a tag can never publish a
build under the wrong number. It then produces a release named after the tag with:

- `<name>-v0.2.0-windows-x64-setup.exe`: the installer, built from `installer/foldline.iss` with Inno Setup. It installs
  the notch and the settings program (`foldline-settings.exe`, which the gear in the open header and the tray menu start from the notch's folder) for the
  current user (no administrator rights), with the project license, the license of the font built into the notch
  (`Geist-LICENSE.txt`) and `THIRD-PARTY-LICENSES.txt` beside them and adds a Start menu entry. A stable version also offers the
  "Start Foldline when Windows starts" task; a dev version does not (see [Updating](#updating))
- `<name>-v0.2.0-windows-x64-setup.exe.sha256`: its SHA-256 checksum
- the release notes described in [Release notes](#release-notes)

`<name>` is the `[[bin]]` name in `crates/notch/Cargo.toml` (`foldline`), so renaming the executable renames the installer with no workflow change.

## Publish to the Microsoft Store

The `store` job of `release.yml` runs after the release job for a plain version tag (no suffix) and sends the same code to
the Store as an update of the live listing. It builds the MSIX with `scripts/build-msix.sh`, keeps it for 30 days as the
workflow artifact `msix`, then runs `msstore reconfigure` and `msstore publish` from the Microsoft Store Developer CLI. The
release page carries the installer only. The job is skipped for a pre-release tag and while the repository variable
`STORE_PRODUCT_ID` does not exist.

The CLI only updates an app that is already live, and only a free one, so set it up once after the first publication:

1. Make the first submission in Partner Center (the CLI cannot create the app or its first publication).
2. Associate a Microsoft Entra tenant with the Partner Center account, register an application in Microsoft Entra ID,
   and add it in Partner Center (Account settings, User management, **Microsoft Entra applications**) with the Manager role.
3. In the repository settings (Settings, Secrets and variables, Actions) add these secrets, found as follows:

   | Name | Where it comes from |
   | --- | --- |
   | `AZURE_AD_TENANT_ID` | entra.microsoft.com, Identity, Overview, **Tenant ID** |
   | `AZURE_AD_APPLICATION_CLIENT_ID` | Entra, Identity, Applications, App registrations, your app, **Application (client) ID** |
   | `AZURE_AD_APPLICATION_SECRET` | the same app, Certificates & secrets, new client secret; its value is shown once |
   | `SELLER_ID` | Partner Center, Account settings, the publisher (seller) ID |

   Add the variable `STORE_PRODUCT_ID` (the Store product ID of the listing, shown in Partner Center). Set the variable
   `STORE_NO_COMMIT` to `true` to upload a draft and press Submit in Partner Center yourself.
4. Create the environment `microsoft-store` and require a reviewer, so each submission waits for an approval.

Store users receive an update through the Store; someone who installed with the installer runs the newer installer. The
job has not run against a live listing yet. The Store version is `major.minor.patch` plus `.0` ([version rule](../explanation/msix-packaging.md#version-rule)).

## Third-party licenses

`THIRD-PARTY-LICENSES.txt` holds the license texts copied from the sources of every crate linked into the two
executables (the settings program links Apache-2.0 GPUI crates, which require their license text to ship). It is
committed and generated: run `scripts/third_party_licenses.sh` after any `Cargo.lock` change and commit the result. The
script reads `cargo metadata` for the MSVC target, follows normal dependencies only, and groups crates by identical
text. Crates whose sources contain no license file are listed at the end of the file with their declared license.

## Updating

A user updates by downloading the newer `-setup.exe` and running it. What happens depends on the version in the
installer; `installer/foldline.iss` decides it from `AppVersion`: a version with a `-` suffix (`alpha`, `beta`, `rc`) is a
dev build, any other is stable.

| | Stable (`1.0.1`) | Dev (`1.1.0-alpha.1`) |
| --- | --- | --- |
| `AppId` | fixed | one per version (the fixed id plus `-dev-<version>`) |
| Folder | `%LOCALAPPDATA%\Programs\Foldline` | `%LOCALAPPDATA%\Programs\Foldline <version>` |
| Start menu and Add or remove programs | `Foldline` | `Foldline <version>` |
| Asks before replacing | the installed stable copy, saying which versions are involved (`1.0.1` replaces `1.0.0`) | never; it replaces no other copy, stable or dev |
| "Start Foldline when Windows starts" task and `Run` entry | offered | none |

After a replacement the installer closes the running notch, replaces it in place and starts it again. Several dev builds
can be installed side by side. Nothing in the notch checks for new versions or uses the network for it.

Never change the stable `AppId` (`AppGuid` in `installer/foldline.iss`), or the installer stops recognising earlier stable
installs.

Copies share state:

- All copies read and write `%APPDATA%\Foldline\settings.txt`, and so does the Store version.
- The settings window is single-instance across copies.
- The settings window's **Start with Windows** switch (General page) writes the `Foldline` value of the `Run` key (`crates/settings/src/startup/`; an MSIX build switches its `FoldlineStartup` startup task instead), the same value the stable copy's installer task writes
  (`installer/foldline.iss`). A dev copy writes one only when the user switches the toggle. A `Notch` value from an earlier build still counts as on and is removed when the switch changes.
- A dev build installed by an older installer sits under the stable `AppId`, so a stable installer still offers to replace it.

## Notes

- The icon is a tile, chosen by channel: `assets/branding/tile-stable` (blue) for a stable version and `tile-dev` (indigo) for a pre-release. Replace the `.svg`, `.png` and `.ico` of a tile together
  when the logo changes; no script exports them. Cargo embeds the `.ico` in each executable; Inno Setup uses the same tile for the setup
  executable and its PNG for the wizard image. The Start menu shortcut and installed-app entry use the icon embedded in `foldline.exe`. See [App icon](build-run-and-debug.md#app-icon).
- The installer is not code-signed, so Windows SmartScreen may warn people who download it. Free signing for open
  source projects exists (SignPath Foundation); it needs an application from the project owner, and signing is then one
  more step in `release.yml`.
- The installer is built only by the release workflow. To build it by hand, install Inno Setup and run
  `ISCC /DAppVersion=0.2.0 /DExeName=foldline.exe /DSourceExe=<release foldline.exe> /DSettingsExeName=foldline-settings.exe /DSourceSettingsExe=<release foldline-settings.exe> installer\foldline.iss`; the result
  goes to `installer\Output` unless `/O` says otherwise (the workflow uses `dist`).
- The workflow installs the latest stable Rust (MSVC host), so a new Rust release can add a Clippy warning that fails the release. Fix the
  warning; do not pin an old compiler unless a release is urgent.
- To delete a bad release, delete the release and its tag on GitHub (or `git push --delete origin <tag>`), fix, and tag
  again.

## Share a pre-release

For a build a teammate should try before the real release, tag it with a SemVer suffix:

1. Set `version` under `[workspace.package]` in `Cargo.toml` to the coming number (for example `1.0.0`), commit and push.
2. `git tag v1.0.0-alpha.1 && git push origin v1.0.0-alpha.1`

The tag is `v<crate version>-<stage>.<number>`: `alpha` (rough, a few testers), `beta` (feature-complete, still tested),
`rc` (release candidate), then the plain `v1.0.0`. SemVer orders them `alpha < beta < rc < final` and compares the number
as a number, so write `alpha.2`, not `alpha2`. The crate version stays `1.0.0` for the whole series.

`release.yml` checks only the part of the tag before the first `-` against the crate version, builds the same installer
and publishes the release as a pre-release, so it is not shown as the latest one. The installer carries the whole tag
version (`1.0.0-alpha.1`), which its Start menu entry, its folder and Add or remove programs show (`Foldline 1.0.0-alpha.1`); it installs beside every other copy and never asks to replace one (see [Updating](#updating)). The settings window's sidebar shows it
too: the Build step passes it as `FOLDLINE_RELEASE_VERSION` (the tag without the `v`), and a local build without it shows the latest pre-release tag of the current version series (`crates/settings/build.rs` reads it from `git describe`), or the plain crate version when there is none. The executables, the installer and its wizard image carry the indigo `tile-dev` icon instead of the blue `tile-stable` one. The **Check the icons** step of `release.yml` (`scripts/check-icon.sh`) fails the release before packaging when an executable embeds the wrong tile. Delete the pre-release and its tag when it is no longer needed.

In Claude Code, `/release <version | major | minor | patch | alpha | beta | rc | stable>` does these steps up to the push: it works out the next tag from the latest one (`/release beta` after `v1.0.0-alpha.2` gives `v1.0.0-beta.1`; `/release minor beta` gives the first beta of the next minor), checks that `master` is clean and green, sets the crate versions when the base number changes, commits and tags locally, shows the result and asks before pushing the commit and the tag.

For code that is not committed, run `scripts/test-build.sh` (it passes the pre-release version to the build when the latest local tag says so, so the icon matches): it builds the working tree into `dist/` (an installer with
Inno Setup installed, otherwise a zip) and needs no GitHub.
