# Work with branches and pull requests

> **How-to.** Read when: you start a change, review one, merge, or are about to push. Skip if: you only build locally; see [build, run and debug](build-run-and-debug.md).

## When it applies

- **Until `v0.5.0` is released** (the tag `v0.5.0` does not exist yet), changes may go straight to `master`.
- **From the `v0.5.0` release on**, every change goes through a pull request, the owner's included. Nobody pushes to
  `master` directly. GitHub cannot be made to enforce this on every plan, so it is a rule we keep ourselves.

Pushing to `master` after that point is a mistake to catch, not a shortcut: stop and open a pull request instead.
An AI agent never pushes to `master` on its own; when asked to after the release, it says so and offers a branch and a
pull request.

## Make a change

1. Start from an up-to-date `master` and make a branch named `type/short-name`, with the type of the commit, for
   example `feat/weather-page` or `fix/shelf-drop`.
2. Commit with `/group-commit` (see [AGENTS.md](../../AGENTS.md)).
3. Push the branch and open a pull request into `master`. Its title is the message the squashed commit will have:
   `type(scope): summary`. The description lists what changed and why, as short bullets.
4. Assign the pull request to the person who did the work: the release notes credit the assignee. The **CI** workflow (`.github/workflows/ci.yml`) runs fmt, clippy and the tests on the pull request. Merge only when
   its **Check** job is green.
5. Squash-merge, so `master` has one commit per change, then delete the branch.

## Release

A release is a tag on `master` after the merges, never a push: [cut a release](cut-a-release.md). Pushes to `master` and
merged pull requests do not publish anything; only a tag that starts with `v` does.

## Make GitHub help (optional)

These settings are in the repository's **Settings > Branches** and **Settings > Tags**; some are not available for a
private repository on a free plan.

- Protect `master`: require a pull request and the **Check** status before merging, and block force pushes.
- Protect the `v*` tags so only the owner can create them.
