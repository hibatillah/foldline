#!/usr/bin/env bash
# Regenerate the MSIX logos in installer/msix/Assets from the blue tile; commit the output.
set -euo pipefail
cd "$(dirname "$0")/.."
uv run --no-project --with pillow scripts/build_msix_assets.py
