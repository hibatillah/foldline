---
paths:
  - "docs/**"
  - "README.md"
  - "ARCHITECTURE.md"
---

# Docs

- State the current behavior and where it lives; leave out how it used to work and what a change replaced.
- Keep each fact in one doc and link to it from the others: settings keys and defaults in `docs/reference/settings-keys.md`, limits
  and gates in `docs/how-to/run-the-perf-check.md`, measured numbers in `docs/reference/performance-baseline.md`.
- `docs/reference/performance-baseline.md` holds the latest baseline only, with its date, machine, workload and the command that produced
  it. Replace the numbers on each run; leave failed or superseded runs to git history.
- Write the user-facing default of a setting in the `README.md` table and in `docs/reference/settings-keys.md` together.
- Sort a document by its job ([Diátaxis](https://diataxis.fr)): `docs/how-to/` completes a task (adding a page, cutting a release), `docs/reference/` states facts, `docs/explanation/` gives the reasons. One document has one type, and its first line says when to read it and when to skip it.
- A new document goes in the folder of its type and in `docs/README.md`; add it to the table in `AGENTS.md` when agents need it for a kind of work.
- `ARCHITECTURE.md` stays at the repository root beside `README.md`, `ONBOARDING.md` and `AGENTS.md`: it is the Explanation overview of the programs, crates, files and services, links to `docs/` for the detail, and states no fact another doc owns.
- Point to the source for what code already defines (a trait, a signature, a constant) instead of restating it; keep in the docs what the code does not show.
