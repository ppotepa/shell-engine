# Engine — Runtime Orchestration

## Purpose

`engine/` is the top-level runtime orchestrator. It wires subsystem crates
together, owns the frame loop, and provides engine-specific world integration.

Most domain logic now lives in focused crates:

- `engine-core` — shared types, `World`, scene model, buffer types
- `engine-scene-runtime` — mutable scene instance state
- `engine-behavior` — behavior implementations and Rhai execution model
- `engine-compositor` — composition, sprite rendering, PostFX, prerender helpers

This directory should mostly contain orchestration, provider impls, and systems
that still need engine-only world access.

## Per-frame system order

| System | Timing field | Purpose |
|---|---|---|
| input | `input_us` | SDL2 event polling |
| lifecycle | `lifecycle_us` | Scene transitions, debug shortcuts, event routing |
| animator | `animator_us` | Stage/step advancement |
| hot_reload | `hot_reload_us` | Dev-mode file scanning |
| engine_io | `engine_io_us` | Sidecar IPC bridge |
| gameplay | `-` | Physics/lifetime component update |
| collision | `-` | Gameplay collision detection |
| gameplay_events (push) | `-` | Publish collision hits for scripts |
| behavior | `behavior_us` | Behavior update, command application, `sync_widget_visuals` |
| audio_sequencer | `audio_us` | Semantic SFX/song tick and synth cue scheduling |
| audio | `audio_us` | Audio playback tick |
| compositor | `compositor_us` | Resource extraction + call into `engine-compositor` |
| postfx | `postfx_us` | PostFX application |
| renderer | `renderer_us` | Diff + flush pipeline |
| gameplay_events (clear) | `-` | Clear per-frame collision buffer |
| visual_binding | `-` | Despawn bound visuals for expired entities |
| sleep | `sleep_us` | Frame budget sleep |

Keep this order stable unless architecture is intentionally changing.

## Strategy wiring

Pipeline optimizations are selected once at startup and stored in
`PipelineStrategies`. Systems should call trait methods, not branch on flags in
hot paths.

Important flags:

- `--opt-comp` — compositor optimizations (layer scratch skip)
- `--opt-diff` — dirty-region diff
- `--opt-present` — hash-based present skipping
- `--opt-skip` — coordinated frame-skip oracle
- `--opt-rowdiff` — row-level dirty diff skip

## Engine-owned responsibilities

This directory still owns the pieces that require engine-specific world access:

- game loop and frame timing
- scene loading and scoped scene activation
- audio sequencer bridge wiring (`audio/sfx.yaml`, `audio/songs`, `audio/synth`)
- debug overlays and engine-level shortcuts
- hot reload
- renderer thread integration and frame tickets
- provider implementations that bridge `World` to subsystem crates

## Key invariants

- On scene transition or resize generation changes, reset the last accepted
  render ticket so new frames are not permanently rejected.
- Preserve dirty-region correctness through compositor, PostFX, and renderer
  handoff.
- Clear scoped resources on scene transitions before registering the new scene
  runtime and related prerender outputs.
- Keep engine wrappers thin when logic can live in a focused subsystem crate.

## Working with this directory

When changing orchestration:

- prefer moving reusable domain logic into subsystem crates,
- keep world/resource extraction here and pure logic elsewhere,
- verify transition flow, scoped cleanup, and virtual-buffer resize behavior,
- re-run `cargo test -p engine --lib`.

When changing scene lifecycle input routing, prefer runtime-owned logic in
`engine-scene-runtime` and leave engine responsible for high-level sequencing.
