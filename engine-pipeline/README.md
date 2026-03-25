# engine-pipeline

Pure data struct for pipeline optimization flags.

## Purpose

Holds the set of boolean flags that control render pipeline
optimizations. Passed through the runtime to enable or disable
compositor, diff, present, skip, and row-diff optimizations
without coupling flag parsing to pipeline internals.

## Key Types

- `PipelineFlags` — struct with fields: `opt_comp`, `opt_diff`, `opt_present`, `opt_skip`, `opt_rowdiff`

## Dependencies

None — this is a standalone data crate with no external dependencies.

## Usage

Flags are set from CLI arguments:

| Flag | Effect |
|------|--------|
| `--opt-comp` | Compositor scratch skip, dirty-region halfblock |
| `--opt-diff` | DirtyRegionDiff instead of full buffer scan |
| `--opt-present` | Hash-based static frame skip |
| `--opt-skip` | Unified frame-skip oracle |
| `--opt-rowdiff` | Row-level dirty skip in diff scan |
| `--opt` | All of the above |
