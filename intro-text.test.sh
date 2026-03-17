#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
export SHELL_QUEST_MOD_SOURCE="${SHELL_QUEST_MOD_SOURCE:-$(pwd)/mods/intro-text.test}"
exec ./launcher.sh "$@"
