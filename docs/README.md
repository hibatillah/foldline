# Documentation index

Three kinds of document, each with one job ([Diátaxis](https://diataxis.fr)). Every document opens with a line that says when to read it and when to skip it. Find the task, open one file.

New to the team? Start with [ONBOARDING.md](../ONBOARDING.md) at the repository root: setup, structure and the rules the code follows.

## How-to guides: do a task

| Read | When |
| --- | --- |
| [Build, run and debug](how-to/build-run-and-debug.md) | building, running a change, or finding why a page does not show |
| [Add a page](how-to/add-a-page.md) | writing a page, with one working example |
| [Add an activity](how-to/add-an-activity.md) | something should show in the collapsed pill |
| [Add a setting](how-to/add-a-setting.md) | a new key, or a control in the settings window |
| [Switch off or remove](how-to/switch-off-or-remove.md) | fewer pages or activities, temporarily or for good |
| [Hide Windows' own popups](how-to/hide-windows-popups.md) | Windows' volume box or notification banners show next to the notch |
| [Test and snapshot](how-to/test-and-snapshot.md) | writing a test, rendering page pictures, the manual checklist |
| [Run the performance check](how-to/run-the-perf-check.md) | after host, animation, timer or drawing changes, and before a release |
| [Work with branches](how-to/work-with-branches.md) | starting a change, opening or merging a pull request, or before you push |
| [Cut a release](how-to/cut-a-release.md) | publishing a version, the installer, license list, how users update |

## Reference: look something up

| Read | When |
| --- | --- |
| [Page API](reference/page-api.md) | where a hook or drawing call is defined, and rules the signatures do not show |
| [Activities](reference/activities.md) | what `Presence` can show, and what each built-in activity watches |
| [Settings keys](reference/settings-keys.md) | a key, its default, an environment variable, where a tunable lives |
| [Settings window](reference/settings-window.md) | what each section of `notch-settings.exe` sets |
| [Source map](reference/source-map.md) | where a file lives and which module may use which |
| [Layout](reference/layout.md) | padding, header and the open and collapsed shapes |
| [Design rules](reference/design-rules.md) | order, gaps, colors and fonts of what the user sees, and the reason for each |
| [Test map](reference/test-map.md) | which file tests which behavior |
| [Performance baseline](reference/performance-baseline.md) | the latest measured CPU and memory |
| [Dependencies and licenses](reference/dependencies-and-licenses.md) | adding or auditing a dependency |

## Explanation: understand why

| Read | When |
| --- | --- |
| [Architecture](explanation/architecture.md) | restructuring, or learning how the host, pages and activities fit |
| [Performance budget](explanation/performance-budget.md) | changing timers, animation or drawing |
