# engine-render-terminal

Terminal presentation backend and display strategies.

## Purpose

`engine-render-terminal` owns the concrete terminal-output path: converting a
prepared buffer into terminal writes, batching ANSI output, and optionally
offloading display flush work.

## Key modules and exports

- `renderer` — `TerminalRenderer`, `renderer_system()`, color resolution, flush helpers
- `strategy` — sync/async display sink and batching strategies
- `provider` — renderer provider integration
- `rasterizer` — terminal-oriented rasterization helpers
- `RendererProvider`
- `AnsiBatchFlusher`
- `AsyncDisplaySink`
- `NaiveFlusher`
- `SyncDisplaySink`

## Working with this crate

- keep presentation concerns here instead of drifting back into the engine loop,
- if terminal write behavior changes, verify startup/shutdown cleanup and async display paths,
- coordinate changes with `engine-render` trait contracts and pipeline strategy wiring.
