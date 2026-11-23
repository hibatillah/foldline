---
description: Prepare a release locally (version commit and tag), then ask before pushing so GitHub builds the installer
argument-hint: <version | major | minor | patch | alpha | beta | rc | stable> [alpha | beta | rc]
disable-model-invocation: true
---

Cut a release for this repository. Arguments: `$ARGUMENTS`. The release workflow and the tag rules are in
`docs/how-to/cut-a-release.md` ("Share a pre-release"); read it first.

## 1. Find where we are

- `git fetch --tags origin`, then list tags that start with `v` and sort them by SemVer yourself (a pre-release such as
  `v1.0.0-beta.1` is lower than `v1.0.0`, and `alpha < beta < rc`; plain `git tag --sort` gets this wrong).
- `latest` is the highest tag. `latest_final` is the highest tag without a suffix.
- The workspace version in the root `Cargo.toml` (`[workspace.package]`) is the base number (`1.0.0`); every crate inherits it.

## 2. Work out the next tag from the arguments

| Arguments | Next tag |
| --- | --- |
| an explicit version (`0.6.0`, `v0.6.0-beta.2`) | exactly that, with a `v` in front |
| `major`, `minor`, `patch` | bump `latest_final`, then reset the lower numbers; a final release |
| `major`, `minor` or `patch` followed by `alpha`, `beta` or `rc` | the same bump, starting that stage: `minor beta` gives `v0.6.0-beta.1` |
| `alpha`, `beta` or `rc` alone, while `latest` is a pre-release of base X | same stage: next number (`alpha.1` to `alpha.2`); a later stage: that stage at `.1` (`alpha.2` to `beta.1`) |
| `alpha`, `beta` or `rc` alone, while `latest` is final or an earlier stage is requested | stop and ask which bump to start from (or, for an earlier stage, whether they meant it) |
| `stable` or `final`, while `latest` is a pre-release of base X | `vX`, the suffix dropped |

Refuse and say why when the next tag is not higher than `latest` by SemVer, when the tag already exists, or when the
arguments match no row. Pre-release numbers count from 1 and are written `alpha.2`, never `alpha2`.

## 3. Check before touching anything

- The branch is `master` and equals `origin/master`; the working tree has no uncommitted changes.
- No CI runs on pushes, so run the checks here with the MSVC toolchain: `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` (use
  `cargo +stable-x86_64-pc-windows-msvc`). Stop and show the output if one fails.

State the plan in one line (`v1.0.0-alpha.1 -> v1.0.0-beta.1`, whether the version files change) and prepare it.

## 4. Prepare it locally, do not push

1. If the base number changes, set `version` under `[workspace.package]` in the root `Cargo.toml` to it (plain,
   no suffix), run `cargo update --workspace --offline` so `Cargo.lock` follows, and commit as "Set the version to X"
   with the attribution lines from the session reminder. A new stage of the same base needs no commit.
2. Create the tag on `HEAD`: `git tag <tag>`.
3. Push nothing. The push of the commit and of the tag is the user's decision.

## 5. Show it and ask

First show the release notes the tag will get: `bash scripts/release-notes.sh <tag> master`. They list the pull requests merged
into `origin/master` since the previous release (assignee credited, grouped by type), or the commit titles when there are
none; commits not pushed yet are not in them.

Show what now exists locally: the tag and the commit it points to (`git log --oneline -1 <tag>`), whether it is a
pre-release, the version commit if one was made, and what the pushes will do (the tag push starts `release.yml`, which
builds the installer and publishes a GitHub release of the same name). The notes end with a Contributors section for outside contributors only. Then ask the user whether to push. On yes, run
`git push origin master` when a version commit was made, then `git push origin <tag>`, and show the run it started with
`gh run list --workflow release.yml --limit 1`. On no, leave everything as it is and mention how to undo it
(`git tag -d <tag>`, and `git reset --hard HEAD~1` only if the version commit was made and the user wants it gone).
Do not create or edit releases by hand: the workflow publishes them.
