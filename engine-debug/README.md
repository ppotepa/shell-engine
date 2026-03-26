# engine-debug

Runtime debug flags, log buffering, and lightweight performance stats.

## Purpose

`engine-debug` holds runtime-only diagnostics that can be enabled without
changing the build profile. It tracks overlay state, recent log messages, FPS,
per-system timings, and sampled process statistics.

## Key types

- `DebugLogBuffer` — recent runtime/script log entries
- `DebugFeatures` — debug toggle state and overlay visibility
- `DebugOverlayMode` — stats vs logs view
- `FpsCounter` — smoothed FPS sample
- `SystemTimings` — smoothed per-system timings
- `ProcessStats` — sampled CPU and RSS statistics

## Working with this crate

- keep debug helpers cheap when disabled,
- prefer pushing meaningful runtime diagnostics here instead of scattering ad-hoc debug prints,
- if new debug UI modes are added, update the launcher/help docs and keybinding docs too.
