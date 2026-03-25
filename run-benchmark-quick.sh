#!/bin/bash
# Quick benchmark runner that executes one benchmark and returns CSV metrics

set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MOD_SOURCE="${REPO_ROOT}/mods/shell-quest-tests"
REPORT_DIR="${REPO_ROOT}/reports/benchmark"

mkdir -p "${REPORT_DIR}"

# Get timestamp before run
BEFORE=$(date +%s)

# Run benchmark with timeout and skip splash
echo "Running benchmark..." >&2
timeout 60 cargo run --release -p app -- \
    --mod-source="${MOD_SOURCE}" \
    --bench 2 \
    --skip-splash < /dev/null 2>/dev/null || true

# Wait a moment for file to be written
sleep 1

# Get latest benchmark report
LATEST_REPORT=$(ls -t "${REPORT_DIR}"/*.txt 2>/dev/null | head -1)

if [[ ! -f "$LATEST_REPORT" ]]; then
    echo "ERROR: No benchmark report found" >&2
    exit 1
fi

# Extract metrics
extract_metric() {
    local pattern="$1"
    grep "$pattern" "$LATEST_REPORT" | head -1 | grep -oE '[0-9]+\.?[0-9]*' | head -1
}

SCORE=$(extract_metric "SCORE")
FRAMES=$(extract_metric "TOTAL FRAMES")
FPS=$(grep "^  FPS" "$LATEST_REPORT" | awk '{print $3}' | sed 's/=//')
FRAMETIME=$(grep "^  Frame" "$LATEST_REPORT" | awk '{print $3}' | sed 's/=//')
COMP=$(grep "^  Compositor" "$LATEST_REPORT" | awk '{print $3}' | sed 's/=//')
REND=$(grep "^  Renderer" "$LATEST_REPORT" | awk '{print $3}' | sed 's/=//')
DIFF=$(grep "^  Diff cells" "$LATEST_REPORT" | awk '{print $3}' | sed 's/=//')

echo "$SCORE|$FRAMES|$FPS|$FRAMETIME|$COMP|$REND|$DIFF"
