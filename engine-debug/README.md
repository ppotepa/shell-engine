# engine-debug

Debug overlay rendering, log buffer, and memory statistics.

## Purpose

Provides runtime diagnostic overlays drawn on top of the rendered scene.
Includes a stats overlay (scene ID, virtual size, errors), a scrollable
log overlay, and memory usage tracking. Activated with `--debug-feature`.

## Key Types

- `DebugOverlay` — renders diagnostic information over the scene output
- `DebugLogBuffer` — ring buffer collecting runtime log entries and script errors
- `MemoryStats` — queries process RSS and heap usage via libc

## Dependencies

- `crossterm` — terminal drawing primitives for overlay rendering
- `libc` — system calls for memory usage queries

## Usage

Launch with `--debug-feature`, then use hotkeys:
- **F1** — toggle Stats overlay
- **~ / `** — toggle Logs overlay
