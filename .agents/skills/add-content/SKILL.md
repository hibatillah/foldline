---
name: add-content
description: Add your own page, activity or page settings to Foldline from a plain description, ready for a pull request. Use when a contributor says add a page, add an activity, new page, new activity, my own content or page settings.
---

Add content to Foldline for a pull request. The contributor's description is the user's message.

1. Read `.claude/agents/add-content.md` and follow it. If your harness has subagents, dispatch the `add-content` agent (`.codex/agents/add-content.toml` or `.agents/agents/add-content.md`) with this brief instead.
2. Read `docs/how-to/add-your-content.md` first and follow its flow: decide the parts, scaffold with `scripts/new-content.sh`, fill in, test, add the docs rows, run `scripts/check-content-paths.sh --id <id>`.
3. Stay inside the contributor's own files. Do not commit or push.

If the description is empty, ask the contributor to describe the page, the activity or the settings in a few sentences, and stop.
