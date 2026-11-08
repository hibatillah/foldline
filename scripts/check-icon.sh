#!/usr/bin/env bash
# Check that an executable embeds the icon of one tile (tile-stable or tile-dev) and not the other: check-icon.sh <exe> <tile>.
set -euo pipefail
cd "$(dirname "$0")/.."

executable="$1"
expected="$2"

# The first frame of an ICO file is stored byte for byte inside the executable by the resource compiler.
frame_hex() {
    local icon="assets/branding/$1.ico" size offset
    size=$(od -An -tu4 -j14 -N4 "$icon" | tr -d ' ')
    offset=$(od -An -tu4 -j18 -N4 "$icon" | tr -d ' ')
    tail -c +$((offset + 1)) "$icon" | head -c "$size" | od -An -v -tx1 | tr -d ' \n'
}

executable_hex=$(mktemp)
trap 'rm -f "$executable_hex"' EXIT
od -An -v -tx1 "$executable" | tr -d ' \n' >"$executable_hex"

for tile in tile-stable tile-dev; do
    if grep -qF "$(frame_hex "$tile")" "$executable_hex"; then found=1; else found=0; fi
    if [ "$tile" = "$expected" ] && [ "$found" = 0 ]; then echo "$executable does not embed $tile" >&2; exit 1; fi
    if [ "$tile" != "$expected" ] && [ "$found" = 1 ]; then echo "$executable embeds $tile, expected $expected" >&2; exit 1; fi
done
echo "$executable embeds $expected"
