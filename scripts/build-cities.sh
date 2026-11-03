#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
uv run --no-project scripts/build_cities.py
