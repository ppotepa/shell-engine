# Status Summary

> **HISTORICAL**: Terminal cleanup is 100% complete as of 2026-04-13. All
> terminal-era rendering code (`SceneRenderedMode`, halfblock/quadblock/braille
> modes, `HalfblockPacker`/`FullScanPacker`/`DirtyRegionPacker`, `ratatui`,
> `crossterm`, `DisplaySink`/`DisplayFrame`, `terminal_crt`, `terminal-input`,
> etc.) has been removed. `engine-render-sdl2` is the only renderer backend.
> This document reflects the partial state at an earlier point in the migration.

## Overall

**Status: partial cleanup complete, broader SDL2-only migration still unfinished.**

The repo now has a working `mods/terrain-playground` mod with an SDL-style main menu at **640x360**. The first option routes to **plain terrain** and the other four entries are placeholders.

The renderer cleanup is **not fully finished**. I removed several active runtime/config paths, but terminal-era architecture still exists in multiple crates and in the editor.

## Done in this pass

| Area | Status | Notes |
| --- | --- | --- |
| `mods/terrain-playground` | Done | Added SDL menu scene, 5 options, first option implemented, remaining options placeholders, asteroids-style presentation. |
| CLI/runtime renderer override | Done | Removed `--renderer-mode` support from `app`, removed runtime renderer override parsing from `engine-runtime`, removed related engine config usage. |
| Sprite forced renderer override | Done | Removed `force_renderer_mode` fields from sprite model/runtime usage and cleaned matching compositor/prerender call sites. |
| Shell Quest font assets | Done | Replaced deleted `terminal-pixels` asset usage with `raster` manifests/assets for Shell Quest fonts. |
| Font loader runtime fallback | Done | Removed `terminal-pixels` fallback logic from `engine-render/src/font_loader.rs`; Rust code no longer references `terminal-pixels`. |
| Authored render-mode surface | Done | `rendered-mode` and `force-renderer-mode` were removed from metadata/root schemas, scene deserialization no longer accepts `rendered-mode`, and object-viewer mode-switch hotkeys were removed. |
| Terminal-mode tests/docs cleanup | Done | Removed stale render-mode test expectations and cleaned editor preview/schema text that still described `terminal-pixels` or halfblock-specific behavior. |
| Launcher/runtime surface | Done | Removed launcher `--renderer-mode` forwarding and deleted the dead `lock_renderer_mode_to_scene` pipeline flag. |
| Terminal scripting API | Done | Removed `TerminalApi`, `TerminalPushOutput`, and `TerminalClearOutput` from behavior scripting/runtime command handling. |
| Devtool scaffold config | Done | New mod scaffolds now emit `display:` with SDL-style render sizing instead of the legacy `terminal:` block. |
| Public generic font modes | Done | Removed authored `generic:half|quad|braille` usage from mod content/tests, removed public normalization for those aliases, and regenerated mod schemas. |
| Internal generic renderer modes | Done | Removed internal `GenericMode::{Half, Quad, Braille}` support, deleted their rasterizers, collapsed compositor line metrics to tiny/standard/large, and switched the perf HUD to the standard generic font. |
| Halfblock strategy wiring | Done | Removed `HalfblockPacker`, `FullScanPacker`, and `DirtyRegionPacker`, dropped halfblock strategy wiring from `PipelineStrategies` and the active compositor system, simplified `pack_halfblock_buffer` to a plain legacy helper used only by tests, and removed the dead cell-vs-halfblock scene compositor strategy API. |
| Backend API naming | Done | Renamed `OutputBackend` to `RendererBackend`, renamed `present_buffer()` to `present_frame()`, and updated the engine, SDL2 backend, splash path, and nearby docs to use renderer-neutral presentation language. |
| SDL virtual sizing cleanup | Done | Collapsed SDL-facing virtual size helpers to a single 1:1 frame scale, so pixel-canvas and OBJ/image virtual dimensions no longer expand from terminal render modes. |
| Scene3D render-mode surface | Done | Removed the dead `viewport.rendered_mode` field from `engine-3d` parsing and `schemas/scene3d.schema.yaml`. |
| Core terminal wording cleanup | Done | Updated core buffer/backend/prepared-frame/schema wording to stop pointing at terminal presentation or `terminal.render_size`. |
| Focused Rust tests | Done | `cargo test -q -p engine-render -p engine -p engine-core -p engine-render-policy -p engine-runtime -p engine-authoring` is green. |

## Still not finished

| Area | Status | Remaining work |
| --- | --- | --- |
| Scene renderer model | In progress | `SceneRenderedMode` is weaker than before and no longer drives SDL virtual sizing or Scene3D viewport parsing, but it still exists internally and is still threaded through compositor/layout/runtime code. |
| Halfblock/compositor pipeline | In progress | The halfblock strategy layer is gone from runtime, but halfblock-specific rendering branches and packing semantics still exist inside compositor/image/object code. |
| Generic text/image modes | In progress | Generic text-mode support is now limited to tiny/standard/large, but terminal-era image/object helper paths and related terminology still remain elsewhere. |
| Terminal authoring helpers | In progress | `terminal-input` sugar was removed, but other terminal-shaped authoring/helpers still exist. |
| Editor terminal stack | Not started | `editor` still uses `crossterm`/`ratatui` and still contains terminal preview logic. |
| Launcher terminal UI | Not started | `launcher/src/menu/*` is still a `crossterm` alternate-screen menu even though renderer-mode forwarding is gone. |
| Splash cleanup | Not started | `engine/src/splash.rs` still contains terminal/ANSI/halfblock codepaths. |
| Backend naming refactor | In progress | The live backend API is now renderer-neutral, but the frame payload is still `Buffer`-shaped and SDL2 still translates from cell diffs plus optional pixel canvas data. |
| Docs/schemas cleanup | In progress | Root docs and some comments still mention terminal, halfblock, braille, and related concepts. |
| Terminal-era tests | In progress | The explicit mode-switch tests were removed, but broader terminal-era coverage around compositor/render paths still exists. |

## Important notes

1. This is **not** a full SDL2-only conversion yet.
2. The codebase is in a better state for that migration: the most immediate runtime renderer override and `terminal-pixels` font fallback paths are gone.
3. The next heavy lifts are the **halfblock pipeline removal**, **internal `SceneRenderedMode` removal**, and the **editor rewrite/removal**.
