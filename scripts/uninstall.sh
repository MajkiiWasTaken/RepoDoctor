#!/usr/bin/env bash
# File: uninstall.sh
# Author: Michal Švrček
# RepoDoctor Linux uninstaller.
# ver. 0.6.0

set -euo pipefail

rm -f -- "$HOME/.local/bin/rdoctor"

echo "[ OK ] RepoDoctor uninstalled."
echo "Project configuration files were preserved."
