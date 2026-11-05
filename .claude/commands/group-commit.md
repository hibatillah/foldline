---
description: Make every commit in this repo with this command, even when the user does not name it. One scope gives one commit, several scopes give a few, each with a `type(scope): message` subject and short bullets
argument-hint: [yes]
---

Commit what is changed in the working tree, grouped by scope. Use this command for every commit, whether the user typed it or only asked to commit; do not run `git commit` by hand or use another commit skill. Commit only when the user has asked; never push. `$ARGUMENTS` of `yes` skips the confirmation in step 3.

## 1. Look

Run `git status --short` and `git diff HEAD --stat`, then read the diff of each group you form. Stop and say so if there is nothing to commit. Leave out files that hold secrets (`.env`, keys, tokens) and tell the user.

## 2. Group

Map each changed file to one scope by its path:

| Scope | Paths |
| --- | --- |
| `notch` | `crates/notch/**` |
| `settings` | `crates/settings/**`, `crates/store/**` (the store is too small to stand alone) |
| `docs` | `docs/**`, `README.md`, `ONBOARDING.md` |
| `agent` | `.claude/**`, `AGENTS.md`, `CLAUDE.md`, `skills-lock.json` |
| `build` | `scripts/**`, `.github/**`, `installer/**`, `Cargo.toml`, `Cargo.lock`, `vendor/**`, `assets/**`, `.cargo/**`, `.gitignore`, `rustfmt.toml`, `LICENSE`, `THIRD-PARTY-LICENSES.txt` |

- A group of one or two small files joins the closest group (a lone `.gitignore` joins `build`). `Cargo.lock` and `Cargo.toml` changes go with the crate group that caused them.
- A change that needs another group to compile goes in the same commit.
- One scope with changes means one commit. Do not split it to look tidy.
- A file belongs to one commit only; never split a file. If it fits two scopes, use the larger one and say so.
- Order the commits so each one builds: `build`, `settings`, `notch`, then `docs`, `agent`.

## 3. Plan and confirm

Pick a type per commit: `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `build` or `chore`. Write the message as:

```
type(scope): imperative summary, lowercase, under 72 characters

- what changed, in a few words
- why, when it is not obvious
```

Two to four bullets, no filler. End every message with the attribution lines from the session reminder.

One commit: state its message and files in a line or two, then commit; no confirmation. Several commits: show one line per commit (scope, type, file count, subject) with its bullets, and ask before committing unless `$ARGUMENTS` is `yes` or the user already approved this split.

## 4. Check

If a commit touches Rust or a `Cargo` file, run once before committing: `cargo +stable-x86_64-pc-windows-msvc fmt --all --check`, `cargo +stable-x86_64-pc-windows-msvc clippy --workspace --all-targets -- -D warnings` and `cargo +stable-x86_64-pc-windows-msvc test --workspace`. Stop and show the output if one fails. Skip this step when only docs, agent files or scripts changed.

## 5. Commit

For each group: `git add -- <its files>` (name the paths; never `git add -A`), then `git commit` with the message from a heredoc. Do not amend, skip hooks or push.

## 6. Report

Show `git log --oneline` for the new commits and `git status --short`. Say what is left uncommitted and why.
