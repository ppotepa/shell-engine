#!/bin/bash
# Benchmark script for CHUNK 43-50 optimizations
# Runs baseline, intermediate, and fully optimized versions and generates comparison report

set -e

cd /home/ppotepa/git/shell-quest

REPORTS_DIR="./benchmark-results"
mkdir -p "$REPORTS_DIR"

# Score formula: (fps.avg * 10) + (1_000_000 / frame.p50 * 5) - (frame.p99 / 100)

echo "=== CHUNK 43-50 Benchmark Suite ==="
echo "Starting benchmarks at $(date)"

# Test configuration
MOD_SOURCE="mods/shell-quest-tests"
BENCH_DURATION=10
TARGET_FPS=120

# Baseline (no optimizations)
echo -e "\n[1/3] Running BASELINE (no optimizations)..."
./target/release/app --mod-source="$MOD_SOURCE" --bench "$BENCH_DURATION" --target-fps="$TARGET_FPS" 2>&1 | tee "$REPORTS_DIR/baseline.log" &
BASELINE_PID=$!
wait $BASELINE_PID || true

# All optimizations enabled
echo -e "\n[2/3] Running OPTIMIZED (--opt-comp --opt-diff --opt-rowdiff --opt-present --opt-skip)..."
./target/release/app --mod-source="$MOD_SOURCE" --bench "$BENCH_DURATION" --target-fps="$TARGET_FPS" \
  --opt-comp --opt-diff --opt-rowdiff --opt-present --opt-skip 2>&1 | tee "$REPORTS_DIR/optimized.log" &
OPTIMIZED_PID=$!
wait $OPTIMIZED_PID || true

# Extract scores from logs
BASELINE_SCORE=$(grep -i "^Score:" "$REPORTS_DIR/baseline.log" | awk '{print $NF}' | head -1 || echo "N/A")
OPTIMIZED_SCORE=$(grep -i "^Score:" "$REPORTS_DIR/optimized.log" | awk '{print $NF}' | head -1 || echo "N/A")

echo -e "\n=== Benchmark Results ==="
echo "Baseline Score:   $BASELINE_SCORE"
echo "Optimized Score:  $OPTIMIZED_SCORE"

# Try to compute percentage improvement if scores are numeric
if [[ "$BASELINE_SCORE" =~ ^[0-9]+([.][0-9]+)?$ ]] && [[ "$OPTIMIZED_SCORE" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
    IMPROVEMENT=$(echo "scale=2; (($OPTIMIZED_SCORE - $BASELINE_SCORE) / $BASELINE_SCORE) * 100" | bc 2>/dev/null || echo "N/A")
    echo "Improvement:      $IMPROVEMENT%"
fi

echo -e "\nBenchmark artifacts saved to: $REPORTS_DIR/"
echo "Complete at $(date)"
