#!/usr/bin/env bash
# File: install.sh
# Author: Michal Švrček
# RepoDoctor Linux installer.
# ver. 0.6.0

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

SOURCE="${1:-}"

if [[ -z "$SOURCE" ]]; then
    if [[ -f "$SCRIPT_DIR/rdoctor" ]]; then
        SOURCE="$SCRIPT_DIR/rdoctor"
    else
        SOURCE="$PROJECT_ROOT/target/release/rdoctor"
    fi
fi

if [[ ! -f "$SOURCE" ]]; then
    echo "[ERROR] rdoctor binary not found: $SOURCE" >&2
    exit 1
fi

INSTALL_DIR="$HOME/.local/bin"

mkdir -p "$INSTALL_DIR"
install -m 755 "$SOURCE" "$INSTALL_DIR/rdoctor"

echo "[ OK ] Installed: $INSTALL_DIR/rdoctor"
"$INSTALL_DIR/rdoctor" --version
echo "Make sure ~/.local/bin is included in PATH."
