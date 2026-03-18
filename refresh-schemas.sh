#!/usr/bin/env bash
# refresh-schemas.sh — regenerate all schema fragments from engine metadata.
# Usage:
#   ./refresh-schemas.sh          # run once
#   ./refresh-schemas.sh --loop   # run every 5 seconds (Ctrl-C to stop)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

run_once() {
    echo "[$(date '+%H:%M:%S')] Regenerating schemas..."
    if cargo run -p schema-gen --quiet -- --all-mods 2>&1; then
        echo "[$(date '+%H:%M:%S')] Done."
    else
        echo "[$(date '+%H:%M:%S')] ERROR: schema-gen failed (see above)." >&2
        return 1
    fi
}

if [[ "${1:-}" == "--loop" ]]; then
    echo "Watching — regenerating every 5s. Press Ctrl-C to stop."
    while true; do
        run_once
        sleep 5
    done
else
    run_once
fi
