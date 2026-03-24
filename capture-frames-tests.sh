#!/bin/bash

# Frame capture workflow script for shell-quest-tests (no user input required)
# Captures frames from baseline and test mod optimized runs for regression testing

set -e

BASELINE_DIR="${1:-reports/baseline-tests}"
OPTIMIZED_DIR="${2:-reports/optimized-tests}"
BENCH_FRAMES="${3:-5}"

echo "🎬 Frame Capture Workflow (shell-quest-tests, no user input)"
echo "============================================================"
echo ""
echo "Baseline dir:  $BASELINE_DIR"
echo "Optimized dir: $OPTIMIZED_DIR"
echo "Frames/scene:  $BENCH_FRAMES"
echo ""

# Capture baseline (safe defaults)
echo "📷 Capturing baseline frames (safe defaults)..."
rm -rf "$BASELINE_DIR"
timeout 60 cargo run -p app --release -- --capture-frames "$BASELINE_DIR" --bench "$BENCH_FRAMES" 2>&1 | tail -10 || true
frames_baseline=$(ls -1 "$BASELINE_DIR" 2>/dev/null | wc -l)
echo "✓ Baseline captured: $frames_baseline frames"

echo ""

# Capture optimized (all optimizations enabled)
echo "📷 Capturing optimized frames (--opt-comp --opt-present --opt-diff)..."
rm -rf "$OPTIMIZED_DIR"
timeout 60 cargo run -p app --release -- --opt-comp --opt-present --opt-diff --capture-frames "$OPTIMIZED_DIR" --bench "$BENCH_FRAMES" 2>&1 | tail -10 || true
frames_optimized=$(ls -1 "$OPTIMIZED_DIR" 2>/dev/null | wc -l)
echo "✓ Optimized captured: $frames_optimized frames"

echo ""

# Compare frames
echo "🔍 Comparing frame captures..."
export FRAME_BASELINE="$BASELINE_DIR"
export FRAME_OPTIMIZED="$OPTIMIZED_DIR"
if cargo test --test frame_regression -p engine -- --nocapture --ignored 2>&1 | tee /tmp/frame_compare.log; then
    echo ""
    echo "✅ All frames match! Optimizations are visually equivalent."
else
    echo ""
    echo "❌ Frame mismatch detected. See details above."
    exit 1
fi
