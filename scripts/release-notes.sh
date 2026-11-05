#!/usr/bin/env bash
# Prints the release notes of a version: GitHub's generated notes (the merged pull requests since the previous release,
# grouped by .github/release.yml), crediting each pull request's assignee when it has one and its author otherwise. When
# no pull request is in range (changes pushed straight to master) it lists the commit titles instead. Needs `gh`.
#   scripts/release-notes.sh v0.5.0 [ref]     ref defaults to the tag; pass master to preview before the tag exists
set -euo pipefail
cd "$(dirname "$0")/.."

tag="${1:?usage: scripts/release-notes.sh <tag> [ref]}"
ref="${2:-$tag}"
repository="${GITHUB_REPOSITORY:-$(gh repo view --json nameWithOwner --jq .nameWithOwner)}"
previous="$(scripts/previous-tag.sh "$tag" "$ref")"

arguments=(-f "tag_name=$tag" -f "target_commitish=$ref")
[ -n "$previous" ] && arguments+=(-f "previous_tag_name=$previous")
notes="$(gh api "repos/$repository/releases/generate-notes" "${arguments[@]}" --jq .body)"

# "* title by @author in https://github.com/owner/repo/pull/12": swap the author for the assignees when there are any.
credited=""
while IFS= read -r line; do
    number="$(printf '%s' "$line" | sed -n 's#^\* .* by @[^ ]* in https://github.com/[^ ]*/pull/\([0-9][0-9]*\)$#\1#p')"
    if [ -n "$number" ]; then
        assignees="$(gh api "repos/$repository/pulls/$number" --jq '[.assignees[].login] | map("@" + .) | join(", ")')"
        [ -n "$assignees" ] && line="$(printf '%s' "$line" | sed "s# by @[^ ]* in # by $assignees in #")"
    fi
    credited+="$line"$'\n'
done <<<"$notes"
notes="${credited%$'\n'}"

if ! printf '%s\n' "$notes" | grep -q '^\* '; then
    range="${previous:+$previous..}$ref"
    commits="$(git log --no-merges --format='* %s' "$range")"
    [ -n "$commits" ] && notes+=$'\n\n'"## Commits"$'\n'"$commits"
fi
printf '%s\n' "$notes"
