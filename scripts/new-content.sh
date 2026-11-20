#!/usr/bin/env bash
# Scaffolds your own content for Foldline: a page, an activity, or the settings section of a page.
#
#   scripts/new-content.sh page <id> ["Display name"]      page in the notch + its row in the settings Pages list
#   scripts/new-content.sh settings <id> ["Display name"]  settings section of an existing page (sidebar entry)
#   scripts/new-content.sh activity <id> ["Display name"]  activity of the collapsed pill
#
# <id> is lowercase letters, digits and underscores, starting with a letter (it is the settings key prefix).
# A page that also needs settings and an activity: run the three kinds with the same id.
# Every file it creates compiles and has TODO lines to replace; docs/how-to/add-your-content.md explains the rest.
set -euo pipefail

cd "$(dirname "$0")/.."
templates=scripts/templates
notch=crates/notch
settings=crates/settings

usage() {
  sed -n '2,9p' "$0" | sed 's/^# \{0,1\}//'
  exit 2
}

[ "$#" -ge 2 ] || usage
kind=$1
id=$2
title=${3:-}

[[ $id =~ ^[a-z][a-z0-9_]*$ ]] || { echo "error: id '$id' must be lowercase letters, digits and underscores, starting with a letter" >&2; exit 1; }

pascal() { echo "$1" | awk -F_ '{ for (i = 1; i <= NF; i++) printf "%s%s", toupper(substr($i, 1, 1)), substr($i, 2) }'; }
name=$(pascal "$id")
[ -n "$title" ] || title=$(echo "$id" | awk -F_ '{ for (i = 1; i <= NF; i++) printf "%s%s%s", (i > 1 ? " " : ""), toupper(substr($i, 1, 1)), substr($i, 2) }')
upper=$(echo "$id" | tr '[:lower:]' '[:upper:]')

fill() { sed -e "s/{{id}}/$id/g" -e "s/{{Name}}/$name/g" -e "s/{{Title}}/$title/g" "$1"; }

fail_if_exists() {
  [ ! -e "$1" ] || { echo "error: $1 already exists" >&2; exit 1; }
}

# Writes a copy of template $1 to $2, creating the folder.
create_from() {
  mkdir -p "$(dirname "$2")"
  fill "$1" > "$2"
  echo "created   $2"
}

# Appends text $2 to file $1 as the line(s) it should end with, matching the file's line endings.
add_lines() {
  local file=$1 text=$2 eol=''
  grep -q $'\r' "$file" && eol=$'\r'
  [ -z "$eol" ] || text=$(printf '%s' "$text" | sed "s/\$/$eol/")
  printf '%s\n' "$text" >> "$file"
}

# Declares `mod $id;` in file $1 in alphabetical order among its `mod` lines, which is where cargo fmt keeps them.
insert_mod() {
  local file=$1 eol=''
  grep -q $'\r' "$file" && eol=$'\r'
  awk -v id="$id" -v eol="$eol" '
    /^(pub )?mod [a-z_0-9]+;/ {
      name = $0; sub(/^(pub )?mod /, "", name); sub(/;.*/, "", name)
      last = NR
      if (!placed && name > id) { print "mod " id ";" eol; placed = 1 }
    }
    { print }
    END { if (!placed && !last) exit 3 }' "$file" > "$file.tmp" || { rm -f "$file.tmp"; echo "error: no mod line in $file" >&2; exit 1; }
  if ! grep -q "^mod $id;" "$file.tmp"; then
    insert_after_last "$file.tmp" '^(pub )?mod [a-z_0-9]+;' "mod $id;" >/dev/null
  fi
  mv "$file.tmp" "$file"
  echo "edited    $file"
}

# Inserts text $3 on the line after the last line of $1 that matches regex $2.
insert_after_last() {
  local file=$1 pattern=$2 text=$3 eol=''
  grep -q $'\r' "$file" && eol=$'\r'
  local last
  last=$(grep -n -E "$pattern" "$file" | tail -1 | cut -d: -f1)
  [ -n "$last" ] || { echo "error: no line of $file matches $pattern" >&2; exit 1; }
  awk -v n="$last" -v text="$text$eol" '{ print } NR == n { print text }' "$file" > "$file.tmp" && mv "$file.tmp" "$file"
  echo "edited    $file"
}

# Inserts text $4 before the first line equal to $3 that comes after the line matching $2.
insert_before_closer() {
  local file=$1 start=$2 closer=$3 text=$4 eol=''
  grep -q $'\r' "$file" && eol=$'\r'
  awk -v start="$start" -v closer="$closer" -v text="$text$eol" -v eol="$eol" '
    $0 ~ start { seen = 1 }
    seen && !done && $0 == closer eol { print text; done = 1 }
    { print }
    END { if (!done) exit 3 }' "$file" > "$file.tmp" || { rm -f "$file.tmp"; echo "error: no closing line '$closer' after '$start' in $file" >&2; exit 1; }
  mv "$file.tmp" "$file"
  echo "edited    $file"
}

# A locale file starts with _version: 2; keys are appended once and never twice across files.
append_locale() {
  local file=$1 template=$2 first_key
  first_key=$(grep -v '^_version' "$template" | grep -m1 -E '^[^ ]+:' | cut -d: -f1 | sed "s/{{id}}/$id/")
  if [ ! -f "$file" ]; then
    mkdir -p "$(dirname "$file")"
    printf '_version: 2\n' > "$file"
    echo "created   $file"
  fi
  grep -q "^$first_key:" "$file" 2>/dev/null && return 0
  grep -v '^_version' "$template" | sed -e "s/{{id}}/$id/g" -e "s/{{Title}}/$title/g" >> "$file"
  echo "extended  $file"
}

# Adds the settings icon of the id once: the constant in icons.rs and a placeholder svg to replace.
ensure_icon() {
  local stem
  stem=$(echo "$id" | tr '_' '-')
  grep -q "^    $upper = " "$settings/src/icons.rs" && return 0
  insert_before_closer "$settings/src/icons.rs" '^own_icons! [{]' '}' "    /// Placeholder square of $id; replace the svg with your icon (24x24 viewBox, currentColor).
    $upper = \"$stem\","
  [ -e "$settings/assets/icons/$stem.svg" ] || { cp "$templates/icon.svg" "$settings/assets/icons/$stem.svg"; echo "created   $settings/assets/icons/$stem.svg"; }
}

case $kind in
  page)
    fail_if_exists "$notch/src/pages/$id"
    grep -q "id: \"$id\"" "$settings/src/sections/pages/list.rs" && { echo "error: the Pages list already has a row for $id" >&2; exit 1; }
    create_from "$templates/page.rs" "$notch/src/pages/$id/mod.rs"
    append_locale "$notch/locales/pages/$id.yml" "$templates/notch_page.yml"
    insert_mod "$notch/src/pages/mod.rs"
    insert_before_closer "$notch/src/pages/mod.rs" '^pub fn all[(][)]' '    ]' "        Box::new($id::${name}Page::new()),"
    ensure_icon
    insert_before_closer "$settings/src/sections/pages/list.rs" '^const PAGE_ROWS' '];' "    PageRow { id: \"$id\", icon: icons::$upper, description: || tr!(\"pages.desc_$id\") },"
    append_locale "$settings/locales/pages/$id.yml" "$templates/settings_page_desc.yml"
    ;;
  settings)
    fail_if_exists "$settings/src/sections/$id"
    create_from "$templates/settings_section.rs" "$settings/src/sections/$id/mod.rs"
    append_locale "$settings/locales/pages/$id.yml" "$templates/settings_section.yml"
    ensure_icon
    insert_mod "$settings/src/sections/mod.rs"
    insert_after_last "$settings/src/sections/mod.rs" 'NavGroup::PageSettings' "        NavEntry::new(NavGroup::PageSettings, icons::$upper, $id::page()),"
    ;;
  activity)
    [ ! -e "$notch/src/activities/$id.rs" ] && [ ! -e "$notch/src/activities/$id" ] || { echo "error: activity $id already exists" >&2; exit 1; }
    create_from "$templates/activity.rs" "$notch/src/activities/$id.rs"
    insert_mod "$notch/src/activities/mod.rs"
    insert_before_closer "$notch/src/activities/mod.rs" '^pub fn all[(][)]' '    ]' "        Box::new($id::${name}Activity::new()),"
    ;;
  *)
    usage
    ;;
esac

echo
echo "Next: replace the TODO lines, then run"
echo "  cargo +stable-x86_64-pc-windows-msvc test --workspace"
echo "and check your branch with scripts/check-content-paths.sh --id $id"
