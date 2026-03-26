# engine-capture

Frame capture and frame-comparison utilities for regression workflows.

## Purpose

`engine-capture` serializes rendered buffers to disk and loads them back for
comparison. It underpins visual regression tests and optimization checks where
we need to detect whether two runs produced different terminal output.

## Main exports

- `FrameCapture` — writes frame data to capture files
- `load_frame()` — loads a captured frame
- `FrameHeader` — capture metadata
- `SerializedCell` — serialized per-cell payload

## Workflow

This crate is used by frame-capture tooling and tests rather than by normal
gameplay.

Typical usage flows through repository scripts and app flags such as:

```bash
cargo run -p app -- --capture-frames /tmp/frames --mod-source=mods/shell-quest-tests
```

and comparison/reporting utilities built on top of the serialized format.

## Working with this crate

- keep the file format stable unless the whole regression workflow is updated,
- if serialized cell contents change, update comparison logic and any docs that
  describe the format,
- optimize for deterministic output because this crate exists to make diffs trustworthy.
