#!/usr/bin/env bash
# Registers the staged MSIX (dist/msix-<version>/package) on this PC for a local test, no certificate needed (Developer Mode on).
# Usage: scripts/install-msix-local.sh [--build] [--remove]
#   --build   run scripts/build-msix.sh first (honours CARGO_TARGET_DIR)
#   --remove  unregister the package and stop
# The package keeps its files in dist/, so rebuilding while it runs needs it removed first (the script does that).
set -euo pipefail
cd "$(dirname "$0")/.."

build=0
remove=0
for argument in "$@"; do
    case "$argument" in
        --build) build=1 ;;
        --remove) remove=1 ;;
        *) echo "unknown option: $argument" >&2; exit 2 ;;
    esac
done

package_name="Hibatillah.foldline"
unregister() {
    powershell.exe -NoProfile -Command "Get-AppxPackage $package_name | Remove-AppxPackage"
}

if [ "$remove" = 1 ]; then unregister; echo "removed $package_name"; exit 0; fi

if [ "$build" = 1 ]; then
    unregister
    bash scripts/build-msix.sh
fi

version=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
manifest="$PWD/dist/msix-$version/package/AppxManifest.xml"
[ -f "$manifest" ] || { echo "no staged package; run with --build" >&2; exit 1; }

powershell.exe -NoProfile -Command "Add-AppxPackage -Register '$(cygpath -w "$manifest")'; \$package = Get-AppxPackage $package_name; \"registered \$(\$package.PackageFullName)\"; Start-Process ('shell:AppsFolder\\' + \$package.PackageFamilyName + '!Foldline')"
