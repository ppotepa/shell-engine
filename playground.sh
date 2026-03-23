#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
exec ./launcher.sh --mod playground --sound-server "$@"
