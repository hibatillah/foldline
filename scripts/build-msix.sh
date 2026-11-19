#!/usr/bin/env bash
# Build an unsigned MSIX for the Microsoft Store (the Store signs it): dist/<name>-<version>-x64.msix.
# Usage: scripts/build-msix.sh [--no-build]   --no-build packs the release binaries already in target/.
set -euo pipefail
cd "$(dirname "$0")/.."

build=1
for argument in "$@"; do
    case "$argument" in
        --no-build) build=0 ;;
        *) echo "unknown option: $argument" >&2; exit 2 ;;
    esac
done

name=$(grep -A3 '^\[\[bin\]\]' crates/notch/Cargo.toml | grep -m1 '^name' | cut -d'"' -f2)
settings_name=$(grep -A3 '^\[\[bin\]\]' crates/settings/Cargo.toml | grep -m1 '^name' | cut -d'"' -f2)
version=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
# An MSIX version is four numbers and the Store keeps the fourth at 0, so a pre-release suffix has no place in it.
if ! [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "workspace version '$version' is not major.minor.patch; an MSIX cannot carry a pre-release suffix" >&2
    exit 1
fi
package_version="$version.0"

# PATH first, then the newest Windows Kits version, as scripts/icon_resource.rs finds rc.exe.
find_sdk_tool() {
    local tool="$1" found kits_directory
    found=$(command -v "$tool" 2>/dev/null || true)
    if [ -n "$found" ]; then echo "$found"; return; fi
    kits_directory="${WindowsSdkDir:-$(printenv 'ProgramFiles(x86)' || true)/Windows Kits/10}"
    kits_directory=$(cygpath -u "$kits_directory")
    for version_directory in $(ls -1 "$kits_directory/bin" 2>/dev/null | grep -E '^[0-9]+(\.[0-9]+)+$' | sort -t. -k1,1nr -k2,2nr -k3,3nr -k4,4nr); do
        if [ -f "$kits_directory/bin/$version_directory/x64/$tool" ]; then
            echo "$kits_directory/bin/$version_directory/x64/$tool"; return
        fi
    done
    echo "$tool not found; install the Windows SDK (App Certification Kit / MSIX packaging tools)" >&2
    exit 1
}
makeappx=$(find_sdk_tool makeappx.exe)
makepri=$(find_sdk_tool makepri.exe)

if [ "$build" = 1 ]; then
    # A plain version gives both programs the blue stable tile, whatever the latest tag (scripts/release_version.rs).
    FOLDLINE_RELEASE_VERSION="$version" cargo +stable-x86_64-pc-windows-msvc build --release
    # GPUI builds only with the MSVC host toolchain.
    FOLDLINE_RELEASE_VERSION="$version" cargo +stable-x86_64-pc-windows-msvc build --release -p settings
fi

release_directory="${CARGO_TARGET_DIR:-target}/x86_64-pc-windows-msvc/release"
# The package carries the blue logos, so binaries with the indigo dev tile (an older --no-build) are refused.
bash scripts/check-icon.sh "$release_directory/$name.exe" tile-stable
bash scripts/check-icon.sh "$release_directory/$settings_name.exe" tile-stable

work_directory="dist/msix-$version"
staging="$work_directory/package"
output="dist/$name-$version-x64.msix"
rm -rf "$work_directory"
mkdir -p "$staging/Assets"

cp "$release_directory/$name.exe" "$release_directory/$settings_name.exe" "$staging/"
cp THIRD-PARTY-LICENSES.txt assets/fonts/Geist-LICENSE.txt "$staging/"
cp LICENSE "$staging/LICENSE.txt"
cp installer/msix/Assets/*.png "$staging/Assets/"
sed "s/{{VERSION}}/$package_version/" installer/msix/AppxManifest.xml > "$staging/AppxManifest.xml"

# The scale and targetsize variants of the logos resolve only through resources.pri.
MSYS2_ARG_CONV_EXCL='*' "$makepri" createconfig /cf "$(cygpath -w "$work_directory/priconfig.xml")" /dq en-US /pv 10.0.0 /o
# Without the default <packaging> block, every variant stays in one resources.pri; split files belong to bundles.
sed -i '/<packaging>/,/<\/packaging>/d' "$work_directory/priconfig.xml"
MSYS2_ARG_CONV_EXCL='*' "$makepri" new /pr "$(cygpath -w "$staging")" /cf "$(cygpath -w "$work_directory/priconfig.xml")" \
    /mn "$(cygpath -w "$staging/AppxManifest.xml")" /of "$(cygpath -w "$staging/resources.pri")" /o

rm -f "$output"
MSYS2_ARG_CONV_EXCL='*' "$makeappx" pack /d "$(cygpath -w "$staging")" /p "$(cygpath -w "$output")" /o

echo "built: $PWD/$output (version $package_version, unsigned)"
