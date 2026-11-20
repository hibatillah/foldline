---
description: Add your own page, activity or page settings to Foldline from a plain description, ready for a pull request
argument-hint: <what your page, activity or settings should do>
disable-model-invocation: true
---

Add content to Foldline for a pull request. The contributor's description: `$ARGUMENTS`.

Dispatch the `add-content` agent through the Agent tool with this brief, and relay its report:

- The description above, word for word.
- Read `docs/how-to/add-your-content.md` first and follow its flow: decide the parts, scaffold with `scripts/new-content.sh`, fill in, test, add the docs rows, run `scripts/check-content-paths.sh --id <id>`.
- Stay inside the contributor's own files. Do not commit or push.

If the description is empty, ask the contributor to describe the page, the activity or the settings in a few sentences, and stop.
