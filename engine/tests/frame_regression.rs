//! Integration test for frame capture regression testing.
//! 
//! Usage:
//!   cargo test --test frame_regression -- --nocapture --ignored
//!
//! This test compares frame captures from two runs (baseline vs optimized).
//! Set environment variables to specify capture directories:
//!   FRAME_BASELINE=reports/baseline/
//!   FRAME_OPTIMIZED=reports/opt/

use engine::frame_compare;
use std::path::Path;

#[test]
#[ignore] // Run manually with --ignored flag
fn compare_frame_captures() {
    let baseline_dir = std::env::var("FRAME_BASELINE")
        .expect("Set FRAME_BASELINE env var to baseline capture directory");
    let optimized_dir = std::env::var("FRAME_OPTIMIZED")
        .expect("Set FRAME_OPTIMIZED env var to optimized capture directory");

    let baseline_path = Path::new(&baseline_dir).canonicalize()
        .expect("FRAME_BASELINE path does not exist");
    let optimized_path = Path::new(&optimized_dir).canonicalize()
        .expect("FRAME_OPTIMIZED path does not exist");

    println!("Baseline:  {}", baseline_path.display());
    println!("Optimized: {}", optimized_path.display());

    let baseline_files = frame_compare::list_frame_files(&baseline_path)
        .expect("failed to list baseline frames");
    let optimized_files = frame_compare::list_frame_files(&optimized_path)
        .expect("failed to list optimized frames");

    println!(
        "Comparing {} baseline frames with {} optimized frames",
        baseline_files.len(),
        optimized_files.len()
    );

    if baseline_files.len() != optimized_files.len() {
        panic!(
            "frame count mismatch: baseline={}, optimized={}",
            baseline_files.len(),
            optimized_files.len()
        );
    }

    let mut divergence_count = 0;
    for (baseline_entry, optimized_entry) in baseline_files.iter().zip(optimized_files.iter()) {
        let baseline_name = baseline_entry.file_name();
        let _optimized_name = optimized_entry.file_name();

        let baseline_path = baseline_entry.path();
        let optimized_path = optimized_entry.path();

        match frame_compare::compare_frames(&baseline_path, &optimized_path) {
            Ok(None) => {
                println!("  ✓ {} matches", baseline_name.to_string_lossy());
            }
            Ok(Some((cell_idx, baseline_cell, optimized_cell))) => {
                divergence_count += 1;
                let x = cell_idx % 80; // Assume 80-wide terminal
                let y = cell_idx / 80;
                eprintln!(
                    "  ✗ {} diverges at cell {} (x={}, y={})",
                    baseline_name.to_string_lossy(),
                    cell_idx,
                    x,
                    y
                );
                eprintln!("      baseline: symbol={} fg=({},{},{}) bg=({},{},{})",
                    baseline_cell.symbol,
                    baseline_cell.fg_r, baseline_cell.fg_g, baseline_cell.fg_b,
                    baseline_cell.bg_r, baseline_cell.bg_g, baseline_cell.bg_b
                );
                eprintln!("      optimized: symbol={} fg=({},{},{}) bg=({},{},{})",
                    optimized_cell.symbol,
                    optimized_cell.fg_r, optimized_cell.fg_g, optimized_cell.fg_b,
                    optimized_cell.bg_r, optimized_cell.bg_g, optimized_cell.bg_b
                );
            }
            Err(e) => {
                panic!("failed to compare {}: {}", baseline_name.to_string_lossy(), e);
            }
        }
    }

    if divergence_count > 0 {
        panic!(
            "{} frame(s) diverged between baseline and optimized",
            divergence_count
        );
    }
    println!("All frames match!");
}
