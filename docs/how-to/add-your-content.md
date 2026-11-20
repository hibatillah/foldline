# Add your own content

> **How-to.** Read when: you want to add one page, one activity or the settings of a page of your own and open a pull request that changes only your own files. Skip if: your change touches the host, the drawing code, the `Page` or `Activity` trait or another page (it is an area change; see [AGENTS.md](../../AGENTS.md)), or you only need the API of one part (see [add a page](add-a-page.md), [add an activity](add-an-activity.md) and [add a setting](add-a-setting.md)).

You add content in three tracks, alone or together under one id:

- **Page:** what the open notch shows.
- **Activity:** what the collapsed pill shows while something happens.
- **Settings:** the controls of a page in the settings window.

Registration is explicit and nothing is discovered: each track is a folder or file of your own plus one line in a registry file. A script scaffolds the files and the lines, and a second script checks that nothing else changed.

## Pick a route

- **With an agent.** Run `/add-content <what it should do>` in Claude Code, or paste the prompt from [CONTRIBUTING.md](../../CONTRIBUTING.md) into any agent that reads [AGENTS.md](../../AGENTS.md). The agent follows this page.
- **By hand.** Follow the steps below.
- **Full control.** Skip the scaffold and read the three how-to guides and the code; the limits at the end of this page still apply.

## What each track touches

| Track | Your files | One line in a registry |
| --- | --- | --- |
| Page | `crates/notch/src/pages/<id>/mod.rs`; `crates/notch/locales/pages/<id>.yml`; `crates/settings/locales/pages/<id>.yml` (the `pages.desc_<id>` text); `crates/settings/assets/icons/<stem>.svg` | `mod <id>;` and one line in `all()` of `crates/notch/src/pages/mod.rs`; one `PageRow` in `crates/settings/src/sections/pages/list.rs`; one `own_icons!` line in `crates/settings/src/icons.rs` |
| Activity | `crates/notch/src/activities/<id>.rs` (or a folder); optionally `crates/notch/locales/activities/<id>.yml` | `mod <id>;` and one line in `all()` of `crates/notch/src/activities/mod.rs` |
| Settings | `crates/settings/src/sections/<id>/mod.rs`; `crates/settings/locales/pages/<id>.yml`; `crates/settings/assets/icons/<stem>.svg` | `mod <id>;` and one `NavEntry` in `crates/settings/src/sections/mod.rs`; one `own_icons!` line in `crates/settings/src/icons.rs` |

`<stem>` is the id with `_` written as `-`. A page with settings uses one id for both tracks and shares the icon line and the settings locale file.

## Steps

### 1. Choose the id

Lowercase letters, digits and underscores, starting with a letter. It must not be used in `pages/`, `activities/` or `sections/`. Every settings key of your content starts with it (`<id>.formal`), so ids never collide.

### 2. Scaffold

From the repository root, in Git Bash:

```
scripts/new-content.sh page <id> ["Name"]
scripts/new-content.sh settings <id> ["Name"]
scripts/new-content.sh activity <id> ["Name"]
```

Run it once per track. It creates files that compile and carry `TODO` lines, makes the registry edits of the table above and stops when a file or a Pages row of that id already exists. For a page with settings, run `page` and `settings` with the same id.

### 3. Fill in

Replace every `TODO`. The code of each track follows its how-to guide:

- **Page:** [add a page](add-a-page.md) for the hooks and drawing. The page's name is the text `page.<id>`; look texts up with `tr!` when you create or rebuild them, never in `draw_*`. Write one whole sentence in `pages.desc_<id>`: the Pages list shows it under the name. Replace the placeholder icon with yours (24 by 24 `viewBox`, `currentColor`).
- **Activity:** [add an activity](add-an-activity.md) for `presence`, `draw` and how to get events without polling.
- **Settings:** [add a setting](add-a-setting.md) for the controls. Read your keys in `on_start` and again in `on_settings_changed`, falling back to the current value.

Texts live in your own locale files:

- Every file starts with `_version: 2`, and every key has an `en` and an `id` line.
- Settings keys are `<id>.title`, `<id>.group_<name>`, `<id>.<setting>` and `<id>.<setting>_note`.
- A key lives in one locale file only; the texts that already exist stay in `app.yml`.
- A key used nowhere in the source fails a test, so remove what you do not use.

### 4. Test

Add a unit test, named as a sentence about behavior, for each piece of logic; the scaffold's own test stays green. A page needs no snapshot sample: a page without one is left out of the host test rigs (`samples()` in `crates/notch/src/pages/mod.rs`). Add one only to render pictures of its states ([test and snapshot](test-and-snapshot.md)).

Two tests keep your content separate: a page imports no other page, and an activity imports no other activity and no page (`privacy` is a shared helper any activity may use). They are in `crates/notch/src/pages/mod.rs` and `crates/notch/src/activities/mod.rs`.

### 5. Check

Close `foldline-settings.exe` first (the settings tests need its single-instance mutex), then run:

```
cargo +stable-x86_64-pc-windows-msvc fmt --check
cargo +stable-x86_64-pc-windows-msvc clippy --workspace --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-msvc test --workspace
```

Run your page with the others hidden by setting `NOTCH_PAGES=shelf,<id>` ([add a page](add-a-page.md#4-run-it)). When your content adds a timer, a thread, a hook or drawing, run `scripts/perf.sh` too ([run the perf check](run-the-perf-check.md)); it moves the mouse for about three minutes.

### 6. Add the docs rows

A new settings key gets a row in [settings keys](../reference/settings-keys.md). A new page or activity gets one line in the "What it includes" list of `README.md`. Leave every other document alone.

### 7. Prove the boundary

```
scripts/check-content-paths.sh --id <id> [--id <other>] [--base <ref>]
```

It compares your branch with the merge base with `master` (or `--base`) and prints Markdown with four lists: files in your content area, registry lines, docs, and files outside. It exits 1 when a file is outside the content area or a registry line is not yours. A registry line is yours when it holds your id, its PascalCase form, its upper-case icon constant or its `<stem>`; a changed or removed line is never yours, so do not reformat a registry file.

If it lists something outside, undo that change. When your content cannot work without it, stop: that change is a separate pull request.

### 8. Open the pull request

Work on a branch named `type/short-name` and open a pull request into `master` ([CONTRIBUTING.md](../../CONTRIBUTING.md)). Paste the output of step 7 into the description.

## Limits

- A content pull request changes no file in `host/`, `gfx/`, `ui/` or `platform/`, none of `page.rs`, `activity.rs` and `crates/store`, and no other page, activity or settings section.
- Registry files change by your lines only. A key built at run time is listed by prefix in `DYNAMIC_KEY_PREFIXES` in `crates/notch/src/i18n.rs`, which is outside your area: use literal keys.
- The budget holds: no polling, a timer or animation stops when its output stops changing, shapes are drawn through `Canvas` caches ([performance budget](../explanation/performance-budget.md)).
- Anything your content reads from another app, the clipboard, the network or the file system goes into the Privacy section of `README.md`.
- A new dependency must allow GPL-3.0-or-later ([dependencies and licenses](../reference/dependencies-and-licenses.md)).
- No accounts, paid tiers or advertising, and no Foldline name or logo in your content ([TRADEMARK.md](../../TRADEMARK.md)).

## Next

- Hooks, drawing and registration of a page: [add a page](add-a-page.md).
- Activities and how they get events: [add an activity](add-an-activity.md).
- Keys and controls: [add a setting](add-a-setting.md).
- Where each file lives: [source map](../reference/source-map.md).
