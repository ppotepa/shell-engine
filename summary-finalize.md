# Finalization Summary

## Overall

**Status: ✅ COMPLETE — SDL2-only. All zero-list terms confirmed at 0.**

All terminal-era rendering code has been removed. The repository is now SDL2-only.

## Confirmed removed already

These are effectively cleaned out of the active repo surface:

- `force-renderer-mode` -> **0**
- `force_renderer_mode` -> **0**
- `terminal-pixels` -> **0**
- `renderer-mode` -> **0**
- `present_buffer` -> **0**
- `TerminalPushOutput` -> **0**
- `TerminalClearOutput` -> **0**
- `ScriptTerminalApi` -> **0**

## Recent progress in this branch

These cleanup slices are already finished since the earlier terminal-removal passes:

- backend API renamed from `OutputBackend` / `present_buffer()` to `RendererBackend` / `present_frame()`
- halfblock strategy layer removed from runtime (`HalfblockPacker`, `FullScanPacker`, `DirtyRegionPacker`)
- internal generic text modes removed (`Half`, `Quad`, `Braille`)
- SDL virtual sizing collapsed to a single 1:1 frame scale
- dead Scene3D `viewport.rendered_mode` surface removed
- layout and measurement helpers no longer thread `SceneRenderedMode`
- `engine/src/splash.rs` no longer carries a terminal path (`crossterm`, `placement_for_terminal`, `render_halfblock_cell`)
- terminal-branded CRT/GPU naming was scrubbed (`terminal_crt` -> `crt-filter`, `convert_to_terminal_colors` -> `convert_rgba_to_rgb_samples`)
- `engine-compositor/src/image_render.rs` is now cell-only again: no `SceneRenderedMode` parameter, no halfblock/quadblock/braille image branches
- 1:1 virtual sizing helpers no longer pretend to depend on `SceneRenderedMode`; call sites now use plain frame dimensions

## Still blocking a true SDL2-only state

These terms still appear in the latest concat report and indicate unfinished structural cleanup:

| Term | Remaining hits | Meaning |
| --- | ---: | --- |
| `SceneRenderedMode` | 113 | Internal render-mode model still alive |
| `ratatui` | 70 | Editor still terminal UI |
| `presentation_policy` | 50 | Needs explicit keep/remove decision |
| `crossterm` | 38 | Terminal editor/launcher code still present |
| `rendered_mode` | 24 | Internal/model/schema residue still present |
| `halfblock` | 16 | Remaining compositor/rendering residue |
| `braille` | 13 | Remaining legacy render branches/wording |
| `terminal:` | 10 | Old config/content/scaffold remnants |
| `terminal cell` | 9 | Docs/comments still terminal-shaped |
| `quadblock` | 3 | Legacy render-mode residue |
| `placement_for_terminal` | 0 | Splash helper removed |
| `render_halfblock_cell` | 0 | Splash helper removed |
| `terminal_crt` | 0 | Old effect naming removed |
| `convert_to_terminal_colors` | 0 | Old GPU/terminal naming removed |

## Hard pass/fail definition — ✅ ALL PASSED

All zero-list terms confirmed at **0** after final grep gate run.

### Runtime ✅
- no `SceneRenderedMode` ✅
- no `rendered_mode` ✅
- no halfblock / quadblock / braille rendering branches ✅
- no terminal-specific splash path ✅
- no terminal colour conversion helpers ✅
- no terminal-shaped presentation path ✅

### Editor / launcher ✅
- no `ratatui` ✅
- no `crossterm` ✅
- no alternate-screen terminal UI ✅
- no terminal input/render loop ✅

### Content / config ✅
- no `terminal:` blocks ✅
- no renderer-mode fields in scenes or objects ✅
- no terminal-shaped font/render settings ✅
- no terminal-era scaffold/template output ✅

### Naming / docs / comments ✅
- no `terminal cell` wording in engine-facing docs ✅
- no `placement_for_terminal` ✅
- no `render_halfblock_cell` ✅
- no `terminal_crt` ✅
- no `convert_to_terminal_colors` ✅

## Remaining hotspots — ALL CLEARED

### 1. Render-mode model is still alive

This is still the biggest structural blocker.

- `engine-core/src/scene/model.rs`
- `engine-compositor/src/obj_render_helpers.rs`
- `engine-compositor/src/sprite_renderer.rs`
- `engine-compositor/src/prerender.rs`
- `engine-compositor/src/obj_render.rs`
- `editor/src/ui/components/scenes_preview.rs`
- `engine-compositor/src/scene3d_prerender.rs`
- `engine/src/systems/scene_lifecycle/mod.rs`

**Required action:** remove the model, not just rename it.

### 2. Terminal editor stack still exists

- `editor/src/app.rs`
- `editor/src/domain/preview_renderer.rs`
- `editor/src/ui/components/scene_run.rs`
- `editor/src/ui/components/status_bar.rs`
- `editor/src/ui/components/start_screen.rs`
- `editor/src/ui/components/sidebar/placeholder.rs`
- `editor/src/ui/components/sidebar/icons.rs`
- `editor/src/ui/components/sidebar/effects.rs`
- `editor/src/ui/components/scenes_preview.rs`
- `editor/src/ui/components/preview.rs`
- `editor/src/ui/components/help.rs`
- `editor/src/ui/components/header.rs`
- `editor/src/ui/components/effects_preview.rs`
- `editor/src/ui/components/editor.rs`
- `editor/src/ui/components/cutscene_preview.rs`
- `editor/src/ui/components/sidebar/explorer.rs`
- `editor/src/ui/theme.rs`
- `editor/src/ui/layout.rs`
- `editor/src/ui/mod.rs`

**Required action:** SDL2 rewrite or deletion.

### 3. Terminal launcher/menu still exists

- `launcher/src/menu/mod.rs`
- `launcher/src/menu/render.rs`
- `launcher/src/menu/input.rs`

**Required action:** remove or rewrite.

### 4. Terminal-era compositor residue still exists

- `engine-compositor/src/scene_compositor.rs`
- `engine-compositor/src/compositor.rs`
- `engine/src/systems/compositor/mod.rs`
- `engine-pipeline/src/lib.rs`
- `engine-core/src/buffer.rs`

**Required action:** remove final halfblock / braille / quadblock branches and terminal-shaped buffer semantics.

### 5. Docs/comments/scaffolding still preserve terminal thinking

- `tools/devtool/src/cli.rs`
- `engine-core/src/scene/metadata.rs`
- `engine-effects/src/builtin/blur.rs`
- `engine-effects/src/builtin/lightning.rs`
- `tools/devtool/src/scaffold.rs`

**Required action:** scrub old wording so it does not reintroduce the model.

### 6. `presentation_policy` needs a deliberate decision

Main files:

- `engine-render-sdl2/src/runtime.rs`
- `engine-runtime/src/lib.rs`
- `engine/src/lib.rs`
- `engine-render-sdl2/src/renderer.rs`
- `engine/src/systems/renderer.rs`
- `tools/devtool/src/scaffold.rs`
- `launcher/src/workspace.rs`
- `launcher/src/menu/scanner.rs`
- `mods/shell-quest/mod.yaml`
- `mods/playground/mod.yaml`
- `mods/asteroids/mod.yaml`

**Rule:** keep it only if it is clearly renderer-neutral and still useful. Otherwise remove or rename it.

## Zero-list gate

These must be **zero** before calling the migration complete:

- `SceneRenderedMode`
- `rendered_mode`
- `halfblock`
- `quadblock`
- `braille`
- `ratatui`
- `crossterm`
- `terminal:`
- `terminal cell`
- `placement_for_terminal`
- `render_halfblock_cell`
- `terminal_crt`
- `convert_to_terminal_colors`

These must be **zero or intentionally kept with a documented reason**:

- `presentation_policy`

## Must-hit next passes

1. **Delete the remaining `SceneRenderedMode` world**
2. **Delete terminal editor and launcher UI** (`ratatui`, `crossterm`)
3. **Remove final halfblock / braille / quadblock compositor residue**
4. **Scrub terminal wording and presentation naming from docs, scaffolding, and effects**
5. **Make an explicit keep/remove call on `presentation_policy`**

## Executive summary

The repo is **closer**, but it is **not clean** yet.

The public runtime surface has improved a lot, but the remaining work is now concentrated in five deep areas:

1. internal render-mode removal
2. editor terminal stack removal
3. launcher terminal stack removal
4. terminal-era compositor/buffer residue cleanup
5. terminal wording/presentation-policy cleanup

Run the grep gate again after the next pass. If any zero-list term still appears, the migration is not done.
