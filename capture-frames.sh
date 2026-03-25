#!/bin/bash

# Frame capture workflow script
# Captures frames from baseline and optimized runs for regression testing

set -e

BASELINE_DIR="${1:-reports/baseline}"
OPTIMIZED_DIR="${2:-reports/optimized}"
BENCH_FRAMES="${3:-5}"

echo "🎬 Frame Capture Workflow"
echo "========================"
echo ""
echo "Baseline dir:  $BASELINE_DIR"
echo "Optimized dir: $OPTIMIZED_DIR"
echo "Frames/scene:  $BENCH_FRAMES"
echo ""

# Capture baseline (safe defaults)
echo "📷 Capturing baseline frames (safe defaults)..."
rm -rf "$BASELINE_DIR"
timeout 30 cargo run -p app -- --capture-frames "$BASELINE_DIR" --bench "$BENCH_FRAMES" 2>&1 | tail -20 || true
echo "✓ Baseline captured: $(ls -1 "$BASELINE_DIR" 2>/dev/null | wc -l) frames"

echo ""

# Capture optimized (all optimizations enabled)
echo "📷 Capturing optimized frames (--opt-comp --opt-present --opt-diff)..."
rm -rf "$OPTIMIZED_DIR"
timeout 30 cargo run -p app -- --opt-comp --opt-present --opt-diff --capture-frames "$OPTIMIZED_DIR" --bench "$BENCH_FRAMES" 2>&1 | tail -20 || true
echo "✓ Optimized captured: $(ls -1 "$OPTIMIZED_DIR" 2>/dev/null | wc -l) frames"

echo ""

# Compare frames
echo "🔍 Comparing frame captures..."
export FRAME_BASELINE="$BASELINE_DIR"
export FRAME_OPTIMIZED="$OPTIMIZED_DIR"
if cargo test --test frame_regression -- --nocapture --ignored 2>&1 | tee /tmp/frame_compare.log; then
    echo ""
    echo "✅ All frames match! Optimizations are visually equivalent."
else
    echo ""
    echo "❌ Frame mismatch detected. See details above."
    exit 1
fi
