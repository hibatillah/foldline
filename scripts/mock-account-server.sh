#!/usr/bin/env bash
# Starts the mock account server (or writes samples with --write-samples <directory>).
cd "$(dirname "$0")/.."
exec uv run --no-project --with cryptography scripts/mock-account-server.py "$@"
