# Codebase simplification review

- Files reviewed: 366
- Total Rust LOC reviewed: 91503
- Files with at least one simplification signal: 76
- Conservative removable LOC estimate while preserving behaviour: 6564 to 9436
- Largest structural duplicate clusters found: 11 high-similarity pairs (>=0.98 similarity)

## Highest-yield targets

- `engine-behavior/src/lib.rs` — est. cut 917 LOC; Oversized file (10098 lines); split orchestration/data/parsing paths and collapse repeated setup branches. | Contains very large function(s) (max ~1049 lines); extract table-driven handlers/helpers to reduce control-flow sprawl. | High count of long functions (25); repeated branch/error/render logic is likely compressible.
- `engine-render-terminal/src/rasterizer/generic.rs` — est. cut 887 LOC; Near-duplicate module with `engine-render/src/generic.rs` (similarity 0.997); move shared implementation to one crate/module and re-export thinly. | Oversized file (949 lines); split orchestration/data/parsing paths and collapse repeated setup branches. | Contains very large function(s) (max ~278 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.
- `engine-render/src/generic.rs` — est. cut 887 LOC; Near-duplicate module with `engine-render-terminal/src/rasterizer/generic.rs` (similarity 0.997); move shared implementation to one crate/module and re-export thinly. | Oversized file (955 lines); split orchestration/data/parsing paths and collapse repeated setup branches. | Contains very large function(s) (max ~278 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.
- `engine-authoring/src/schema/mod.rs` — est. cut 477 LOC; Oversized file (3383 lines); split orchestration/data/parsing paths and collapse repeated setup branches. | Contains very large function(s) (max ~179 lines); extract table-driven handlers/helpers to reduce control-flow sprawl. | High count of long functions (14); repeated branch/error/render logic is likely compressible.
- `engine-authoring/src/document/scene.rs` — est. cut 467 LOC; Oversized file (2606 lines); split orchestration/data/parsing paths and collapse repeated setup branches. | Contains very large function(s) (max ~219 lines); extract table-driven handlers/helpers to reduce control-flow sprawl. | High count of long functions (11); repeated branch/error/render logic is likely compressible.
- `engine-authoring/src/compile/scene.rs` — est. cut 368 LOC; Oversized file (2387 lines); split orchestration/data/parsing paths and collapse repeated setup branches. | High count of long functions (8); repeated branch/error/render logic is likely compressible. | Branch-heavy module (96 ifs, 24 matches); candidate for declarative tables, enum methods, or strategy objects.
- `engine-compositor/src/obj_render.rs` — est. cut 364 LOC; Oversized file (1862 lines); split orchestration/data/parsing paths and collapse repeated setup branches. | Contains very large function(s) (max ~357 lines); extract table-driven handlers/helpers to reduce control-flow sprawl. | High count of long functions (8); repeated branch/error/render logic is likely compressible.
- `engine-core/src/effects/builtin/lightning.rs` — est. cut 317 LOC; Oversized file (1801 lines); split orchestration/data/parsing paths and collapse repeated setup branches. | Contains very large function(s) (max ~256 lines); extract table-driven handlers/helpers to reduce control-flow sprawl. | High count of long functions (7); repeated branch/error/render logic is likely compressible.
- `engine/src/scene3d_format.rs` — est. cut 305 LOC; Near-duplicate module with `engine-3d/src/scene3d_format.rs` (similarity 0.997); move shared implementation to one crate/module and re-export thinly.
- `engine-3d/src/scene3d_format.rs` — est. cut 305 LOC; Near-duplicate module with `engine/src/scene3d_format.rs` (similarity 0.997); move shared implementation to one crate/module and re-export thinly.
- `engine/src/rasterizer/font_loader.rs` — est. cut 299 LOC; Near-duplicate module with `engine-render/src/font_loader.rs` (similarity 0.997); move shared implementation to one crate/module and re-export thinly.
- `engine-render/src/font_loader.rs` — est. cut 299 LOC; Near-duplicate module with `engine/src/rasterizer/font_loader.rs` (similarity 0.997); move shared implementation to one crate/module and re-export thinly.
- `engine/src/image_loader.rs` — est. cut 287 LOC; Near-duplicate module with `engine-render/src/image_loader.rs` (similarity 0.982); move shared implementation to one crate/module and re-export thinly.
- `engine-render/src/image_loader.rs` — est. cut 287 LOC; Near-duplicate module with `engine/src/image_loader.rs` (similarity 0.982); move shared implementation to one crate/module and re-export thinly.
- `engine/src/splash.rs` — est. cut 265 LOC; Oversized file (1822 lines); split orchestration/data/parsing paths and collapse repeated setup branches. | High count of long functions (10); repeated branch/error/render logic is likely compressible.
- `engine-game/src/game_state.rs` — est. cut 264 LOC; Near-duplicate module with `engine-core/src/game_state.rs` (similarity 1.000); move shared implementation to one crate/module and re-export thinly.
- `engine-core/src/game_state.rs` — est. cut 264 LOC; Near-duplicate module with `engine-game/src/game_state.rs` (similarity 1.000); move shared implementation to one crate/module and re-export thinly.
- `engine/src/scene3d_resolve.rs` — est. cut 222 LOC; Near-duplicate module with `engine-3d/src/scene3d_resolve.rs` (similarity 1.000); move shared implementation to one crate/module and re-export thinly.
- `engine-3d/src/scene3d_resolve.rs` — est. cut 222 LOC; Near-duplicate module with `engine/src/scene3d_resolve.rs` (similarity 1.000); move shared implementation to one crate/module and re-export thinly.
- `engine-render-sdl2/src/runtime.rs` — est. cut 197 LOC; Oversized file (1217 lines); split orchestration/data/parsing paths and collapse repeated setup branches. | Contains very large function(s) (max ~267 lines); extract table-driven handlers/helpers to reduce control-flow sprawl. | High count of long functions (5); repeated branch/error/render logic is likely compressible.
- `engine-core/src/authoring/catalog.rs` — est. cut 187 LOC; Oversized file (1407 lines); split orchestration/data/parsing paths and collapse repeated setup branches. | Contains very large function(s) (max ~853 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.
- `editor/src/state/mod.rs` — est. cut 187 LOC; Oversized file (939 lines); split orchestration/data/parsing paths and collapse repeated setup branches. | Contains very large function(s) (max ~171 lines); extract table-driven handlers/helpers to reduce control-flow sprawl. | High count of long functions (5); repeated branch/error/render logic is likely compressible.
- `engine-compositor/src/sprite_renderer.rs` — est. cut 181 LOC; Oversized file (1200 lines); split orchestration/data/parsing paths and collapse repeated setup branches. | Contains very large function(s) (max ~941 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.
- `engine-scene-runtime/src/behavior_runner.rs` — est. cut 152 LOC; Oversized file (1052 lines); split orchestration/data/parsing paths and collapse repeated setup branches. | Contains very large function(s) (max ~350 lines); extract table-driven handlers/helpers to reduce control-flow sprawl. | Clone-heavy code (39 clones); some allocations/copies can likely be removed with borrowing/Cow/Arc reuse.
- `engine/src/systems/scene_lifecycle.rs` — est. cut 150 LOC; Oversized file (1513 lines); split orchestration/data/parsing paths and collapse repeated setup branches. | Contains very large function(s) (max ~152 lines); extract table-driven handlers/helpers to reduce control-flow sprawl. | High count of long functions (4); repeated branch/error/render logic is likely compressible.

## Per-file findings

### `app/src/main.rs`

- Size: 602 LOC; functions: 30; max fn span: 144 LOC; if/match: 11/5; clones: 2
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/app.rs`

- Size: 135 LOC; functions: 1; max fn span: 109 LOC; if/match: 12/1; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/cli.rs`

- Size: 22 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/domain/asset_index.rs`

- Size: 18 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/domain/diagnostics.rs`

- Size: 8 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/domain/effect_params.rs`

- Size: 446 LOC; functions: 41; max fn span: 78 LOC; if/match: 7/9; clones: 5
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/domain/effects_catalog.rs`

- Size: 65 LOC; functions: 4; max fn span: 23 LOC; if/match: 5/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/domain/effects_preview_scene.rs`

- Size: 519 LOC; functions: 18; max fn span: 141 LOC; if/match: 8/5; clones: 6
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/domain/mod.rs`

- Size: 11 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/domain/mod_manifest.rs`

- Size: 12 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/domain/preview_renderer.rs`

- Size: 115 LOC; functions: 4; max fn span: 32 LOC; if/match: 3/1; clones: 1
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/domain/scene_index.rs`

- Size: 8 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/input/commands.rs`

- Size: 38 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/input/keys.rs`

- Size: 91 LOC; functions: 1; max fn span: 83 LOC; if/match: 9/5; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/input/mod.rs`

- Size: 5 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/io/fs_scan.rs`

- Size: 807 LOC; functions: 37; max fn span: 96 LOC; if/match: 35/11; clones: 1
- Estimated removable LOC from this file/refactor area: 40
  - Oversized file (807 lines); split orchestration/data/parsing paths and collapse repeated setup branches.

### `editor/src/io/indexer.rs`

- Size: 89 LOC; functions: 2; max fn span: 40 LOC; if/match: 1/1; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/io/mod.rs`

- Size: 7 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/io/recent.rs`

- Size: 86 LOC; functions: 5; max fn span: 34 LOC; if/match: 7/0; clones: 1
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/io/yaml.rs`

- Size: 13 LOC; functions: 1; max fn span: 5 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/main.rs`

- Size: 51 LOC; functions: 2; max fn span: 26 LOC; if/match: 0/1; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/state/cutscene.rs`

- Size: 196 LOC; functions: 0; max fn span: 0 LOC; if/match: 18/3; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/state/editor_pane.rs`

- Size: 88 LOC; functions: 0; max fn span: 0 LOC; if/match: 2/3; clones: 4
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/state/effects_browser.rs`

- Size: 207 LOC; functions: 4; max fn span: 161 LOC; if/match: 15/1; clones: 0
- Estimated removable LOC from this file/refactor area: 24
  - Contains very large function(s) (max ~161 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.

### `editor/src/state/filters.rs`

- Size: 9 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/state/focus.rs`

- Size: 30 LOC; functions: 2; max fn span: 9 LOC; if/match: 0/2; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/state/mod.rs`

- Size: 939 LOC; functions: 27; max fn span: 171 LOC; if/match: 20/25; clones: 0
- Estimated removable LOC from this file/refactor area: 187
  - Oversized file (939 lines); split orchestration/data/parsing paths and collapse repeated setup branches.
  - Contains very large function(s) (max ~171 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.
  - High count of long functions (5); repeated branch/error/render logic is likely compressible.
  - Branch-heavy module (20 ifs, 25 matches); candidate for declarative tables, enum methods, or strategy objects.

### `editor/src/state/project_explorer.rs`

- Size: 66 LOC; functions: 1; max fn span: 28 LOC; if/match: 5/1; clones: 3
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/state/scene_run.rs`

- Size: 371 LOC; functions: 11; max fn span: 98 LOC; if/match: 23/4; clones: 3
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/state/scenes_browser.rs`

- Size: 337 LOC; functions: 0; max fn span: 0 LOC; if/match: 30/2; clones: 1
- Estimated removable LOC from this file/refactor area: 0
  - Large data/model module but not obviously redundant from text-level scan; review for generated/schema-like content before manual refactor.

### `editor/src/state/selection.rs`

- Size: 9 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/state/start_screen.rs`

- Size: 508 LOC; functions: 7; max fn span: 276 LOC; if/match: 16/13; clones: 9
- Estimated removable LOC from this file/refactor area: 41
  - Contains very large function(s) (max ~276 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.

### `editor/src/state/watch.rs`

- Size: 164 LOC; functions: 2; max fn span: 67 LOC; if/match: 11/3; clones: 1
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/ui/components/cutscene_preview.rs`

- Size: 85 LOC; functions: 1; max fn span: 74 LOC; if/match: 5/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/ui/components/editor.rs`

- Size: 47 LOC; functions: 1; max fn span: 36 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/ui/components/effects_preview.rs`

- Size: 465 LOC; functions: 13; max fn span: 89 LOC; if/match: 17/7; clones: 1
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/ui/components/header.rs`

- Size: 43 LOC; functions: 1; max fn span: 32 LOC; if/match: 1/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/ui/components/help.rs`

- Size: 53 LOC; functions: 2; max fn span: 21 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/ui/components/mod.rs`

- Size: 14 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/ui/components/preview.rs`

- Size: 175 LOC; functions: 5; max fn span: 45 LOC; if/match: 2/4; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/ui/components/scene_run.rs`

- Size: 43 LOC; functions: 1; max fn span: 30 LOC; if/match: 1/1; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/ui/components/scenes_preview.rs`

- Size: 311 LOC; functions: 7; max fn span: 116 LOC; if/match: 22/3; clones: 1
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/ui/components/sidebar/effects.rs`

- Size: 68 LOC; functions: 1; max fn span: 55 LOC; if/match: 4/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/ui/components/sidebar/explorer.rs`

- Size: 75 LOC; functions: 1; max fn span: 63 LOC; if/match: 1/1; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/ui/components/sidebar/icons.rs`

- Size: 49 LOC; functions: 1; max fn span: 38 LOC; if/match: 1/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/ui/components/sidebar/mod.rs`

- Size: 7 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/ui/components/sidebar/placeholder.rs`

- Size: 31 LOC; functions: 1; max fn span: 20 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/ui/components/start_screen.rs`

- Size: 636 LOC; functions: 12; max fn span: 180 LOC; if/match: 24/4; clones: 8
- Estimated removable LOC from this file/refactor area: 27
  - Contains very large function(s) (max ~180 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.

### `editor/src/ui/components/status_bar.rs`

- Size: 43 LOC; functions: 1; max fn span: 32 LOC; if/match: 1/0; clones: 1
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/ui/icons.rs`

- Size: 132 LOC; functions: 5; max fn span: 69 LOC; if/match: 0/4; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/ui/layout.rs`

- Size: 66 LOC; functions: 1; max fn span: 50 LOC; if/match: 1/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/ui/mod.rs`

- Size: 133 LOC; functions: 1; max fn span: 121 LOC; if/match: 13/2; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `editor/src/ui/theme.rs`

- Size: 190 LOC; functions: 23; max fn span: 17 LOC; if/match: 7/1; clones: 0
- Estimated removable LOC from this file/refactor area: 66
  - Wrapper-heavy API surface (23 short functions across 190 lines); macro/generic accessors can remove boilerplate.

### `engine-3d/src/lib.rs`

- Size: 19 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-3d/src/obj_frame_cache.rs`

- Size: 63 LOC; functions: 5; max fn span: 15 LOC; if/match: 0/0; clones: 0
- Duplication signal: near-duplicate of `engine/src/obj_frame_cache.rs` (similarity 1.000, shared body roughly 63 LOC)
- Estimated removable LOC from this file/refactor area: 53
  - Near-duplicate module with `engine/src/obj_frame_cache.rs` (similarity 1.000); move shared implementation to one crate/module and re-export thinly.

### `engine-3d/src/obj_prerender.rs`

- Size: 60 LOC; functions: 5; max fn span: 14 LOC; if/match: 0/0; clones: 0
- Duplication signal: near-duplicate of `engine-compositor/src/obj_prerender.rs` (similarity 1.000, shared body roughly 60 LOC)
- Estimated removable LOC from this file/refactor area: 51
  - Near-duplicate module with `engine-compositor/src/obj_prerender.rs` (similarity 1.000); move shared implementation to one crate/module and re-export thinly.

### `engine-3d/src/scene3d_atlas.rs`

- Size: 88 LOC; functions: 7; max fn span: 16 LOC; if/match: 2/0; clones: 0
- Duplication signal: near-duplicate of `engine-compositor/src/scene3d_atlas.rs` (similarity 1.000, shared body roughly 88 LOC)
- Estimated removable LOC from this file/refactor area: 74
  - Near-duplicate module with `engine-compositor/src/scene3d_atlas.rs` (similarity 1.000); move shared implementation to one crate/module and re-export thinly.

### `engine-3d/src/scene3d_format.rs`

- Size: 359 LOC; functions: 12; max fn span: 67 LOC; if/match: 4/2; clones: 0
- Duplication signal: near-duplicate of `engine/src/scene3d_format.rs` (similarity 0.997, shared body roughly 359 LOC)
- Estimated removable LOC from this file/refactor area: 305
  - Near-duplicate module with `engine/src/scene3d_format.rs` (similarity 0.997); move shared implementation to one crate/module and re-export thinly.

### `engine-3d/src/scene3d_resolve.rs`

- Size: 262 LOC; functions: 13; max fn span: 100 LOC; if/match: 7/8; clones: 3
- Duplication signal: near-duplicate of `engine/src/scene3d_resolve.rs` (similarity 1.000, shared body roughly 262 LOC)
- Estimated removable LOC from this file/refactor area: 222
  - Near-duplicate module with `engine/src/scene3d_resolve.rs` (similarity 1.000); move shared implementation to one crate/module and re-export thinly.

### `engine-animation/src/access.rs`

- Size: 20 LOC; functions: 4; max fn span: 5 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-animation/src/animator.rs`

- Size: 65 LOC; functions: 5; max fn span: 13 LOC; if/match: 1/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-animation/src/lib.rs`

- Size: 19 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-animation/src/menu.rs`

- Size: 98 LOC; functions: 6; max fn span: 43 LOC; if/match: 9/1; clones: 2
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-animation/src/provider.rs`

- Size: 20 LOC; functions: 9; max fn span: 3 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-animation/src/systems.rs`

- Size: 337 LOC; functions: 14; max fn span: 92 LOC; if/match: 11/2; clones: 4
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-asset/src/lib.rs`

- Size: 18 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-asset/src/repositories.rs`

- Size: 939 LOC; functions: 53; max fn span: 59 LOC; if/match: 18/7; clones: 12
- Estimated removable LOC from this file/refactor area: 40
  - Oversized file (939 lines); split orchestration/data/parsing paths and collapse repeated setup branches.

### `engine-asset/src/scene_compiler.rs`

- Size: 31 LOC; functions: 1; max fn span: 15 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-asset/src/source_loader.rs`

- Size: 113 LOC; functions: 7; max fn span: 21 LOC; if/match: 0/2; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-audio-sequencer/src/lib.rs`

- Size: 882 LOC; functions: 27; max fn span: 82 LOC; if/match: 46/3; clones: 1
- Estimated removable LOC from this file/refactor area: 40
  - Oversized file (882 lines); split orchestration/data/parsing paths and collapse repeated setup branches.

### `engine-audio/src/access.rs`

- Size: 20 LOC; functions: 4; max fn span: 5 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-audio/src/audio.rs`

- Size: 419 LOC; functions: 25; max fn span: 77 LOC; if/match: 10/5; clones: 3
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-audio/src/lib.rs`

- Size: 16 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-audio/src/systems_audio.rs`

- Size: 61 LOC; functions: 4; max fn span: 21 LOC; if/match: 2/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-authoring/src/compile/cutscene.rs`

- Size: 446 LOC; functions: 17; max fn span: 125 LOC; if/match: 20/2; clones: 6
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-authoring/src/compile/mod.rs`

- Size: 16 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-authoring/src/compile/scene.rs`

- Size: 2387 LOC; functions: 73; max fn span: 95 LOC; if/match: 96/24; clones: 31
- Estimated removable LOC from this file/refactor area: 368
  - Oversized file (2387 lines); split orchestration/data/parsing paths and collapse repeated setup branches.
  - High count of long functions (8); repeated branch/error/render logic is likely compressible.
  - Branch-heavy module (96 ifs, 24 matches); candidate for declarative tables, enum methods, or strategy objects.
  - Clone-heavy code (31 clones); some allocations/copies can likely be removed with borrowing/Cow/Arc reuse.

### `engine-authoring/src/document/mod.rs`

- Size: 13 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-authoring/src/document/object.rs`

- Size: 44 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-authoring/src/document/scene.rs`

- Size: 2606 LOC; functions: 60; max fn span: 219 LOC; if/match: 69/45; clones: 26
- Estimated removable LOC from this file/refactor area: 467
  - Oversized file (2606 lines); split orchestration/data/parsing paths and collapse repeated setup branches.
  - Contains very large function(s) (max ~219 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.
  - High count of long functions (11); repeated branch/error/render logic is likely compressible.
  - Branch-heavy module (69 ifs, 45 matches); candidate for declarative tables, enum methods, or strategy objects.
  - Clone-heavy code (26 clones); some allocations/copies can likely be removed with borrowing/Cow/Arc reuse.

### `engine-authoring/src/document/value.rs`

- Size: 23 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-authoring/src/lib.rs`

- Size: 17 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-authoring/src/package/mod.rs`

- Size: 305 LOC; functions: 17; max fn span: 39 LOC; if/match: 7/0; clones: 2
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-authoring/src/repository/mod.rs`

- Size: 81 LOC; functions: 7; max fn span: 22 LOC; if/match: 4/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-authoring/src/schema/mod.rs`

- Size: 3383 LOC; functions: 97; max fn span: 179 LOC; if/match: 92/10; clones: 2
- Estimated removable LOC from this file/refactor area: 477
  - Oversized file (3383 lines); split orchestration/data/parsing paths and collapse repeated setup branches.
  - Contains very large function(s) (max ~179 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.
  - High count of long functions (14); repeated branch/error/render logic is likely compressible.
  - Branch-heavy module (92 ifs, 10 matches); candidate for declarative tables, enum methods, or strategy objects.

### `engine-authoring/src/validate/mod.rs`

- Size: 221 LOC; functions: 6; max fn span: 67 LOC; if/match: 5/1; clones: 2
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-behavior-registry/src/lib.rs`

- Size: 5 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-behavior-registry/src/provider.rs`

- Size: 3 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-behavior/src/catalog.rs`

- Size: 322 LOC; functions: 7; max fn span: 117 LOC; if/match: 23/0; clones: 10
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-behavior/src/factory.rs`

- Size: 52 LOC; functions: 2; max fn span: 30 LOC; if/match: 11/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-behavior/src/lib.rs`

- Size: 10098 LOC; functions: 377; max fn span: 1049 LOC; if/match: 316/40; clones: 216
- Estimated removable LOC from this file/refactor area: 917
  - Oversized file (10098 lines); split orchestration/data/parsing paths and collapse repeated setup branches.
  - Contains very large function(s) (max ~1049 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.
  - High count of long functions (25); repeated branch/error/render logic is likely compressible.
  - Branch-heavy module (316 ifs, 40 matches); candidate for declarative tables, enum methods, or strategy objects.
  - Clone-heavy code (216 clones); some allocations/copies can likely be removed with borrowing/Cow/Arc reuse.

### `engine-behavior/src/registry/mod.rs`

- Size: 227 LOC; functions: 13; max fn span: 45 LOC; if/match: 7/5; clones: 1
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-behavior/src/registry/provider.rs`

- Size: 19 LOC; functions: 11; max fn span: 3 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-capture/src/capture.rs`

- Size: 133 LOC; functions: 6; max fn span: 30 LOC; if/match: 1/1; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-capture/src/compare.rs`

- Size: 163 LOC; functions: 4; max fn span: 60 LOC; if/match: 5/0; clones: 4
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-capture/src/lib.rs`

- Size: 6 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/access.rs`

- Size: 41 LOC; functions: 9; max fn span: 3 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/buffer_pool.rs`

- Size: 236 LOC; functions: 17; max fn span: 26 LOC; if/match: 4/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/compositor.rs`

- Size: 300 LOC; functions: 8; max fn span: 88 LOC; if/match: 7/2; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/effect_applicator.rs`

- Size: 378 LOC; functions: 7; max fn span: 88 LOC; if/match: 16/8; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/image_render.rs`

- Size: 472 LOC; functions: 19; max fn span: 68 LOC; if/match: 13/6; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/layer_compositor.rs`

- Size: 178 LOC; functions: 1; max fn span: 157 LOC; if/match: 10/1; clones: 0
- Estimated removable LOC from this file/refactor area: 23
  - Contains very large function(s) (max ~157 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.

### `engine-compositor/src/layout/area.rs`

- Size: 56 LOC; functions: 2; max fn span: 17 LOC; if/match: 0/2; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/layout/flex.rs`

- Size: 242 LOC; functions: 6; max fn span: 123 LOC; if/match: 3/3; clones: 1
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/layout/grid.rs`

- Size: 284 LOC; functions: 5; max fn span: 170 LOC; if/match: 6/2; clones: 1
- Estimated removable LOC from this file/refactor area: 25
  - Contains very large function(s) (max ~170 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.

### `engine-compositor/src/layout/measure.rs`

- Size: 293 LOC; functions: 0; max fn span: 0 LOC; if/match: 7/4; clones: 1
- Estimated removable LOC from this file/refactor area: 0
  - Large data/model module but not obviously redundant from text-level scan; review for generated/schema-like content before manual refactor.

### `engine-compositor/src/layout/mod.rs`

- Size: 15 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/layout/tracks.rs`

- Size: 155 LOC; functions: 4; max fn span: 121 LOC; if/match: 12/1; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/lib.rs`

- Size: 66 LOC; functions: 2; max fn span: 5 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/obj_loader.rs`

- Size: 416 LOC; functions: 14; max fn span: 111 LOC; if/match: 28/1; clones: 1
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/obj_prerender.rs`

- Size: 60 LOC; functions: 5; max fn span: 14 LOC; if/match: 0/0; clones: 0
- Duplication signal: near-duplicate of `engine-3d/src/obj_prerender.rs` (similarity 1.000, shared body roughly 60 LOC)
- Estimated removable LOC from this file/refactor area: 51
  - Near-duplicate module with `engine-3d/src/obj_prerender.rs` (similarity 1.000); move shared implementation to one crate/module and re-export thinly.

### `engine-compositor/src/obj_render.rs`

- Size: 1862 LOC; functions: 44; max fn span: 357 LOC; if/match: 106/6; clones: 0
- Estimated removable LOC from this file/refactor area: 364
  - Oversized file (1862 lines); split orchestration/data/parsing paths and collapse repeated setup branches.
  - Contains very large function(s) (max ~357 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.
  - High count of long functions (8); repeated branch/error/render logic is likely compressible.
  - Branch-heavy module (106 ifs, 6 matches); candidate for declarative tables, enum methods, or strategy objects.

### `engine-compositor/src/prerender.rs`

- Size: 259 LOC; functions: 3; max fn span: 153 LOC; if/match: 3/1; clones: 4
- Estimated removable LOC from this file/refactor area: 22
  - Contains very large function(s) (max ~153 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.

### `engine-compositor/src/provider.rs`

- Size: 14 LOC; functions: 6; max fn span: 3 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/render/common.rs`

- Size: 133 LOC; functions: 0; max fn span: 0 LOC; if/match: 6/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/render/containers.rs`

- Size: 69 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/render/mod.rs`

- Size: 11 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/scene3d_atlas.rs`

- Size: 88 LOC; functions: 7; max fn span: 16 LOC; if/match: 2/0; clones: 0
- Duplication signal: near-duplicate of `engine-3d/src/scene3d_atlas.rs` (similarity 1.000, shared body roughly 88 LOC)
- Estimated removable LOC from this file/refactor area: 74
  - Near-duplicate module with `engine-3d/src/scene3d_atlas.rs` (similarity 1.000); move shared implementation to one crate/module and re-export thinly.

### `engine-compositor/src/scene3d_prerender.rs`

- Size: 486 LOC; functions: 9; max fn span: 130 LOC; if/match: 9/4; clones: 7
- Estimated removable LOC from this file/refactor area: 48
  - High count of long functions (4); repeated branch/error/render logic is likely compressible.

### `engine-compositor/src/scene_compositor.rs`

- Size: 57 LOC; functions: 1; max fn span: 18 LOC; if/match: 0/1; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/sprite_renderer.rs`

- Size: 1200 LOC; functions: 8; max fn span: 941 LOC; if/match: 37/4; clones: 0
- Estimated removable LOC from this file/refactor area: 181
  - Oversized file (1200 lines); split orchestration/data/parsing paths and collapse repeated setup branches.
  - Contains very large function(s) (max ~941 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.

### `engine-compositor/src/systems/mod.rs`

- Size: 2 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/systems/postfx/glow.rs`

- Size: 332 LOC; functions: 5; max fn span: 142 LOC; if/match: 20/3; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/systems/postfx/mod.rs`

- Size: 371 LOC; functions: 6; max fn span: 111 LOC; if/match: 16/6; clones: 3
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/systems/postfx/pass_burn_in.rs`

- Size: 294 LOC; functions: 6; max fn span: 132 LOC; if/match: 12/0; clones: 2
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/systems/postfx/pass_crt.rs`

- Size: 536 LOC; functions: 3; max fn span: 426 LOC; if/match: 26/1; clones: 0
- Estimated removable LOC from this file/refactor area: 63
  - Contains very large function(s) (max ~426 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.

### `engine-compositor/src/systems/postfx/pass_crt_distort.rs`

- Size: 129 LOC; functions: 2; max fn span: 28 LOC; if/match: 5/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/systems/postfx/pass_ruby_crt.rs`

- Size: 116 LOC; functions: 0; max fn span: 0 LOC; if/match: 4/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/systems/postfx/pass_scan_glitch.rs`

- Size: 85 LOC; functions: 0; max fn span: 0 LOC; if/match: 3/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/systems/postfx/pass_underlay.rs`

- Size: 71 LOC; functions: 0; max fn span: 0 LOC; if/match: 3/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/systems/postfx/registry.rs`

- Size: 222 LOC; functions: 9; max fn span: 39 LOC; if/match: 5/2; clones: 3
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/text_render.rs`

- Size: 350 LOC; functions: 13; max fn span: 73 LOC; if/match: 12/4; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-compositor/src/warmup.rs`

- Size: 61 LOC; functions: 3; max fn span: 31 LOC; if/match: 1/1; clones: 1
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/access.rs`

- Size: 51 LOC; functions: 10; max fn span: 7 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/access_tests.rs`

- Size: 67 LOC; functions: 4; max fn span: 21 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/animations/animation.rs`

- Size: 18 LOC; functions: 1; max fn span: 3 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/animations/builtin/float.rs`

- Size: 24 LOC; functions: 1; max fn span: 14 LOC; if/match: 0/1; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/animations/builtin/mod.rs`

- Size: 6 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/animations/mod.rs`

- Size: 158 LOC; functions: 5; max fn span: 75 LOC; if/match: 2/2; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/animations/params.rs`

- Size: 40 LOC; functions: 2; max fn span: 4 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/asset_cache.rs`

- Size: 36 LOC; functions: 1; max fn span: 19 LOC; if/match: 3/0; clones: 2
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/asset_source.rs`

- Size: 99 LOC; functions: 11; max fn span: 13 LOC; if/match: 1/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/assets.rs`

- Size: 32 LOC; functions: 4; max fn span: 6 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/authoring/catalog.rs`

- Size: 1407 LOC; functions: 15; max fn span: 853 LOC; if/match: 1/0; clones: 0
- Estimated removable LOC from this file/refactor area: 187
  - Oversized file (1407 lines); split orchestration/data/parsing paths and collapse repeated setup branches.
  - Contains very large function(s) (max ~853 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.

### `engine-core/src/authoring/metadata.rs`

- Size: 63 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/authoring/mod.rs`

- Size: 7 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/buffer.rs`

- Size: 1006 LOC; functions: 55; max fn span: 63 LOC; if/match: 38/2; clones: 0
- Estimated removable LOC from this file/refactor area: 40
  - Oversized file (1006 lines); split orchestration/data/parsing paths and collapse repeated setup branches.

### `engine-core/src/color.rs`

- Size: 124 LOC; functions: 8; max fn span: 24 LOC; if/match: 1/1; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/builtin/artifact.rs`

- Size: 187 LOC; functions: 7; max fn span: 101 LOC; if/match: 13/0; clones: 2
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/builtin/blur.rs`

- Size: 198 LOC; functions: 5; max fn span: 78 LOC; if/match: 5/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/builtin/brighten.rs`

- Size: 60 LOC; functions: 2; max fn span: 31 LOC; if/match: 2/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/builtin/clear_to_colour.rs`

- Size: 52 LOC; functions: 2; max fn span: 15 LOC; if/match: 1/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/builtin/crt_burn_in.rs`

- Size: 37 LOC; functions: 2; max fn span: 6 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/builtin/crt_distort.rs`

- Size: 36 LOC; functions: 2; max fn span: 6 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/builtin/crt_on.rs`

- Size: 147 LOC; functions: 2; max fn span: 86 LOC; if/match: 10/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/builtin/crt_reflection.rs`

- Size: 307 LOC; functions: 13; max fn span: 55 LOC; if/match: 10/0; clones: 1
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/builtin/crt_ruby.rs`

- Size: 34 LOC; functions: 2; max fn span: 6 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/builtin/crt_scan_glitch.rs`

- Size: 34 LOC; functions: 2; max fn span: 6 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/builtin/crt_underlay.rs`

- Size: 36 LOC; functions: 2; max fn span: 6 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/builtin/cutout.rs`

- Size: 621 LOC; functions: 23; max fn span: 112 LOC; if/match: 25/2; clones: 2
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/builtin/devour.rs`

- Size: 188 LOC; functions: 9; max fn span: 103 LOC; if/match: 11/1; clones: 1
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/builtin/fade.rs`

- Size: 95 LOC; functions: 4; max fn span: 25 LOC; if/match: 5/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/builtin/fade_to_black.rs`

- Size: 66 LOC; functions: 2; max fn span: 36 LOC; if/match: 3/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/builtin/glitch.rs`

- Size: 279 LOC; functions: 9; max fn span: 168 LOC; if/match: 22/0; clones: 0
- Estimated removable LOC from this file/refactor area: 25
  - Contains very large function(s) (max ~168 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.

### `engine-core/src/effects/builtin/lightning.rs`

- Size: 1801 LOC; functions: 44; max fn span: 256 LOC; if/match: 85/8; clones: 2
- Estimated removable LOC from this file/refactor area: 317
  - Oversized file (1801 lines); split orchestration/data/parsing paths and collapse repeated setup branches.
  - Contains very large function(s) (max ~256 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.
  - High count of long functions (7); repeated branch/error/render logic is likely compressible.
  - Branch-heavy module (85 ifs, 8 matches); candidate for declarative tables, enum methods, or strategy objects.

### `engine-core/src/effects/builtin/mod.rs`

- Size: 241 LOC; functions: 3; max fn span: 19 LOC; if/match: 0/0; clones: 0
- Estimated removable LOC from this file/refactor area: 0
  - Large data/model module but not obviously redundant from text-level scan; review for generated/schema-like content before manual refactor.

### `engine-core/src/effects/builtin/neon_edge_glow.rs`

- Size: 173 LOC; functions: 2; max fn span: 111 LOC; if/match: 11/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/builtin/posterize.rs`

- Size: 135 LOC; functions: 6; max fn span: 41 LOC; if/match: 3/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/builtin/power_off.rs`

- Size: 71 LOC; functions: 2; max fn span: 40 LOC; if/match: 7/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/builtin/scanlines.rs`

- Size: 41 LOC; functions: 2; max fn span: 13 LOC; if/match: 1/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/builtin/shake.rs`

- Size: 87 LOC; functions: 3; max fn span: 40 LOC; if/match: 2/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/builtin/shatter.rs`

- Size: 402 LOC; functions: 11; max fn span: 236 LOC; if/match: 25/1; clones: 1
- Estimated removable LOC from this file/refactor area: 35
  - Contains very large function(s) (max ~236 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.

### `engine-core/src/effects/builtin/shine.rs`

- Size: 127 LOC; functions: 2; max fn span: 65 LOC; if/match: 6/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/builtin/terminal_crt.rs`

- Size: 329 LOC; functions: 11; max fn span: 87 LOC; if/match: 10/0; clones: 1
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/builtin/whiteout.rs`

- Size: 49 LOC; functions: 2; max fn span: 22 LOC; if/match: 2/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/effect.rs`

- Size: 93 LOC; functions: 5; max fn span: 44 LOC; if/match: 0/1; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/metadata.rs`

- Size: 329 LOC; functions: 5; max fn span: 153 LOC; if/match: 2/9; clones: 0
- Estimated removable LOC from this file/refactor area: 22
  - Contains very large function(s) (max ~153 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.

### `engine-core/src/effects/mod.rs`

- Size: 98 LOC; functions: 10; max fn span: 13 LOC; if/match: 2/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/utils/color.rs`

- Size: 65 LOC; functions: 4; max fn span: 23 LOC; if/match: 0/1; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/utils/math.rs`

- Size: 47 LOC; functions: 4; max fn span: 12 LOC; if/match: 1/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/utils/mod.rs`

- Size: 9 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/effects/utils/noise.rs`

- Size: 26 LOC; functions: 3; max fn span: 13 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/game_object.rs`

- Size: 27 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- Duplication signal: near-duplicate of `engine-game/src/game_object.rs` (similarity 0.981, shared body roughly 26 LOC)
- Estimated removable LOC from this file/refactor area: 22
  - Near-duplicate module with `engine-game/src/game_object.rs` (similarity 0.981); move shared implementation to one crate/module and re-export thinly.

### `engine-core/src/game_state.rs`

- Size: 311 LOC; functions: 22; max fn span: 54 LOC; if/match: 23/9; clones: 2
- Duplication signal: near-duplicate of `engine-game/src/game_state.rs` (similarity 1.000, shared body roughly 311 LOC)
- Estimated removable LOC from this file/refactor area: 264
  - Near-duplicate module with `engine-game/src/game_state.rs` (similarity 1.000); move shared implementation to one crate/module and re-export thinly.

### `engine-core/src/level_state.rs`

- Size: 314 LOC; functions: 22; max fn span: 48 LOC; if/match: 16/3; clones: 4
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/lib.rs`

- Size: 43 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/logging.rs`

- Size: 334 LOC; functions: 21; max fn span: 48 LOC; if/match: 20/2; clones: 3
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/markup.rs`

- Size: 157 LOC; functions: 9; max fn span: 53 LOC; if/match: 9/0; clones: 2
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/scene/color.rs`

- Size: 136 LOC; functions: 4; max fn span: 51 LOC; if/match: 6/2; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/scene/easing.rs`

- Size: 33 LOC; functions: 1; max fn span: 16 LOC; if/match: 1/1; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/scene/metadata.rs`

- Size: 1296 LOC; functions: 2; max fn span: 15 LOC; if/match: 1/0; clones: 0
- Estimated removable LOC from this file/refactor area: 40
  - Oversized file (1296 lines); split orchestration/data/parsing paths and collapse repeated setup branches.

### `engine-core/src/scene/mod.rs`

- Size: 37 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/scene/model.rs`

- Size: 926 LOC; functions: 26; max fn span: 156 LOC; if/match: 0/2; clones: 2
- Estimated removable LOC from this file/refactor area: 63
  - Oversized file (926 lines); split orchestration/data/parsing paths and collapse repeated setup branches.
  - Contains very large function(s) (max ~156 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.

### `engine-core/src/scene/sprite.rs`

- Size: 1090 LOC; functions: 29; max fn span: 527 LOC; if/match: 0/20; clones: 0
- Estimated removable LOC from this file/refactor area: 131
  - Oversized file (1090 lines); split orchestration/data/parsing paths and collapse repeated setup branches.
  - Contains very large function(s) (max ~527 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.
  - Branch-heavy module (0 ifs, 20 matches); candidate for declarative tables, enum methods, or strategy objects.

### `engine-core/src/scene/template.rs`

- Size: 132 LOC; functions: 6; max fn span: 41 LOC; if/match: 14/1; clones: 7
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/scene/ui_theme.rs`

- Size: 179 LOC; functions: 8; max fn span: 73 LOC; if/match: 1/1; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/scene_runtime_types.rs`

- Size: 158 LOC; functions: 13; max fn span: 41 LOC; if/match: 0/0; clones: 1
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/strategy/diff.rs`

- Size: 51 LOC; functions: 4; max fn span: 14 LOC; if/match: 1/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/strategy/effect_factory.rs`

- Size: 16 LOC; functions: 1; max fn span: 3 LOC; if/match: 1/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/strategy/mod.rs`

- Size: 8 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-core/src/world.rs`

- Size: 103 LOC; functions: 9; max fn span: 29 LOC; if/match: 1/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-debug/src/access.rs`

- Size: 41 LOC; functions: 14; max fn span: 5 LOC; if/match: 0/0; clones: 0
- Estimated removable LOC from this file/refactor area: 14
  - Wrapper-heavy API surface (14 short functions across 41 lines); macro/generic accessors can remove boilerplate.

### `engine-debug/src/lib.rs`

- Size: 169 LOC; functions: 8; max fn span: 35 LOC; if/match: 2/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-debug/src/log.rs`

- Size: 188 LOC; functions: 14; max fn span: 31 LOC; if/match: 6/1; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-debug/src/profiling.rs`

- Size: 375 LOC; functions: 25; max fn span: 47 LOC; if/match: 10/1; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-error/src/lib.rs`

- Size: 52 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-events/src/access.rs`

- Size: 20 LOC; functions: 4; max fn span: 5 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-events/src/input_backend.rs`

- Size: 10 LOC; functions: 1; max fn span: 3 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-events/src/key.rs`

- Size: 94 LOC; functions: 5; max fn span: 35 LOC; if/match: 0/1; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-events/src/lib.rs`

- Size: 52 LOC; functions: 4; max fn span: 5 LOC; if/match: 1/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-frame/src/lib.rs`

- Size: 89 LOC; functions: 6; max fn span: 15 LOC; if/match: 3/1; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-game/src/collision.rs`

- Size: 134 LOC; functions: 4; max fn span: 42 LOC; if/match: 4/2; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-game/src/components.rs`

- Size: 273 LOC; functions: 15; max fn span: 44 LOC; if/match: 9/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-game/src/game_object.rs`

- Size: 26 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- Duplication signal: near-duplicate of `engine-core/src/game_object.rs` (similarity 0.981, shared body roughly 26 LOC)
- Estimated removable LOC from this file/refactor area: 22
  - Near-duplicate module with `engine-core/src/game_object.rs` (similarity 0.981); move shared implementation to one crate/module and re-export thinly.

### `engine-game/src/game_state.rs`

- Size: 311 LOC; functions: 22; max fn span: 54 LOC; if/match: 23/9; clones: 2
- Duplication signal: near-duplicate of `engine-core/src/game_state.rs` (similarity 1.000, shared body roughly 311 LOC)
- Estimated removable LOC from this file/refactor area: 264
  - Near-duplicate module with `engine-core/src/game_state.rs` (similarity 1.000); move shared implementation to one crate/module and re-export thinly.

### `engine-game/src/gameplay.rs`

- Size: 1079 LOC; functions: 83; max fn span: 48 LOC; if/match: 52/5; clones: 6
- Estimated removable LOC from this file/refactor area: 40
  - Oversized file (1079 lines); split orchestration/data/parsing paths and collapse repeated setup branches.

### `engine-game/src/lib.rs`

- Size: 28 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-game/src/strategy.rs`

- Size: 69 LOC; functions: 3; max fn span: 43 LOC; if/match: 4/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-io/src/lib.rs`

- Size: 420 LOC; functions: 32; max fn span: 37 LOC; if/match: 13/4; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-mod/src/lib.rs`

- Size: 169 LOC; functions: 8; max fn span: 33 LOC; if/match: 6/0; clones: 2
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-mod/src/output_backend.rs`

- Size: 45 LOC; functions: 2; max fn span: 19 LOC; if/match: 0/2; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-mod/src/startup/check.rs`

- Size: 13 LOC; functions: 2; max fn span: 3 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-mod/src/startup/checks/action_map.rs`

- Size: 159 LOC; functions: 7; max fn span: 75 LOC; if/match: 12/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-mod/src/startup/checks/audio_sequencer.rs`

- Size: 327 LOC; functions: 12; max fn span: 118 LOC; if/match: 24/2; clones: 1
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-mod/src/startup/checks/catalogs.rs`

- Size: 64 LOC; functions: 2; max fn span: 40 LOC; if/match: 3/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-mod/src/startup/checks/effect_registry.rs`

- Size: 414 LOC; functions: 12; max fn span: 70 LOC; if/match: 11/1; clones: 2
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-mod/src/startup/checks/font_glyph_coverage.rs`

- Size: 117 LOC; functions: 2; max fn span: 96 LOC; if/match: 4/0; clones: 3
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-mod/src/startup/checks/font_manifest.rs`

- Size: 89 LOC; functions: 2; max fn span: 69 LOC; if/match: 4/0; clones: 1
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-mod/src/startup/checks/image_assets.rs`

- Size: 68 LOC; functions: 2; max fn span: 50 LOC; if/match: 3/0; clones: 2
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-mod/src/startup/checks/level_config.rs`

- Size: 342 LOC; functions: 16; max fn span: 66 LOC; if/match: 14/2; clones: 3
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-mod/src/startup/checks/mod.rs`

- Size: 26 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-mod/src/startup/checks/rhai_scripts.rs`

- Size: 398 LOC; functions: 9; max fn span: 62 LOC; if/match: 6/2; clones: 4
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-mod/src/startup/checks/scene_graph.rs`

- Size: 271 LOC; functions: 9; max fn span: 94 LOC; if/match: 16/0; clones: 11
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-mod/src/startup/checks/terminal_requirements.rs`

- Size: 220 LOC; functions: 8; max fn span: 65 LOC; if/match: 11/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-mod/src/startup/context.rs`

- Size: 165 LOC; functions: 15; max fn span: 21 LOC; if/match: 1/0; clones: 0
- Estimated removable LOC from this file/refactor area: 57
  - Wrapper-heavy API surface (15 short functions across 165 lines); macro/generic accessors can remove boilerplate.

### `engine-mod/src/startup/mod.rs`

- Size: 13 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-mod/src/startup/report.rs`

- Size: 48 LOC; functions: 3; max fn span: 9 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-mod/src/startup/runner.rs`

- Size: 52 LOC; functions: 3; max fn span: 17 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-mod/src/terminal_caps.rs`

- Size: 228 LOC; functions: 15; max fn span: 40 LOC; if/match: 10/1; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-persistence/src/lib.rs`

- Size: 292 LOC; functions: 22; max fn span: 42 LOC; if/match: 17/8; clones: 2
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-physics/src/lib.rs`

- Size: 109 LOC; functions: 7; max fn span: 23 LOC; if/match: 1/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-pipeline/src/lib.rs`

- Size: 113 LOC; functions: 1; max fn span: 16 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-pipeline/src/strategies/display.rs`

- Size: 23 LOC; functions: 2; max fn span: 3 LOC; if/match: 1/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-pipeline/src/strategies/halfblock.rs`

- Size: 60 LOC; functions: 4; max fn span: 26 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-pipeline/src/strategies/layer.rs`

- Size: 27 LOC; functions: 3; max fn span: 11 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-pipeline/src/strategies/mod.rs`

- Size: 83 LOC; functions: 2; max fn span: 34 LOC; if/match: 5/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-pipeline/src/strategies/present.rs`

- Size: 29 LOC; functions: 3; max fn span: 13 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-pipeline/src/strategies/skip.rs`

- Size: 107 LOC; functions: 10; max fn span: 21 LOC; if/match: 3/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-render-policy/src/lib.rs`

- Size: 306 LOC; functions: 19; max fn span: 55 LOC; if/match: 7/5; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-render-sdl2/src/bitmap_font.rs`

- Size: 537 LOC; functions: 6; max fn span: 494 LOC; if/match: 1/0; clones: 0
- Estimated removable LOC from this file/refactor area: 74
  - Contains very large function(s) (max ~494 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.

### `engine-render-sdl2/src/color_convert.rs`

- Size: 7 LOC; functions: 1; max fn span: 6 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-render-sdl2/src/input.rs`

- Size: 56 LOC; functions: 3; max fn span: 24 LOC; if/match: 0/1; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-render-sdl2/src/lib.rs`

- Size: 6 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-render-sdl2/src/renderer.rs`

- Size: 221 LOC; functions: 14; max fn span: 38 LOC; if/match: 6/3; clones: 2
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-render-sdl2/src/runtime.rs`

- Size: 1217 LOC; functions: 36; max fn span: 267 LOC; if/match: 57/5; clones: 0
- Estimated removable LOC from this file/refactor area: 197
  - Oversized file (1217 lines); split orchestration/data/parsing paths and collapse repeated setup branches.
  - Contains very large function(s) (max ~267 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.
  - High count of long functions (5); repeated branch/error/render logic is likely compressible.

### `engine-render-terminal/src/color_convert.rs`

- Size: 151 LOC; functions: 7; max fn span: 33 LOC; if/match: 1/3; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-render-terminal/src/input.rs`

- Size: 109 LOC; functions: 5; max fn span: 41 LOC; if/match: 5/3; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-render-terminal/src/lib.rs`

- Size: 18 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-render-terminal/src/provider.rs`

- Size: 33 LOC; functions: 16; max fn span: 5 LOC; if/match: 0/0; clones: 0
- Estimated removable LOC from this file/refactor area: 11
  - Wrapper-heavy API surface (16 short functions across 33 lines); macro/generic accessors can remove boilerplate.

### `engine-render-terminal/src/rasterizer/generic.rs`

- Size: 949 LOC; functions: 29; max fn span: 278 LOC; if/match: 28/15; clones: 0
- Duplication signal: near-duplicate of `engine-render/src/generic.rs` (similarity 0.997, shared body roughly 949 LOC)
- Estimated removable LOC from this file/refactor area: 887
  - Near-duplicate module with `engine-render/src/generic.rs` (similarity 0.997); move shared implementation to one crate/module and re-export thinly.
  - Oversized file (949 lines); split orchestration/data/parsing paths and collapse repeated setup branches.
  - Contains very large function(s) (max ~278 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.

### `engine-render-terminal/src/rasterizer/mod.rs`

- Size: 2 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-render-terminal/src/renderer.rs`

- Size: 878 LOC; functions: 26; max fn span: 244 LOC; if/match: 44/7; clones: 1
- Estimated removable LOC from this file/refactor area: 96
  - Oversized file (878 lines); split orchestration/data/parsing paths and collapse repeated setup branches.
  - Contains very large function(s) (max ~244 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.

### `engine-render-terminal/src/strategy/display.rs`

- Size: 65 LOC; functions: 6; max fn span: 18 LOC; if/match: 4/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-render-terminal/src/strategy/flush.rs`

- Size: 43 LOC; functions: 2; max fn span: 26 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-render-terminal/src/strategy/flush_trait.rs`

- Size: 13 LOC; functions: 1; max fn span: 3 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-render-terminal/src/strategy/mod.rs`

- Size: 8 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-render/src/font_loader.rs`

- Size: 352 LOC; functions: 15; max fn span: 66 LOC; if/match: 17/3; clones: 1
- Duplication signal: near-duplicate of `engine/src/rasterizer/font_loader.rs` (similarity 0.997, shared body roughly 352 LOC)
- Estimated removable LOC from this file/refactor area: 299
  - Near-duplicate module with `engine/src/rasterizer/font_loader.rs` (similarity 0.997); move shared implementation to one crate/module and re-export thinly.

### `engine-render/src/generic.rs`

- Size: 955 LOC; functions: 29; max fn span: 278 LOC; if/match: 28/15; clones: 0
- Duplication signal: near-duplicate of `engine-render-terminal/src/rasterizer/generic.rs` (similarity 0.997, shared body roughly 949 LOC)
- Estimated removable LOC from this file/refactor area: 887
  - Near-duplicate module with `engine-render-terminal/src/rasterizer/generic.rs` (similarity 0.997); move shared implementation to one crate/module and re-export thinly.
  - Oversized file (955 lines); split orchestration/data/parsing paths and collapse repeated setup branches.
  - Contains very large function(s) (max ~278 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.

### `engine-render/src/image_loader.rs`

- Size: 338 LOC; functions: 20; max fn span: 43 LOC; if/match: 6/4; clones: 1
- Duplication signal: near-duplicate of `engine/src/image_loader.rs` (similarity 0.982, shared body roughly 338 LOC)
- Estimated removable LOC from this file/refactor area: 287
  - Near-duplicate module with `engine/src/image_loader.rs` (similarity 0.982); move shared implementation to one crate/module and re-export thinly.

### `engine-render/src/lib.rs`

- Size: 162 LOC; functions: 15; max fn span: 12 LOC; if/match: 1/0; clones: 0
- Estimated removable LOC from this file/refactor area: 56
  - Wrapper-heavy API surface (15 short functions across 162 lines); macro/generic accessors can remove boilerplate.

### `engine-render/src/overlay.rs`

- Size: 56 LOC; functions: 3; max fn span: 22 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-render/src/rasterizer.rs`

- Size: 254 LOC; functions: 10; max fn span: 86 LOC; if/match: 12/4; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-render/src/simd_text.rs`

- Size: 351 LOC; functions: 10; max fn span: 139 LOC; if/match: 7/1; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-render/src/types.rs`

- Size: 53 LOC; functions: 1; max fn span: 16 LOC; if/match: 3/0; clones: 0
- Duplication signal: near-duplicate of `engine/src/rasterizer/types.rs` (similarity 1.000, shared body roughly 53 LOC)
- Estimated removable LOC from this file/refactor area: 45
  - Near-duplicate module with `engine/src/rasterizer/types.rs` (similarity 1.000); move shared implementation to one crate/module and re-export thinly.

### `engine-render/src/vector_overlay.rs`

- Size: 33 LOC; functions: 1; max fn span: 5 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-runtime/src/access.rs`

- Size: 16 LOC; functions: 2; max fn span: 5 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-runtime/src/lib.rs`

- Size: 534 LOC; functions: 32; max fn span: 84 LOC; if/match: 14/6; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-scene-runtime/src/access.rs`

- Size: 34 LOC; functions: 4; max fn span: 6 LOC; if/match: 2/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-scene-runtime/src/behavior_runner.rs`

- Size: 1052 LOC; functions: 22; max fn span: 350 LOC; if/match: 66/11; clones: 39
- Estimated removable LOC from this file/refactor area: 152
  - Oversized file (1052 lines); split orchestration/data/parsing paths and collapse repeated setup branches.
  - Contains very large function(s) (max ~350 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.
  - Clone-heavy code (39 clones); some allocations/copies can likely be removed with borrowing/Cow/Arc reuse.

### `engine-scene-runtime/src/camera_3d.rs`

- Size: 176 LOC; functions: 11; max fn span: 32 LOC; if/match: 13/2; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-scene-runtime/src/construction.rs`

- Size: 357 LOC; functions: 12; max fn span: 136 LOC; if/match: 12/3; clones: 21
- Estimated removable LOC from this file/refactor area: 31
  - Clone-heavy code (21 clones); some allocations/copies can likely be removed with borrowing/Cow/Arc reuse.

### `engine-scene-runtime/src/lib.rs`

- Size: 778 LOC; functions: 27; max fn span: 78 LOC; if/match: 6/7; clones: 5
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-scene-runtime/src/lifecycle_controls.rs`

- Size: 165 LOC; functions: 4; max fn span: 70 LOC; if/match: 20/1; clones: 4
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-scene-runtime/src/materialization.rs`

- Size: 796 LOC; functions: 19; max fn span: 130 LOC; if/match: 62/16; clones: 11
- Estimated removable LOC from this file/refactor area: 48
  - High count of long functions (4); repeated branch/error/render logic is likely compressible.

### `engine-scene-runtime/src/object_graph.rs`

- Size: 172 LOC; functions: 16; max fn span: 28 LOC; if/match: 6/0; clones: 14
- Estimated removable LOC from this file/refactor area: 60
  - Wrapper-heavy API surface (16 short functions across 172 lines); macro/generic accessors can remove boilerplate.

### `engine-scene-runtime/src/terminal_shell.rs`

- Size: 868 LOC; functions: 30; max fn span: 119 LOC; if/match: 72/6; clones: 20
- Estimated removable LOC from this file/refactor area: 90
  - Oversized file (868 lines); split orchestration/data/parsing paths and collapse repeated setup branches.
  - Clone-heavy code (20 clones); some allocations/copies can likely be removed with borrowing/Cow/Arc reuse.

### `engine-scene-runtime/src/ui_focus.rs`

- Size: 280 LOC; functions: 18; max fn span: 100 LOC; if/match: 20/3; clones: 7
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-terminal/src/lib.rs`

- Size: 228 LOC; functions: 15; max fn span: 40 LOC; if/match: 10/1; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine-vector/src/lib.rs`

- Size: 358 LOC; functions: 17; max fn span: 55 LOC; if/match: 27/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/asset_cache.rs`

- Size: 2 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/asset_source.rs`

- Size: 7 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/assets.rs`

- Size: 2 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/audio_sequencer.rs`

- Size: 209 LOC; functions: 10; max fn span: 64 LOC; if/match: 8/5; clones: 1
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/behavior.rs`

- Size: 5 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/bench.rs`

- Size: 545 LOC; functions: 16; max fn span: 97 LOC; if/match: 11/0; clones: 0
- Estimated removable LOC from this file/refactor area: 48
  - High count of long functions (4); repeated branch/error/render logic is likely compressible.

### `engine/src/debug_features.rs`

- Size: 2 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/debug_log.rs`

- Size: 2 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/error.rs`

- Size: 2 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/events.rs`

- Size: 6 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/frame_capture.rs`

- Size: 2 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/frame_compare.rs`

- Size: 2 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/frame_ticket.rs`

- Size: 3 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/game_loop.rs`

- Size: 289 LOC; functions: 4; max fn span: 248 LOC; if/match: 28/1; clones: 1
- Estimated removable LOC from this file/refactor area: 37
  - Contains very large function(s) (max ~248 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.

### `engine/src/game_object.rs`

- Size: 3 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/game_state.rs`

- Size: 3 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/gpu/context.rs`

- Size: 113 LOC; functions: 6; max fn span: 41 LOC; if/match: 2/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/gpu/mesh.rs`

- Size: 106 LOC; functions: 3; max fn span: 57 LOC; if/match: 1/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/gpu/mod.rs`

- Size: 19 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/gpu/render.rs`

- Size: 134 LOC; functions: 2; max fn span: 98 LOC; if/match: 2/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/image_loader.rs`

- Size: 338 LOC; functions: 20; max fn span: 43 LOC; if/match: 6/4; clones: 1
- Duplication signal: near-duplicate of `engine-render/src/image_loader.rs` (similarity 0.982, shared body roughly 338 LOC)
- Estimated removable LOC from this file/refactor area: 287
  - Near-duplicate module with `engine-render/src/image_loader.rs` (similarity 0.982); move shared implementation to one crate/module and re-export thinly.

### `engine/src/level_state.rs`

- Size: 161 LOC; functions: 8; max fn span: 56 LOC; if/match: 9/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/lib.rs`

- Size: 778 LOC; functions: 21; max fn span: 359 LOC; if/match: 15/7; clones: 3
- Estimated removable LOC from this file/refactor area: 53
  - Contains very large function(s) (max ~359 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.

### `engine/src/mod_behaviors.rs`

- Size: 2 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/mod_loader.rs`

- Size: 2 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/obj_frame_cache.rs`

- Size: 63 LOC; functions: 5; max fn span: 15 LOC; if/match: 0/0; clones: 0
- Duplication signal: near-duplicate of `engine-3d/src/obj_frame_cache.rs` (similarity 1.000, shared body roughly 63 LOC)
- Estimated removable LOC from this file/refactor area: 53
  - Near-duplicate module with `engine-3d/src/obj_frame_cache.rs` (similarity 1.000); move shared implementation to one crate/module and re-export thinly.

### `engine/src/obj_prerender.rs`

- Size: 3 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/pipeline_flags.rs`

- Size: 2 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/pipelines/mod.rs`

- Size: 4 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/pipelines/startup/mod.rs`

- Size: 5 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/prepared_frame.rs`

- Size: 35 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/rasterizer/font_loader.rs`

- Size: 352 LOC; functions: 15; max fn span: 66 LOC; if/match: 17/3; clones: 1
- Duplication signal: near-duplicate of `engine-render/src/font_loader.rs` (similarity 0.997, shared body roughly 352 LOC)
- Estimated removable LOC from this file/refactor area: 299
  - Near-duplicate module with `engine-render/src/font_loader.rs` (similarity 0.997); move shared implementation to one crate/module and re-export thinly.

### `engine/src/rasterizer/mod.rs`

- Size: 254 LOC; functions: 10; max fn span: 86 LOC; if/match: 12/4; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/rasterizer/types.rs`

- Size: 53 LOC; functions: 1; max fn span: 16 LOC; if/match: 3/0; clones: 0
- Duplication signal: near-duplicate of `engine-render/src/types.rs` (similarity 1.000, shared body roughly 53 LOC)
- Estimated removable LOC from this file/refactor area: 45
  - Near-duplicate module with `engine-render/src/types.rs` (similarity 1.000); move shared implementation to one crate/module and re-export thinly.

### `engine/src/render_policy.rs`

- Size: 6 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/runtime_settings.rs`

- Size: 102 LOC; functions: 4; max fn span: 27 LOC; if/match: 2/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/scene3d_atlas.rs`

- Size: 3 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/scene3d_format.rs`

- Size: 359 LOC; functions: 12; max fn span: 67 LOC; if/match: 4/2; clones: 0
- Duplication signal: near-duplicate of `engine-3d/src/scene3d_format.rs` (similarity 0.997, shared body roughly 359 LOC)
- Estimated removable LOC from this file/refactor area: 305
  - Near-duplicate module with `engine-3d/src/scene3d_format.rs` (similarity 0.997); move shared implementation to one crate/module and re-export thinly.

### `engine/src/scene3d_resolve.rs`

- Size: 262 LOC; functions: 13; max fn span: 100 LOC; if/match: 7/8; clones: 3
- Duplication signal: near-duplicate of `engine-3d/src/scene3d_resolve.rs` (similarity 1.000, shared body roughly 262 LOC)
- Estimated removable LOC from this file/refactor area: 222
  - Near-duplicate module with `engine-3d/src/scene3d_resolve.rs` (similarity 1.000); move shared implementation to one crate/module and re-export thinly.

### `engine/src/scene_loader.rs`

- Size: 203 LOC; functions: 11; max fn span: 33 LOC; if/match: 6/0; clones: 1
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/scene_pipeline.rs`

- Size: 97 LOC; functions: 5; max fn span: 17 LOC; if/match: 1/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/scene_runtime.rs`

- Size: 5 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/services.rs`

- Size: 416 LOC; functions: 88; max fn span: 12 LOC; if/match: 2/0; clones: 2
- Estimated removable LOC from this file/refactor area: 120
  - Wrapper-heavy API surface (88 short functions across 416 lines); macro/generic accessors can remove boilerplate.

### `engine/src/splash.rs`

- Size: 1822 LOC; functions: 61; max fn span: 129 LOC; if/match: 63/13; clones: 3
- Estimated removable LOC from this file/refactor area: 265
  - Oversized file (1822 lines); split orchestration/data/parsing paths and collapse repeated setup branches.
  - High count of long functions (10); repeated branch/error/render logic is likely compressible.

### `engine/src/strategy/behavior_factory.rs`

- Size: 5 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/strategy/mod.rs`

- Size: 28 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/strategy/scene_compositor.rs`

- Size: 6 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/systems/audio_sequencer.rs`

- Size: 35 LOC; functions: 1; max fn span: 27 LOC; if/match: 1/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/systems/bake.rs`

- Size: 2 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/systems/behavior.rs`

- Size: 762 LOC; functions: 13; max fn span: 191 LOC; if/match: 11/3; clones: 11
- Estimated removable LOC from this file/refactor area: 28
  - Contains very large function(s) (max ~191 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.

### `engine/src/systems/collision.rs`

- Size: 31 LOC; functions: 1; max fn span: 28 LOC; if/match: 2/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/systems/compositor/mod.rs`

- Size: 411 LOC; functions: 6; max fn span: 231 LOC; if/match: 9/2; clones: 3
- Estimated removable LOC from this file/refactor area: 34
  - Contains very large function(s) (max ~231 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.

### `engine/src/systems/engine_io.rs`

- Size: 321 LOC; functions: 3; max fn span: 249 LOC; if/match: 30/4; clones: 5
- Estimated removable LOC from this file/refactor area: 37
  - Contains very large function(s) (max ~249 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.

### `engine/src/systems/gameplay.rs`

- Size: 49 LOC; functions: 1; max fn span: 46 LOC; if/match: 8/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/systems/gameplay_events.rs`

- Size: 28 LOC; functions: 2; max fn span: 15 LOC; if/match: 3/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/systems/hot_reload.rs`

- Size: 311 LOC; functions: 11; max fn span: 113 LOC; if/match: 14/4; clones: 2
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/systems/mod.rs`

- Size: 24 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/systems/prerender.rs`

- Size: 41 LOC; functions: 2; max fn span: 18 LOC; if/match: 1/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/systems/renderer.rs`

- Size: 3 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/systems/renderer_tests.rs`

- Size: 2 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/systems/scene3d_prerender.rs`

- Size: 30 LOC; functions: 2; max fn span: 12 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/systems/scene_lifecycle.rs`

- Size: 1513 LOC; functions: 53; max fn span: 152 LOC; if/match: 59/8; clones: 4
- Estimated removable LOC from this file/refactor area: 150
  - Oversized file (1513 lines); split orchestration/data/parsing paths and collapse repeated setup branches.
  - Contains very large function(s) (max ~152 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.
  - High count of long functions (4); repeated branch/error/render logic is likely compressible.

### `engine/src/systems/ship_controller.rs`

- Size: 76 LOC; functions: 1; max fn span: 59 LOC; if/match: 5/2; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/systems/visual_binding.rs`

- Size: 40 LOC; functions: 2; max fn span: 21 LOC; if/match: 3/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/systems/visual_sync.rs`

- Size: 54 LOC; functions: 1; max fn span: 42 LOC; if/match: 1/0; clones: 2
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/systems/warmup.rs`

- Size: 27 LOC; functions: 2; max fn span: 9 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/terminal_caps.rs`

- Size: 3 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/src/world.rs`

- Size: 39 LOC; functions: 4; max fn span: 7 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `engine/tests/frame_regression.rs`

- Size: 113 LOC; functions: 1; max fn span: 98 LOC; if/match: 2/1; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `tools/devtool/src/cli.rs`

- Size: 195 LOC; functions: 0; max fn span: 0 LOC; if/match: 0/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `tools/devtool/src/edit.rs`

- Size: 127 LOC; functions: 3; max fn span: 93 LOC; if/match: 10/0; clones: 2
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `tools/devtool/src/fs_utils.rs`

- Size: 78 LOC; functions: 5; max fn span: 24 LOC; if/match: 9/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `tools/devtool/src/main.rs`

- Size: 369 LOC; functions: 11; max fn span: 108 LOC; if/match: 0/4; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `tools/devtool/src/scaffold.rs`

- Size: 443 LOC; functions: 17; max fn span: 53 LOC; if/match: 23/1; clones: 2
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `tools/devtool/src/schema.rs`

- Size: 39 LOC; functions: 2; max fn span: 20 LOC; if/match: 4/0; clones: 0
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `tools/schema-gen/src/main.rs`

- Size: 570 LOC; functions: 19; max fn span: 133 LOC; if/match: 15/3; clones: 1
- No clear high-confidence simplification issue from text-level scan. Keep unless touched by adjacent refactor.

### `tools/sound-server/src/main.rs`

- Size: 311 LOC; functions: 8; max fn span: 156 LOC; if/match: 28/8; clones: 0
- Estimated removable LOC from this file/refactor area: 23
  - Contains very large function(s) (max ~156 lines); extract table-driven handlers/helpers to reduce control-flow sprawl.
  - 
  - 
  - If you want to keep the `engine-*` convention, I’d keep it, but make the names reflect **ownership and layer** much more strictly.

Right now the problem is probably not the prefix itself. It is that some crates sound like “misc engine stuff” instead of a single bounded domain.

## Naming rule I’d use

Use this pattern:

* `engine-<domain>` for real business/runtime domains
* `engine-<domain>-<role>` when a domain has clear sublayers
* avoid vague names like `core`, `common`, `utils`, `services` unless they are truly tiny and stable

So:

* good: `engine-scene-runtime`
* good: `engine-render-api`
* weaker: `engine-core`
* weak: `engine-services`

---

# Recommended naming set

## Foundation

Keep foundation very small.

* `engine-types` — shared neutral data types only
* `engine-diagnostics` — errors, warnings, spans, reporting
* `engine-workspace` — project/workspace scanning, indexing, file ownership

I would try to **eliminate** or drastically shrink:

* `engine-core`
* `engine-services`

Those names attract random code.

---

## Assets

Instead of loaders/resolvers scattered around:

* `engine-assets` — asset model, IDs, descriptors
* `engine-assets-io` — reading/writing/loading raw asset data
* `engine-assets-compile` — authoring-time compilation/transforms
* `engine-assets-resolve` — ref/path/import resolution

If that feels too many crates at first:

* `engine-assets`
* `engine-assets-compile`

and keep io/resolve as modules inside `engine-assets`.

---

## Scenes

This is one of the cleanest places for explicit layered naming:

* `engine-scene` — canonical scene model only
* `engine-scene-compile` — validation/lowering/normalization
* `engine-scene-runtime` — running scenes, lifecycle, applying state

If you also have authoring/editor-facing scene mutation:

* `engine-scene-authoring`

That is better than burying scene authoring inside a generic authoring crate.

---

## Behavior

For that huge behavior crate, I’d do:

* `engine-behavior` — behavior model / definitions
* `engine-behavior-compile` — parse/build/validate/normalize
* `engine-behavior-runtime` — execution/ticking/dispatch
* `engine-behavior-builtins` — built-in commands/actions/effects

This is much clearer than one giant `engine-behavior`.

---

## Effects

If effects are big enough to stand alone:

* `engine-effects` — shared effect contracts/types
* `engine-effects-builtin` — built-in effects like lightning etc.

If not, keep effects under scene/behavior/runtime. But avoid effect logic being smeared into `core`.

---

## Rendering

This is where naming matters a lot.

I would avoid:

* `engine-render`
* `engine-render-terminal`
* `engine-render-sdl2`

unless `engine-render` is clearly the abstraction.

Better:

* `engine-render-api` — traits, draw primitives, frame contracts
* `engine-render-compositor` — composition/planning/batching
* `engine-render-terminal` — terminal backend
* `engine-render-sdl2` — SDL2 backend

If you want even more consistency:

* `engine-render-backend-terminal`
* `engine-render-backend-sdl2`

That is more verbose, but much clearer.

---

## 3D

Since 3D is duplicated between general engine and 3D crates:

* `engine-scene3d` — 3D scene model/format
* `engine-scene3d-resolve` — 3D reference/asset resolution
* `engine-scene3d-runtime` — optional, if runtime logic is significant

Or simpler:

* `engine-3d` — only if it is truly one cohesive domain

But from what you showed, I think `scene3d`-based naming is more precise.

---

## Game/application state

For state:

* `engine-state` — canonical runtime/app state model
* `engine-game` — game-specific logic using that state

If the project is not actually a “game” in the usual sense, then avoid `game` and use:

* `engine-runtime-state`
* `engine-app`

Because duplicate `game_state.rs` across crates usually means naming/ownership is muddy.

---

## Editor / authoring

If authoring is a first-class part of the repo:

* `engine-editor-domain` — editor-side domain concepts
* `engine-editor-app` — commands/use-cases/session orchestration
* `engine-editor-ui` — UI widgets/rendering
* `engine-authoring` — only if it strictly means compile/edit authoring domain, not “all editor stuff”

If you want fewer crates:

* `engine-editor`
* `engine-editor-ui`
* `engine-authoring`

But I would not let `engine-authoring` absorb UI and general editor state.

---

# What I would rename from vague current-style names

## `engine-core`

Usually this should be split or shrunk into:

* `engine-types`
* `engine-diagnostics`
* `engine-state`

`engine-core` is a magnet crate. I would only keep it if it becomes tiny and truly foundational.

## `engine-services`

I would almost always remove this name.

Replace with something concrete:

* `engine-workspace`
* `engine-assets`
* `engine-app`
* `engine-composition`
* `engine-state`

“services” says nothing about ownership.

## `engine-render`

Rename depending on actual role:

* if abstraction: `engine-render-api`
* if shared implementation: `engine-render-compositor`
* if mixed: split it

## `engine-authoring`

Maybe keep it, but only if it means:

* compile-time/editor-facing content authoring domain

If it contains schema, documents, compile, indexing, file IO, UI state, and runtime glue, then it is too broad.

---

# My preferred full naming proposal

If I were standardising the repo today, I’d use:

* `engine-types`

* `engine-diagnostics`

* `engine-workspace`

* `engine-assets`

* `engine-assets-compile`

* `engine-scene`

* `engine-scene-authoring`

* `engine-scene-compile`

* `engine-scene-runtime`

* `engine-behavior`

* `engine-behavior-compile`

* `engine-behavior-runtime`

* `engine-behavior-builtins`

* `engine-effects`

* `engine-effects-builtin`

* `engine-render-api`

* `engine-render-compositor`

* `engine-render-terminal`

* `engine-render-sdl2`

* `engine-scene3d`

* `engine-scene3d-resolve`

* `engine-state`

* `engine-game`

* `engine-editor-domain`

* `engine-editor-app`

* `engine-editor-ui`

---

# Simpler version if you want fewer renames

If you want to stay practical and not over-split yet, use this reduced set:

* `engine-types`
* `engine-diagnostics`
* `engine-assets`
* `engine-scene`
* `engine-scene-runtime`
* `engine-behavior`
* `engine-behavior-runtime`
* `engine-render-api`
* `engine-render-terminal`
* `engine-render-sdl2`
* `engine-scene3d`
* `engine-state`
* `engine-editor`

This is a good middle ground.

---

# My strongest naming advice

Three rules:

1. **Ban vague crate names**
   Try to phase out `core`, `common`, `services`, `helpers`, `misc`.

2. **Name by ownership, not by activity**
   Example:

   * better: `engine-scene-runtime`
   * worse: `engine-runner`

3. **Use suffixes consistently**
   Pick a system and stick to it:

   * `-compile`
   * `-runtime`
   * `-api`
   * `-ui`
   * `-resolve`
   * `-builtin`

That consistency alone makes big codebases much easier to reason about.

My blunt recommendation:

* keep `engine-*`
* replace `core` and `services`
* make scene / behavior / render naming layered and explicit

A very good next step would be for me to map your **current crate names** to **proposed new names** one by one in a rename table.

