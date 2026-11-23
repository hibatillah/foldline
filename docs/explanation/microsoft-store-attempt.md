# The Microsoft Store attempt

> **Explanation.** Read when: you wonder why Foldline is not on the Microsoft Store, or you plan to try the Store again. Skip if: you build or test the MSIX (see [build and test the MSIX](../how-to/build-and-test-the-msix.md)) or cut a release (see [cut a release](../how-to/cut-a-release.md)).

## Status

The Microsoft Store release was dropped (paused 2026-11-21). On 2026-11-22 the Store listing and the `store` job of `release.yml` were removed. Foldline ships only as GitHub releases: the Inno Setup installer and the exe. The MSIX files are unused but kept in the repository so a later attempt can reuse them: `installer/msix/`, `scripts/build-msix.sh` and `crates/settings/src/startup/packaged.rs`.

## What was done

- Partner Center product reserved: name `foldline`, product ID `9NGTV96SZDS7`, package identity `Hibatillah.foldline` (the `Identity` in `installer/msix/AppxManifest.xml`).
- An unsigned MSIX built with `scripts/build-msix.sh` as `1.0.0.0` (workspace version `1.0.0` plus `.0`, see [the version rule](msix-packaging.md#version-rule)).
- The Store listing, the properties and the privacy policy (entered as text in Properties) filled in.
- Package uploaded and validated, then submitted for certification on 2026-11-20. The maintainer pressed Submit by hand: browser automation clicks on the Submit button and the section links had no effect (cause unconfirmed).

## Result

Certification finished on 2026-11-20 with the status "Attention needed". The only failure was policy 10.6.3 Capabilities: the request to use the restricted capability `unvirtualizedResources` was reviewed and denied based on the information provided.

Microsoft's note: remove the capability and resubmit, or request reconsideration with a new submission that has new or expanded information in the business justification, including a valid support contact or developer website URL. Without new information the same outcome is likely.

The report did not list `runFullTrust` as a failure, so it appears accepted (not confirmed beyond that).

## Why the capability was needed

**Hide Windows notification banners** (`notch.hide_windows_notifications`, on by default) writes `ShowBanner` under `HKCU\Software\Microsoft\Windows\CurrentVersion\Notifications\Settings\<app id>`, which Windows itself reads ([how it works](../how-to/hide-windows-popups.md#notification-banners)). In a packaged app, `HKCU` writes go to a private copy unless the manifest sets `desktop6:RegistryWriteVirtualization` to `disabled`, and that setting requires `unvirtualizedResources` ([Microsoft Learn, flexible virtualization](https://learn.microsoft.com/en-us/windows/msix/desktop/flexible-virtualization); the install test is in [MSIX packaging](msix-packaging.md#what-the-install-test-found)).

Without the capability the switch would change nothing, and the Windows banner and the notch's own would both show. Start with Windows is not affected: packaged builds use the manifest startup task.

## Alternatives checked (2026-11-21)

- No documented way to hide the banners without the registry write.
- `ToastEnabled=0` had no effect (see [hide Windows' own popups](../how-to/hide-windows-popups.md#notification-banners)).
- Emptying the window region of the banner window, as the notch does for the volume flyout, was not found documented and was never tested. It would rely on undocumented behavior of a Windows system process.

## Why GitHub only

- Foldline sells nothing and is open source.
- The Store needs a support site and a public privacy-policy page. Neither exists yet: there is no landing page.
- A Store build without banner hiding is a worse product than the exe.

The cost of GitHub only:

- The exe and the installer are unsigned, so Windows SmartScreen shows "Windows protected your PC". Users click **More info**, then **Run anyway**.
- There is no Store auto-update: people download a new release, or click **Check for updates** on the About page, which only compares versions and opens the release page.

## If trying again

Needed:

- A landing page or a support URL.
- A public privacy-policy URL.
- Either a business justification text for `unvirtualizedResources` (that the switch is opt-in is not new information by itself; Microsoft asked for a support contact or a developer website), or a Store build without the capability and with the banner switch hidden (a change in `crates/settings`, through the `settings-gpui` agent).
- A package version the Store accepts. The workspace version is now `0.6.0` (Foldline continues with minor bumps after `v0.5.0`), so `scripts/build-msix.sh` would build `0.6.0.0`, which the Store refuses because the first number cannot be 0 ([version rule](msix-packaging.md#version-rule)). Either add a mapping to the script, or build once the workspace version reaches `1.0.0` or later. `1.0.0.0` was already submitted, so a repeat may be refused: the next accepted number would be higher, such as `1.0.1.0`.

Lessons from Partner Center:

- The full package name must be unique, so a package marked for removal blocks a re-upload until the removal is saved.
- A removal cannot be saved while the package list is empty with validation errors showing. Reloading the page clears them.
- Restricted capabilities give a validation warning and need approval.
- Certification took less than a day here.

## Open items

Items never verified in a package are listed once in [build and test the MSIX](../how-to/build-and-test-the-msix.md#what-is-not-verified-yet).
