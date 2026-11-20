#!/usr/bin/env bash
# Lists what a content branch (a page, an activity or settings of your own) changed outside its own content area.
#
#   scripts/check-content-paths.sh --id <id> [--id <other>] [--base <ref>]
#
# The base is the merge base with master unless --base says otherwise. The output is Markdown: paste it into the pull
# request description. It exits 1 when something is outside the content area or a registry line is not yours.
set -euo pipefail

cd "$(dirname "$0")/.."

ids=()
base=''
while [ "$#" -gt 0 ]; do
  case $1 in
    --id) ids+=("$2"); shift 2 ;;
    --base) base=$2; shift 2 ;;
    *) sed -n '2,8p' "$0" | sed 's/^# \{0,1\}//'; exit 2 ;;
  esac
done
[ "${#ids[@]}" -gt 0 ] || { echo "error: pass at least one --id <id>" >&2; exit 2; }

if [ -z "$base" ]; then
  for candidate in origin/master master; do
    if git rev-parse --verify -q "$candidate" >/dev/null; then base=$(git merge-base HEAD "$candidate"); break; fi
  done
fi
[ -n "$base" ] || { echo "error: no master branch to compare with; pass --base <ref>" >&2; exit 2; }

registries=(
  crates/notch/src/pages/mod.rs
  crates/notch/src/activities/mod.rs
  crates/settings/src/sections/mod.rs
  crates/settings/src/sections/pages/list.rs
  crates/settings/src/icons.rs
)

is_registry() {
  local file
  for file in "${registries[@]}"; do [ "$file" = "$1" ] && return 0; done
  return 1
}

# True when $1 is a file of the content area of one of the ids.
in_area() {
  local path=$1 id
  for id in "${ids[@]}"; do
    case $path in
      crates/notch/src/pages/$id/*|crates/notch/src/activities/$id.rs|crates/notch/src/activities/$id/*) return 0 ;;
      crates/settings/src/sections/$id/*) return 0 ;;
      crates/notch/locales/pages/$id.yml|crates/notch/locales/activities/$id.yml|crates/settings/locales/pages/$id.yml) return 0 ;;
      crates/settings/assets/icons/${id//_/-}.svg) return 0 ;;
    esac
  done
  return 1
}

# Words that mark a registry line as belonging to one of the ids: the id, its PascalCase and its upper-case icon constant.
words=()
for id in "${ids[@]}"; do
  words+=("$id" "$(echo "$id" | awk -F_ '{ for (i = 1; i <= NF; i++) printf "%s%s", toupper(substr($i, 1, 1)), substr($i, 2) }')" "$(echo "$id" | tr '[:lower:]' '[:upper:]')" "${id//_/-}")
done

line_is_yours() {
  local line=$1 word
  for word in "${words[@]}"; do
    case $line in *"$word"*) return 0 ;; esac
  done
  return 1
}

mapfile -t changed < <({ git diff --name-only "$base"; git ls-files --others --exclude-standard; } | sort -u)

area=()
registry=()
docs=()
outside=()
for path in "${changed[@]}"; do
  [ -n "$path" ] || continue
  if in_area "$path"; then area+=("$path")
  elif is_registry "$path"; then registry+=("$path")
  elif [[ $path == docs/* || $path == README.md ]]; then docs+=("$path")
  else outside+=("$path")
  fi
done

problems=0
echo "## Content check for \`${ids[*]}\`"
echo
echo "Compared with \`${base:0:9}\`."
echo
echo "**In the content area (${#area[@]})**"
for path in "${area[@]}"; do echo "- \`$path\`"; done
[ "${#area[@]}" -gt 0 ] || echo "- none"

echo
echo "**Registry lines**"
if [ "${#registry[@]}" -eq 0 ]; then echo "- none"; fi
for path in "${registry[@]}"; do
  added=0
  foreign=()
  while IFS= read -r line; do
    case $line in
      '+++'*|'---'*) continue ;;
      +*) added=$((added + 1)); line_is_yours "${line:1}" || foreign+=("$line") ;;
      -*) foreign+=("$line") ;;
    esac
  done < <(git diff -U0 "$base" -- "$path")
  if [ "${#foreign[@]}" -eq 0 ]; then
    echo "- \`$path\`: $added line(s) added, all yours"
  else
    problems=1
    echo "- \`$path\`: lines that are not yours:"
    for line in "${foreign[@]}"; do echo "  - \`$line\`"; done
  fi
done

echo
echo "**Docs (check the rows are yours)**"
for path in "${docs[@]}"; do echo "- \`$path\`"; done
[ "${#docs[@]}" -gt 0 ] || echo "- none"

echo
echo "**Outside the content area (${#outside[@]})**"
for path in "${outside[@]}"; do echo "- \`$path\`"; done
if [ "${#outside[@]}" -gt 0 ]; then problems=1; else echo "- none"; fi

exit "$problems"
