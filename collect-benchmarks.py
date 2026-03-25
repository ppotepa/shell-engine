#!/usr/bin/env python3
"""
Benchmark report aggregator and CSV generator.
Converts existing benchmark text reports into a comprehensive CSV spreadsheet.

Usage:
    python3 collect-benchmarks.py         # Aggregate all reports into results.csv
    python3 collect-benchmarks.py <name>  # Filter by flag combination name
"""

import sys
import csv
import re
from pathlib import Path
from datetime import datetime

REPO_ROOT = Path(__file__).parent
REPORT_DIR = REPO_ROOT / "reports" / "benchmark"

def parse_report(filepath):
    """Extract metrics from a benchmark report file."""
    try:
        with open(filepath) as f:
            content = f.read()
    except FileNotFoundError:
        return None
    
    metrics = {
        'file': filepath.name,
        'timestamp': filepath.stat().st_mtime,
    }
    
    # Extract configuration flags
    opt_comp = opt_diff = opt_skip = opt_rowdiff = "off"
    for line in content.split('\n'):
        if '--opt-comp' in line and 'ON' in line:
            opt_comp = "ON"
        if '--opt-diff' in line and 'ON' in line:
            opt_diff = "ON"
        if '--opt-skip' in line and 'ON' in line:
            opt_skip = "ON"
        if '--opt-rowdiff' in line and 'ON' in line:
            opt_rowdiff = "ON"
    
    metrics['opt_comp'] = opt_comp
    metrics['opt_diff'] = opt_diff
    metrics['opt_skip'] = opt_skip
    metrics['opt_rowdiff'] = opt_rowdiff
    
    # Extract key metrics using regex
    for line in content.split('\n'):
        # SCORE
        if 'SCORE' in line and '.' in line:
            m = re.search(r'(\d+)(?:\s|$)', line.split('SCORE')[-1])
            if m:
                metrics['score'] = m.group(1)
        # TOTAL FRAMES
        elif 'TOTAL FRAMES' in line and '.' in line:
            m = re.search(r'(\d+)', line.split('TOTAL FRAMES')[-1])
            if m:
                metrics['total_frames'] = m.group(1)
        # FPS (avg)
        elif line.strip().startswith('FPS') and 'avg=' in line:
            m = re.search(r'avg=\s*([0-9.]+)', line)
            if m:
                metrics['fps_avg'] = m.group(1)
                m = re.search(r'min=\s*([0-9.]+)', line)
                if m:
                    metrics['fps_min'] = m.group(1)
                m = re.search(r'max=\s*([0-9.]+)', line)
                if m:
                    metrics['fps_max'] = m.group(1)
                m = re.search(r'p99=\s*([0-9.]+)', line)
                if m:
                    metrics['fps_p99'] = m.group(1)
        # Frame time
        elif line.strip().startswith('Frame') and 'avg=' in line:
            m = re.search(r'avg=\s*([0-9.]+)', line)
            if m:
                metrics['frame_time_avg'] = m.group(1)
                m = re.search(r'p50=\s*([0-9.]+)', line)
                if m:
                    metrics['frame_time_p50'] = m.group(1)
                m = re.search(r'p99=\s*([0-9.]+)', line)
                if m:
                    metrics['frame_time_p99'] = m.group(1)
        # Compositor
        elif line.strip().startswith('Compositor') and 'avg=' in line:
            m = re.search(r'avg=\s*([0-9.]+)', line)
            if m:
                metrics['comp_time_avg'] = m.group(1)
        # Renderer
        elif line.strip().startswith('Renderer') and 'avg=' in line:
            m = re.search(r'avg=\s*([0-9.]+)', line)
            if m:
                metrics['rend_time_avg'] = m.group(1)
        # Behavior
        elif line.strip().startswith('Behavior') and 'avg=' in line:
            m = re.search(r'avg=\s*([0-9.]+)', line)
            if m:
                metrics['behavior_time_avg'] = m.group(1)
        # Diff cells
        elif line.strip().startswith('Diff cells') and 'avg=' in line:
            m = re.search(r'avg=\s*([0-9.]+)', line)
            if m:
                metrics['diff_cells_avg'] = m.group(1)
        # Dirty cells
        elif line.strip().startswith('Dirty cells') and 'avg=' in line:
            m = re.search(r'avg=\s*([0-9.]+)', line)
            if m:
                metrics['dirty_cells_avg'] = m.group(1)
        # Avg dirty coverage
        elif 'Avg dirty coverage' in line:
            m = re.search(r'([0-9.]+)%', line)
            if m:
                metrics['dirty_coverage_pct'] = m.group(1)
        # Avg diff coverage
        elif 'Avg diff coverage' in line:
            m = re.search(r'([0-9.]+)%', line)
            if m:
                metrics['diff_coverage_pct'] = m.group(1)
    
    # Extract per-scene breakdown (new section)
    scene_section = False
    scene_data = []
    for line in content.split('\n'):
        if 'SCENE BREAKDOWN' in line:
            scene_section = True
            continue
        if scene_section:
            if line.strip().startswith('─') or line.strip().startswith('SCENE'):
                continue
            if not line.strip():
                scene_section = False
                continue
            parts = line.split()
            if len(parts) >= 7:
                scene_data.append({
                    'scene_id': parts[0],
                    'frames': parts[1],
                    'fps_avg': parts[2],
                    'comp_us': parts[3],
                    'pfx_us': parts[4],
                    'rend_us': parts[5],
                    'bhv_us': parts[6],
                })
    
    if scene_data:
        metrics['scene_count'] = str(len(scene_data))
        metrics['scene_ids'] = ','.join(s['scene_id'] for s in scene_data)
        for s in scene_data:
            prefix = f"scene_{s['scene_id']}"
            metrics[f"{prefix}_frames"] = s['frames']
            metrics[f"{prefix}_fps"] = s['fps_avg']
            metrics[f"{prefix}_comp"] = s['comp_us']
            metrics[f"{prefix}_pfx"] = s['pfx_us']
            metrics[f"{prefix}_rend"] = s['rend_us']
    
    return metrics if len(metrics) > 2 else None

def generate_flag_name(metrics):
    """Generate a friendly flag combination name."""
    flags = []
    if metrics.get('opt_comp') == 'ON':
        flags.append('comp')
    if metrics.get('opt_diff') == 'ON':
        flags.append('diff')
    if metrics.get('opt_skip') == 'ON':
        flags.append('skip')
    if metrics.get('opt_rowdiff') == 'ON':
        flags.append('rowdiff')
    
    if not flags:
        return 'baseline'
    return '+'.join(flags)

def main():
    if not REPORT_DIR.exists():
        print(f"ERROR: Report directory not found: {REPORT_DIR}")
        return 1
    
    # Get all reports
    reports = sorted(REPORT_DIR.glob("*.txt"), key=lambda p: p.stat().st_mtime, reverse=True)
    
    if not reports:
        print(f"ERROR: No benchmark reports found in {REPORT_DIR}")
        return 1
    
    print(f"Found {len(reports)} benchmark reports")
    
    # Parse all reports
    all_metrics = []
    for report_file in reports:
        metrics = parse_report(report_file)
        if metrics:
            metrics['flag_name'] = generate_flag_name(metrics)
            all_metrics.append(metrics)
            print(f"  ✓ {report_file.name} → {metrics['flag_name']} ({metrics.get('fps_avg', '?')} FPS)")
    
    if not all_metrics:
        print("ERROR: Could not parse any reports")
        return 1
    
    # Sort by FPS descending
    all_metrics.sort(key=lambda m: float(m.get('fps_avg', 0)), reverse=True)
    
    # Generate CSV
    output_csv = REPORT_DIR / "results.csv"
    columns = [
        'flag_name', 'opt_comp', 'opt_diff', 'opt_skip', 'opt_rowdiff',
        'score', 'total_frames',
        'fps_avg', 'fps_min', 'fps_max', 'fps_p99',
        'frame_time_avg', 'frame_time_p50', 'frame_time_p99',
        'comp_time_avg', 'rend_time_avg', 'behavior_time_avg',
        'diff_cells_avg', 'dirty_cells_avg',
        'dirty_coverage_pct', 'diff_coverage_pct',
        'scene_count', 'scene_ids',
        'file',
    ]
    
    # Collect all per-scene column names dynamically
    scene_cols = set()
    for row in all_metrics:
        for k in row:
            if k.startswith('scene_') and k not in ('scene_count', 'scene_ids'):
                scene_cols.add(k)
    scene_cols = sorted(scene_cols)
    columns = columns[:-1] + scene_cols + [columns[-1]]  # insert before 'file'
    
    with open(output_csv, 'w', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=columns)
        writer.writeheader()
        for row in all_metrics:
            full_row = {col: row.get(col, '') for col in columns}
            writer.writerow(full_row)
    
    print(f"\nGenerated: {output_csv}")
    
    # Display results table
    print("\n" + "─" * 120)
    print(f"{'Flag':<20} {'Score':>6} {'FPS':>8} {'Frame (us)':>12} {'Comp (us)':>12} {'Rend (us)':>12} {'Dirty%':>8}")
    print("─" * 120)
    
    for metrics in all_metrics:
        fps = metrics.get('fps_avg', '?')
        frame = metrics.get('frame_time_avg', '?')
        comp = metrics.get('comp_time_avg', '?')
        rend = metrics.get('rend_time_avg', '?')
        dirty = metrics.get('dirty_coverage_pct', '?')
        score = metrics.get('score', '?')
        
        try:
            fps = f"{float(fps):>7.1f}"
            frame = f"{float(frame):>11.0f}"
            comp = f"{float(comp):>11.0f}"
            rend = f"{float(rend):>11.0f}"
            dirty = f"{float(dirty):>7.1f}"
            score = f"{int(score):>6}"
        except (ValueError, TypeError):
            pass
        
        print(f"{metrics['flag_name']:<20} {score} {fps} {frame} {comp} {rend} {dirty}%")
    
    print()
    print(f"✓ CSV file: {output_csv}")
    print(f"✓ Total entries: {len(all_metrics)}")
    
    return 0

if __name__ == '__main__':
    sys.exit(main())
