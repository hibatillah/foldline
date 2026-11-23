#!/usr/bin/env bash
# Pack the sparse package that gives the installed foldline.exe a package identity: dist/foldline-sparse-<version>.msix,
# unsigned. Sign it with a certificate whose subject equals Publisher in installer/sparse/AppxManifest.xml, then copy it
# to installer/sparse/foldline-sparse.msix; installer/foldline.iss ships and registers it only when that file exists.
# Usage: scripts/build-sparse-package.sh
set -euo pipefail
cd "$(dirname "$0")/.."

version=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
# A package version is four numbers, so a pre-release suffix has no place in it; dev installs do not register it.
if ! [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "workspace version '$version' is not major.minor.patch; a package cannot carry a pre-release suffix" >&2
    exit 1
fi
package_version="$version.0"

# PATH first, then the newest Windows Kits version, as scripts/build-msix.sh finds them.
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
    echo "$tool not found; install the Windows SDK (MSIX packaging tools)" >&2
    exit 1
}
makeappx=$(find_sdk_tool makeappx.exe)
makepri=$(find_sdk_tool makepri.exe)

work_directory="dist/sparse-$version"
staging="$work_directory/package"
output="dist/foldline-sparse-$version.msix"
rm -rf "$work_directory"
mkdir -p "$staging/Assets"
# Only the manifest and logos: the program stays in the install folder (the external location).
cp installer/msix/Assets/*.png "$staging/Assets/"
sed "s/{{VERSION}}/$package_version/" installer/sparse/AppxManifest.xml > "$staging/AppxManifest.xml"

# The scale and targetsize variants of the logos resolve only through resources.pri.
MSYS2_ARG_CONV_EXCL='*' "$makepri" createconfig /cf "$(cygpath -w "$work_directory/priconfig.xml")" /dq en-US /pv 10.0.0 /o
sed -i '/<packaging>/,/<\/packaging>/d' "$work_directory/priconfig.xml"
MSYS2_ARG_CONV_EXCL='*' "$makepri" new /pr "$(cygpath -w "$staging")" /cf "$(cygpath -w "$work_directory/priconfig.xml")" \
    /mn "$(cygpath -w "$staging/AppxManifest.xml")" /of "$(cygpath -w "$staging/resources.pri")" /o

rm -f "$output"
# /nv: a sparse package has no executable inside it, which the default validation expects.
MSYS2_ARG_CONV_EXCL='*' "$makeappx" pack /d "$(cygpath -w "$staging")" /p "$(cygpath -w "$output")" /nv /o

echo "built: $PWD/$output (version $package_version, unsigned)"
