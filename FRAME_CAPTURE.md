# Frame Capture Regression Testing

## Overview

Frame capture enables visual regression testing by serializing the output buffer state to binary files for frame-by-frame comparison between different rendering paths (baseline vs optimized).

This approach avoids:
- **Golden snapshots** (maintenance burden)
- **Headless harness** (doesn't test real app)
- **Manual visual inspection** (error-prone, not repeatable)

Instead, it captures the actual output from `cargo run -p app`, making it easy to verify that optimizations don't introduce visual glitches.

## Usage

### 1. Capture Baseline (Safe Defaults)

```bash
cargo run -p app -- --capture-frames reports/baseline/ --bench 5
```

This runs 5 scenes with safe defaults and saves output to `reports/baseline/frame_NNNNNN.bin`.

### 2. Capture Optimized Run

```bash
cargo run -p app -- --opt-comp --opt-present --opt-diff --capture-frames reports/optimized/ --bench 5
```

This runs the same 5 scenes with all optimizations and saves to `reports/optimized/frame_NNNNNN.bin`.

### 3. Compare Captures

```bash
export FRAME_BASELINE=reports/baseline/
export FRAME_OPTIMIZED=reports/optimized/
cargo test --test frame_regression -- --nocapture --ignored
```

The test compares frame files and reports the first diverging cell with byte-level details.

### Quick Workflow

Run the automated script:

```bash
./capture-frames.sh [baseline_dir] [optimized_dir] [frames_per_scene]
```

Example:

```bash
./capture-frames.sh reports/baseline reports/optimized 5
```

## Binary Format

Each frame file contains:

```
[width:u16 LE]
[height:u16 LE]
[cell_0...cell_N]
```

Each cell is 10 bytes:

```
[symbol:u32 LE] [fg_r:u8] [fg_g:u8] [fg_b:u8] [bg_r:u8] [bg_g:u8] [bg_b:u8]
```

Colors are serialized as 8-bit RGB triples:
- `Color::Rgb { r, g, b }` → stored as-is
- `Color::Reset` → `(0, 0, 0)`
- ANSI palette colors → expanded to standard RGB triples

## When to Use

✅ **Do use frame capture when:**
- Enabling new optimizations (benchmark performance, validate visuals)
- Changing rendering strategies
- Fixing buffer/compositor bugs
- Running CI regression checks

❌ **Don't use frame capture for:**
- Interactive testing (just run `cargo run -p app`)
- Unit tests (use existing snapshot tests in engine tests)
- Scene authoring (use editor or live preview)

## Troubleshooting

### "frame file too small" or "frame truncated"

The capture completed but file is incomplete. Check:
- Disk space
- File permissions
- App crashed mid-run (check logs)

### Frame count mismatch

Baseline and optimized runs captured different number of frames:
- Verify `--bench N` is the same for both
- Check that both runs completed successfully

### Divergence in first few frames

Splash screen or intro animations often differ. This is usually expected if you changed intro logic.
Focus on comparing scene after intro completes.

### Color values don't match

ANSI palette colors expand differently across terminals. Ensure both captures used the same terminal.
For precise results, use only `Color::Rgb` in test scenes.

## Future Enhancements

- **Visual diff output** — show diverging regions as colored terminal output or PNG
- **Partial comparison** — skip first N frames (intro/splash)
- **Tolerance modes** — allow minor color variations
- **Concurrent capture** — capture multiple scenes in parallel
- **Streaming format** — for long-running sessions

## API Reference

### `FrameCapture` (engine::frame_capture)

```rust
pub struct FrameCapture { ... }

impl FrameCapture {
    pub fn new(output_dir: impl Into<PathBuf>) -> Result<Self, EngineError>
    pub fn capture(&mut self, buffer: &Buffer) -> Result<(), EngineError>
}
```

### Frame Comparison (engine::frame_compare)

```rust
pub struct FrameHeader { pub width: u16, pub height: u16 }
pub struct SerializedCell { ... }

pub fn load_frame(path: &Path) 
    -> Result<(FrameHeader, Vec<SerializedCell>), std::io::Error>

pub fn compare_frames(path1: &Path, path2: &Path) 
    -> Result<Option<(usize, SerializedCell, SerializedCell)>, std::io::Error>

pub fn list_frame_files(dir: &Path) 
    -> Result<Vec<fs::DirEntry>, std::io::Error>
```

Returns `Some((cell_index, baseline_cell, optimized_cell))` if divergence found, or `None` if frames match.

## CLI Integration

```
cargo run -p app -- --capture-frames <DIR> [--opt-*] [--bench N]

--capture-frames <DIR>
    Enable frame capture to specified directory.
    Creates directory if it doesn't exist.
    Saves one file per frame: frame_000000.bin, frame_000001.bin, ...

--bench N
    Benchmark mode: run for N seconds and exit.
    Splash is automatically skipped.
    Designed for frame capture and benchmarking workflows.

--opt-comp, --opt-present, --opt-diff, --opt-skip, --opt-rowdiff
    Enable specific optimizations (see OPTIMIZATION_PLAN.md).

--opt
    Enable all optimizations at once.
```
