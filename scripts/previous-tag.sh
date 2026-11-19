#!/usr/bin/env bash
# Prints the tag a release's notes start from: for a stable tag the last stable tag, so its notes cover the whole series
# of pre-releases; for a pre-release the tag right before it. Only tags reachable from the release commit count.
#   scripts/previous-tag.sh v1.0.0 [ref]      ref defaults to the tag itself; pass master to preview before tagging
set -euo pipefail
cd "$(dirname "$0")/.."

tag="${1:?usage: scripts/previous-tag.sh <tag> [ref]}"
ref="${2:-$tag}"

case "$tag" in
    *-*) keep='.' ;;
    *) keep='^v[0-9]+\.[0-9]+\.[0-9]+$' ;;
esac

# versionsort.suffix puts v1.0.0-alpha.2 before v1.0.0.
git -c versionsort.suffix=- tag --list 'v*' --merged "$ref" --sort=-version:refname \
    | grep -E "$keep" \
    | grep -vxF "$tag" \
    | head -n 1 || true
