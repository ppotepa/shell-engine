# engine-capture

Frame capture and comparison for regression testing.

## Purpose

Captures rendered frames to disk as structured data so they can be
compared across runs. Used by the benchmarking and regression testing
tools to detect visual regressions in scene output.

## Key Types

- `FrameCapture` — serializes a rendered buffer to a capture file
- `FrameHeader` — metadata (scene ID, dimensions, timestamp) stored with each capture
- `SerializedCell` — per-cell representation (char, fg, bg, attributes)
- `compare_frames()` — diffs two captured frames and reports cell-level changes
- `load_frame()` — deserializes a capture file back into memory

## Dependencies

- `engine-core` — buffer and cell types
- `engine-error` — shared error types
- `crossterm` — terminal style types used in cell serialization

## Usage

```bash
# Capture frames from a test run
./capture-frames.sh
```
