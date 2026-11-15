#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

python3 -m venv .venv
.venv/bin/python -m pip install --upgrade pip
.venv/bin/python -m pip install -e engine

cargo install --path crates/tt-cli --force

printf '\nInstalled binary: %s/.cargo/bin/tt\n' "$HOME"
printf 'If commands are not found, add Cargo to PATH:\n  export PATH="%s/.cargo/bin:$PATH"\n' "$HOME"
