#!/bin/bash
#
# Comprehensive benchmarking suite for Shell Quest engine optimizations.
# Tests all flag combinations across 3 scenarios (5 secs each).
# Generates detailed CSV report to reports/benchmark/results.csv
#

set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MOD_SOURCE="${REPO_ROOT}/mods/shell-quest-tests"
REPORT_DIR="${REPO_ROOT}/reports/benchmark"
OUTPUT_CSV="${REPORT_DIR}/results.csv"

# Ensure report directory exists
mkdir -p "${REPORT_DIR}"

# ANSI colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# ─────────────────────────────────────────────────────────────────────
# Scenario definitions: (name, description, duration_secs)
# ─────────────────────────────────────────────────────────────────────

declare -A SCENARIOS=(
    ["quick"]="Quick test,2"
    ["standard"]="Standard benchmark,5"
    ["extended"]="Extended benchmark,10"
)

# ─────────────────────────────────────────────────────────────────────
# Flag combinations to test
# Format: name|--flags
# ─────────────────────────────────────────────────────────────────────

FLAGS=(
    "baseline|"
    "opt-comp|--opt-comp"
    "opt-diff|--opt-diff"
    "opt-skip|--opt-skip"
    "opt-rowdiff|--opt-rowdiff"
    "opt-comp+diff|--opt-comp --opt-diff"
    "opt-comp+skip|--opt-comp --opt-skip"
    "opt-comp+rowdiff|--opt-comp --opt-rowdiff"
    "opt-all|--opt"
)

# ─────────────────────────────────────────────────────────────────────
# Extract metrics from benchmark report file
# ─────────────────────────────────────────────────────────────────────

extract_metrics() {
    local report_file="$1"
    
    if [[ ! -f "$report_file" ]]; then
        echo "ERROR:Report not found"
        return 1
    fi
    
    # Extract key metrics using grep + awk
    local score=$(grep "SCORE" "$report_file" | awk -F'[.]+' '{print $NF}' | xargs)
    local total_frames=$(grep "TOTAL FRAMES" "$report_file" | awk -F'[.]+' '{print $NF}' | xargs)
    local fps_avg=$(grep "^  FPS" "$report_file" | awk '{print $3}' | sed 's/=//')
    local frame_avg=$(grep "^  Frame" "$report_file" | awk '{print $3}' | sed 's/=//')
    local comp_avg=$(grep "^  Compositor" "$report_file" | awk '{print $3}' | sed 's/=//')
    local renderer_avg=$(grep "^  Renderer" "$report_file" | awk '{print $3}' | sed 's/=//')
    local diff_cells=$(grep "^  Diff cells" "$report_file" | awk '{print $3}' | sed 's/=//')
    local dirty_cells=$(grep "^  Dirty cells" "$report_file" | awk '{print $3}' | sed 's/=//')
    
    echo "${score}|${total_frames}|${fps_avg}|${frame_avg}|${comp_avg}|${renderer_avg}|${diff_cells}|${dirty_cells}"
}

# ─────────────────────────────────────────────────────────────────────
# Run single benchmark and return results
# ─────────────────────────────────────────────────────────────────────

run_benchmark() {
    local flag_name="$1"
    local flags="$2"
    local duration="$3"
    local scenario="$4"
    
    log_info "Running: ${flag_name} (${flags:-none}) for ${duration}s [${scenario}]"
    
    # Build command
    local cmd="cargo run --release -p app -- --mod-source=${MOD_SOURCE} --bench ${duration}"
    if [[ -n "$flags" ]]; then
        cmd="${cmd} ${flags}"
    fi
    
    # Run and capture output
    local output
    output=$($cmd 2>&1)
    
    # Get latest benchmark report
    local latest_report
    latest_report=$(ls -t "${REPORT_DIR}"/*.txt 2>/dev/null | head -1)
    
    if [[ ! -f "$latest_report" ]]; then
        log_error "No benchmark report generated for ${flag_name}"
        echo "ERROR"
        return 1
    fi
    
    # Extract metrics
    local metrics
    metrics=$(extract_metrics "$latest_report")
    
    if [[ "$metrics" == "ERROR"* ]]; then
        log_error "Failed to extract metrics from ${latest_report}"
        echo "ERROR"
        return 1
    fi
    
    log_success "${flag_name}: $(echo $metrics | cut -d'|' -f3) FPS"
    echo "$metrics"
}

# ─────────────────────────────────────────────────────────────────────
# Main execution
# ─────────────────────────────────────────────────────────────────────

main() {
    log_info "Shell Quest Benchmark Suite"
    log_info "Mod source: ${MOD_SOURCE}"
    log_info "Output: ${OUTPUT_CSV}"
    echo ""
    
    # Verify mod exists
    if [[ ! -d "$MOD_SOURCE" ]]; then
        log_error "Mod source not found: ${MOD_SOURCE}"
        exit 1
    fi
    
    # Choose scenario
    local scenario="standard"
    local scenario_duration=5
    
    # Parse scenario from command line if provided
    if [[ $# -gt 0 ]]; then
        if [[ -v SCENARIOS[$1] ]]; then
            scenario=$1
            scenario_duration=$(echo "${SCENARIOS[$1]}" | cut -d',' -f2)
            shift
        else
            log_error "Unknown scenario: $1"
            echo "Available scenarios:"
            for s in "${!SCENARIOS[@]}"; do
                echo "  - $s: $(echo "${SCENARIOS[$s]}" | cut -d',' -f1)"
            done
            exit 1
        fi
    fi
    
    local scenario_desc=$(echo "${SCENARIOS[$scenario]}" | cut -d',' -f1)
    log_info "Scenario: ${scenario} (${scenario_desc}, ${scenario_duration}s per test)"
    echo ""
    
    # Build engine first
    log_info "Building engine in release mode..."
    cargo build --release -p app >/dev/null 2>&1
    log_success "Build complete"
    echo ""
    
    # Initialize CSV header
    echo "Scenario,Flag Combination,Score,Total Frames,Avg FPS,Frame Time (us),Compositor (us),Renderer (us),Diff Cells,Dirty Cells" > "${OUTPUT_CSV}"
    
    # Run benchmarks
    log_info "Running benchmark suite..."
    echo ""
    
    local total_tests=${#FLAGS[@]}
    local current_test=0
    
    for flag_spec in "${FLAGS[@]}"; do
        ((current_test++))
        local flag_name=$(echo "$flag_spec" | cut -d'|' -f1)
        local flags=$(echo "$flag_spec" | cut -d'|' -f2-)
        
        echo "[${current_test}/${total_tests}] Testing: ${flag_name}"
        
        # Run benchmark
        local metrics
        metrics=$(run_benchmark "$flag_name" "$flags" "$scenario_duration" "$scenario")
        
        if [[ "$metrics" != "ERROR" ]]; then
            # Append to CSV
            echo "${scenario},${flag_name},${metrics}" >> "${OUTPUT_CSV}"
        fi
        
        sleep 1  # Brief pause between runs
    done
    
    echo ""
    log_success "Benchmark suite complete!"
    log_info "Results saved to: ${OUTPUT_CSV}"
    echo ""
    
    # Display CSV
    echo "─── CSV Results ───"
    column -t -s',' "${OUTPUT_CSV}"
    echo ""
    
    # Generate summary stats
    log_info "Generating summary statistics..."
    echo ""
    
    local baseline_fps=$(grep "^standard,baseline," "${OUTPUT_CSV}" | cut -d',' -f5)
    echo "Baseline FPS: ${baseline_fps}"
    echo ""
    echo "Performance vs Baseline:"
    awk -F',' 'NR>2 {
        if ($3 != "ERROR") {
            flag=$2
            fps=$5
            if (fps+0 > 0 && baseline+0 > 0) {
                pct = ((fps - baseline) / baseline) * 100
                if (pct > 0) symbol="↑"
                else if (pct < 0) symbol="↓"
                else symbol="="
                printf "  %-20s %6.1f FPS  %s%5.1f%%\n", flag, fps, symbol, pct
            }
        }
    }' baseline="$baseline_fps" "${OUTPUT_CSV}"
    
    echo ""
    log_success "All done!"
}

# Run main
main "$@"
