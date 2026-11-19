#!/usr/bin/env bash
# Build the current working tree (committed or not) and package it for a teammate: an installer in dist/ when Inno Setup is installed, else a zip.
set -euo pipefail
cd "$(dirname "$0")/.."

name=$(grep -A3 '^\[\[bin\]\]' crates/notch/Cargo.toml | grep -m1 '^name' | cut -d'"' -f2)
settings_name=$(grep -A3 '^\[\[bin\]\]' crates/settings/Cargo.toml | grep -m1 '^name' | cut -d'"' -f2)
version=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
# A pre-release tag of this series makes the build a dev build (build.rs does the same), so the installer must use its version.
latest_tag=$(git describe --tags --abbrev=0 --match 'v*' 2>/dev/null || true)
case "$latest_tag" in "v$version"-?*) version="${latest_tag#v}" ;; esac
commit=$(git rev-parse --short HEAD 2>/dev/null || echo nogit)
git diff --quiet HEAD 2>/dev/null || commit="$commit-dirty"
package="$name-$version-test-$commit-windows-x64"

cargo +stable-x86_64-pc-windows-msvc build --release
# GPUI builds only with the MSVC host toolchain.
cargo +stable-x86_64-pc-windows-msvc build --release -p settings

release_directory="$PWD/target/x86_64-pc-windows-msvc/release"
mkdir -p dist

iscc=""
for candidate in "/c/Program Files (x86)/Inno Setup 6/ISCC.exe" "/c/Program Files/Inno Setup 6/ISCC.exe" "$LOCALAPPDATA/Programs/Inno Setup 6/ISCC.exe"; do
    if [ -f "$candidate" ]; then iscc="$candidate"; break; fi
done
if [ -z "$iscc" ]; then iscc=$(command -v ISCC.exe || true); fi

if [ -n "$iscc" ]; then
    # The installer compares AppVersion as numbers, so the commit goes in the file name only.
    MSYS2_ARG_CONV_EXCL='*' "$iscc" \
        "/DAppVersion=$version" "/DExeName=$name.exe" \
        "/DSourceExe=$(cygpath -w "$release_directory/$name.exe")" \
        "/DSettingsExeName=$settings_name.exe" \
        "/DSourceSettingsExe=$(cygpath -w "$release_directory/$settings_name.exe")" \
        "/O$(cygpath -w "$PWD/dist")" "/F$package-setup" installer/foldline.iss
    output="dist/$package-setup.exe"
else
    echo "Inno Setup not found (winget install JRSoftware.InnoSetup); packing a zip instead." >&2
    rm -f "dist/$package.zip"
    powershell.exe -NoProfile -Command "Compress-Archive -Force -Path '$(cygpath -w "$release_directory/$name.exe")','$(cygpath -w "$release_directory/$settings_name.exe")' -DestinationPath '$(cygpath -w "$PWD/dist/$package.zip")'"
    output="dist/$package.zip"
fi

echo "built: $PWD/$output"
