#!/usr/bin/env python3
"""
Comprehensive benchmarking suite for Shell Quest engine optimizations.
Tests all flag combinations and generates detailed CSV reports.

Usage:
    python3 benchmark.py quick       # 2 sec per test
    python3 benchmark.py standard    # 5 sec per test  
    python3 benchmark.py extended    # 10 sec per test
"""

import os
import sys
import subprocess
import csv
import re
import time
from pathlib import Path
from datetime import datetime
from collections import defaultdict

# Configuration
REPO_ROOT = Path(__file__).parent
MOD_SOURCE = REPO_ROOT / "mods" / "shell-quest-tests"
REPORT_DIR = REPO_ROOT / "reports" / "benchmark"

SCENARIOS = {
    "quick": (2, "Quick test (2s per test)"),
    "standard": (5, "Standard benchmark (5s per test)"),
    "extended": (10, "Extended benchmark (10s per test)"),
}

# All flag combinations to test
# Format: (name, flags_string)
FLAG_COMBINATIONS = [
    ("baseline", ""),
    ("opt-comp", "--opt-comp"),
    ("opt-diff", "--opt-diff"),
    ("opt-skip", "--opt-skip"),
    ("opt-rowdiff", "--opt-rowdiff"),
    ("opt-comp+diff", "--opt-comp --opt-diff"),
    ("opt-comp+skip", "--opt-comp --opt-skip"),
    ("opt-comp+rowdiff", "--opt-comp --opt-rowdiff"),
    ("opt-skip+rowdiff", "--opt-skip --opt-rowdiff"),
    ("opt-diff+skip", "--opt-diff --opt-skip"),
    ("opt-all", "--opt"),
]

# ANSI colors
class Color:
    BLUE = '\033[0;34m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    RED = '\033[0;31m'
    RESET = '\033[0m'

def log_info(msg):
    print(f"{Color.BLUE}[INFO]{Color.RESET} {msg}")

def log_ok(msg):
    print(f"{Color.GREEN}[OK]{Color.RESET} {msg}")

def log_warn(msg):
    print(f"{Color.YELLOW}[WARN]{Color.RESET} {msg}")

def log_error(msg):
    print(f"{Color.RED}[ERROR]{Color.RESET} {msg}", file=sys.stderr)

def parse_benchmark_report(filepath):
    """Extract metrics from a benchmark report file."""
    try:
        with open(filepath) as f:
            content = f.read()
    except FileNotFoundError:
        return None
    
    metrics = {}
    for line in content.split('\n'):
        # SCORE
        if 'SCORE' in line and '.' in line:
            parts = line.split()
            metrics['score'] = parts[-1]
        # TOTAL FRAMES
        elif 'TOTAL FRAMES' in line and '.' in line:
            parts = line.split()
            metrics['frames'] = parts[-1]
        # FPS (avg)
        elif line.strip().startswith('FPS') and 'avg=' in line:
            m = re.search(r'avg=\s*([0-9.]+)', line)
            if m:
                metrics['fps_avg'] = m.group(1)
        # Frame time
        elif line.strip().startswith('Frame') and 'avg=' in line:
            m = re.search(r'avg=\s*([0-9.]+)', line)
            if m:
                metrics['frame_time'] = m.group(1)
        # Compositor
        elif line.strip().startswith('Compositor') and 'avg=' in line:
            m = re.search(r'avg=\s*([0-9.]+)', line)
            if m:
                metrics['comp_time'] = m.group(1)
        # Renderer
        elif line.strip().startswith('Renderer') and 'avg=' in line:
            m = re.search(r'avg=\s*([0-9.]+)', line)
            if m:
                metrics['rend_time'] = m.group(1)
        # Diff cells
        elif line.strip().startswith('Diff cells') and 'avg=' in line:
            m = re.search(r'avg=\s*([0-9.]+)', line)
            if m:
                metrics['diff_cells'] = m.group(1)
        # Dirty cells
        elif line.strip().startswith('Dirty cells') and 'avg=' in line:
            m = re.search(r'avg=\s*([0-9.]+)', line)
            if m:
                metrics['dirty_cells'] = m.group(1)
        # Avg dirty coverage
        elif 'Avg dirty coverage' in line:
            m = re.search(r'([0-9.]+)%', line)
            if m:
                metrics['dirty_pct'] = m.group(1)
        # Avg diff coverage
        elif 'Avg diff coverage' in line:
            m = re.search(r'([0-9.]+)%', line)
            if m:
                metrics['diff_pct'] = m.group(1)
    
    return metrics if metrics else None

def get_latest_report():
    """Get the most recently created benchmark report."""
    reports = sorted(REPORT_DIR.glob("*.txt"), reverse=True)
    return reports[0] if reports else None

def run_benchmark(flags, duration, name):
    """Run a single benchmark with the given flags."""
    log_info(f"Testing: {name} ({flags or 'baseline'}) for {duration}s...")
    
    cmd = [
        "cargo", "run", "--release", "-p", "app", "--",
        f"--mod-source={MOD_SOURCE}",
        f"--bench={duration}",
        "--skip-splash",
    ]
    
    if flags:
        cmd.extend(flags.split())
    
    try:
        # Run with timeout, suppress output
        subprocess.run(
            cmd,
            cwd=REPO_ROOT,
            timeout=duration + 30,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
    except subprocess.TimeoutExpired:
        log_error(f"Benchmark timed out for {name}")
        return None
    except Exception as e:
        log_error(f"Benchmark failed for {name}: {e}")
        return None
    
    # Give filesystem time to write
    time.sleep(1)
    
    # Parse the latest report
    latest = get_latest_report()
    if latest:
        metrics = parse_benchmark_report(latest)
        if metrics:
            log_ok(f"{name}: {metrics.get('fps_avg', '?')} FPS")
            return metrics
    
    log_warn(f"Could not parse report for {name}")
    return None

def main():
    log_info("Shell Quest Benchmark Suite")
    log_info(f"Mod source: {MOD_SOURCE}")
    
    # Parse arguments
    if len(sys.argv) > 1 and sys.argv[1] in SCENARIOS:
        scenario_name = sys.argv[1]
    else:
        scenario_name = "standard"
    
    duration, scenario_desc = SCENARIOS[scenario_name]
    log_info(f"Scenario: {scenario_name} - {scenario_desc}")
    
    # Ensure directories exist
    REPORT_DIR.mkdir(parents=True, exist_ok=True)
    
    # Build engine
    log_info("Building engine in release mode...")
    try:
        subprocess.run(
            ["cargo", "build", "--release", "-p", "app"],
            cwd=REPO_ROOT,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=300,
        )
        log_ok("Build complete")
    except Exception as e:
        log_error(f"Build failed: {e}")
        return 1
    
    print()
    
    # Run benchmarks
    results = []
    total = len(FLAG_COMBINATIONS)
    for i, (name, flags) in enumerate(FLAG_COMBINATIONS, 1):
        print(f"[{i}/{total}]", end=" ")
        metrics = run_benchmark(flags, duration, name)
        
        if metrics:
            results.append({
                'scenario': scenario_name,
                'name': name,
                'flags': flags,
                **metrics,
            })
        
        time.sleep(1)  # Brief pause between runs
    
    # Generate CSV
    output_csv = REPORT_DIR / "results.csv"
    csv_columns = [
        'scenario', 'name', 'flags',
        'score', 'frames', 'fps_avg', 'frame_time',
        'comp_time', 'rend_time', 'diff_cells', 'dirty_cells',
        'dirty_pct', 'diff_pct',
    ]
    
    with open(output_csv, 'w', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=csv_columns)
        writer.writeheader()
        for row in results:
            # Fill in missing fields with empty string
            full_row = {col: row.get(col, '') for col in csv_columns}
            writer.writerow(full_row)
    
    log_ok(f"Results saved to: {output_csv}")
    print()
    
    # Display results
    print("─" * 100)
    print(f"{'Flag Combination':<25} {'FPS':>8} {'Frame (us)':>12} {'Comp (us)':>12} {'Rend (us)':>12} {'Diff%':>8}")
    print("─" * 100)
    
    for row in results:
        fps = row.get('fps_avg', '?')
        frame_time = row.get('frame_time', '?')
        comp_time = row.get('comp_time', '?')
        rend_time = row.get('rend_time', '?')
        diff_pct = row.get('diff_pct', '?')
        print(f"{row['name']:<25} {float(fps):>8.1f} {float(frame_time):>12.1f} {float(comp_time):>12.1f} {float(rend_time):>12.1f} {diff_pct:>7}%")
    
    print()
    
    # Calculate improvements
    if results:
        baseline = next((r for r in results if r['name'] == 'baseline'), None)
        if baseline:
            baseline_fps = float(baseline.get('fps_avg', 0))
            print(f"Performance vs Baseline ({baseline_fps:.1f} FPS):")
            print("─" * 60)
            for row in results:
                if row['name'] != 'baseline':
                    fps = float(row.get('fps_avg', 0))
                    if baseline_fps > 0:
                        improvement = ((fps - baseline_fps) / baseline_fps) * 100
                        symbol = "↑" if improvement > 0 else "↓" if improvement < 0 else "="
                        print(f"  {row['name']:<22} {fps:>7.1f} FPS  {symbol}{abs(improvement):>5.1f}%")
    
    print()
    log_ok("All done!")
    
    return 0

if __name__ == '__main__':
    sys.exit(main())
