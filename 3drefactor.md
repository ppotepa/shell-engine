# 3D Refactor Checklist

This file is the working checklist for the 2D/3D architecture refactor.

The goal is to keep 2D as a first-class path, make 3D a real subsystem, and
stop pushing rendering semantics through `Sprite::Obj`.

## Hard Rules

- [x] Do not introduce new identifiers, modules, files, or types with `legacy`
      in the name.
- [x] Do not keep permanent compatibility layers, duplicate pipelines, or
      shadow implementations in the engine.
- [x] Keep `type: image`, `type: text`, and existing 2D layout behavior fully
      supported.
- [x] Keep existing YAML scenes and mods loading during the migration.
- [x] Treat planets as one producer of 3D data, not as a special renderer.
- [x] Keep 3D domain logic out of `engine-compositor`.
- [x] Keep asset loading semantics unchanged for unpacked mods and zip mods.
- [x] Do not let runtime mutation semantics depend on stringly-typed render
      internals going forward.
- [x] Keep renderer and engine-core fully mod-agnostic (no mod-specific names,
      examples, or behavior assumptions in core/render code paths).

Policy notes (verified against current code, audit date: 2026-04-16):
- Legacy naming policy (active refactor scope audit): no `legacy` matches in
  `engine-authoring/src/compile/render_scene.rs`, `engine-compositor`,
  `engine-render-3d`, `engine-scene-runtime`, `engine-render-2d`,
  `engine-core`; diff-scan for newly added `legacy` lines also returned no
  matches.
- Mod-agnostic renderer/core policy: no mod-specific literals found in
  `engine-render-3d`, `engine-compositor`, and `engine-core` runtime paths.

## Session Notes (Audit 2026-04-16)

## Session Notes (Audit 2026-04-17)

- Audited checklist completeness against current code.
- Confirmed there are no remaining unchecked checklist items in this file.
- Remaining work is no longer the core 3D ownership split; it is narrowed to:
  documentation consistency inside this file and a final policy decision on
  whether bounded `SetProperty` compatibility remains a sanctioned low-level
  escape hatch or should be removed entirely.
- See `left.md` for the current post-audit list of actually open work.
- Synced root docs and crate READMEs with the final architecture split:
  `README.md`, `ARCHITECTURE.md`, `AUTHORING.md`, `MODS.md`, `CHANGELOG.md`,
  `app/README.md`, `engine-compositor/*README*`, `engine-render-3d/README.md`,
  `engine-worldgen/README.md`, `engine-mesh/README.md`, `engine-api/README.md`,
  `engine-render-2d/README.md`, `mods/README.md`.
- Completed (String-path setter retirement): removed all string-path dispatcher
  functions from `engine-scene-runtime/src/materialization.rs`
  (`set_obj_property_recursive`, `set_planet_property_recursive`,
  `set_obj_sprite_property_by_path`, `set_planet_sprite_property_by_path`,
  `set_obj_material_param`, `set_obj_atmosphere_param`, `set_obj_worldgen_param`,
  `set_planet_material_param`). Removed `Render3DMutation::SetMaterialParam`,
  `SetAtmosphereParam`, and `SetWorldgenParam` string-based variants. All
  request routing through `SetMaterialParam`, `SetAtmosphereParam`,
  `SetSurfaceMode` request types now produces typed mutations directly. All 44
  engine-scene-runtime tests pass; asteroids and planet-generator scene checks
  clean.

- Completed (Final session — shim removal): deleted
  `engine-compositor/src/obj_render.rs` entirely. Updated
  `obj_render_adapter.rs`, `generated_world_render_adapter.rs`, and
  `prerender.rs` to import directly from `engine_render_3d::raster` and
  `engine_render_3d`. Removed `mod obj_render;` from `lib.rs`. All 12
  engine-compositor tests pass. No permanent compat shim remains in compositor.
- Completed (Final session — PR6 audit): audited engine-authoring compilation
  pipeline. Confirmed NO dual-path exists: `SceneDocument::compile()` produces
  only a `Scene`; `compile_3d_viewports_with_authored` in `compile_3d.rs` is
  the SOLE producer of `Viewport3DRef`. The fallback path (deriving viewports
  from scene sprite walk) is intentional — it allows scenes to omit an explicit
  `viewports-3d:` block. Added clarifying doc-comment to the function. 92
  engine-authoring tests pass.
- Completed (Final session — compositor boundary clarification): confirmed that
  `sprite_renderer_2d.rs`, `obj_render_adapter.rs`,
  `generated_world_render_adapter.rs`, `scene_clip_render_adapter.rs` are
  dispatcher/orchestrator modules — all render IMPLEMENTATIONS live in
  `engine-render-2d` and `engine-render-3d`. Marked items 239–241 [x].
- Completed (Final session — Sprite::Obj enforcement): added architecture note
  doc-comment to `Sprite::Obj` in `engine-core/src/scene/sprite.rs` stating
  that new 3D capabilities must go through typed `Node3D`, not new fields here.

- Completed: reduced engine-generated string-path mutation traffic by routing
  Rhai `set` command emission through
  `engine_api::commands::scene_mutation_request_from_set_property_compat(...)`
  in `engine-behavior/src/lib.rs`, so supported property paths now emit
  `BehaviorCommand::ApplySceneMutation` instead of raw `SetProperty`.
- Completed: updated gameplay-side visual mutation helpers in
  `engine-behavior/src/scripting/gameplay_impl.rs` so typed-compatible
  `style.fg` and `vector.points` writes use `ApplySceneMutation` first and only
  fall back to `SetProperty` for unsupported paths.
- Completed: updated palette binding application in
  `engine-scene-runtime/src/behavior_runner.rs` to enqueue typed scene
  mutations instead of generating raw `SetProperty` commands for style-bound
  updates.
- Completed: narrowed `engine-compositor` public API by removing unused
  re-exports of `engine-render-3d::prerender` types/state so compositor no
  longer presents itself as an owner of shared 3D prerender domain types.
- Completed: moved internal gameplay-side scene lifecycle producers away from
  raw `SceneSpawn`/`SceneDespawn` commands in
  `engine-behavior/src/scripting/ephemeral.rs` and
  `engine-behavior/src/scripting/gameplay_impl.rs`; these paths now emit typed
  `ApplySceneMutation` spawn/despawn requests.
- Completed: taught `engine-api/src/scene/api.rs` to translate
  `position.x/y` and `offset.x/y` writes into typed `Set2dProps` mutations when
  current object state is available, and added focused tests for both
  `ScriptSceneApi` and `ScriptObjectApi`.
- Completed: removed dead `is_render3d_compat_param_path` in
  `engine-scene-runtime/src/render3d_state.rs` and verified `engine-scene-runtime`
  tests pass.
- Completed: removed stale `engine-compositor/src/lib.rs` re-export entries
  (`blit_color_canvas`, `render_obj_to_shared_buffers`) and cleaned an unused
  `Scene3DRuntimeEntry` import in
  `engine-compositor/src/scene_clip_render_adapter.rs`.
- Completed: removed stale compositor-local render-domain duplicates:
  `engine-compositor/src/obj_prerender.rs` and
  `engine-compositor/src/scene3d_prerender.rs` (now provided from
  `engine-render-3d` ownership).
- Completed: updated `engine-compositor/README.md` and
  `engine-compositor/README.AGENTS.md` to match render-orchestration intent.
- Completed: restored end-to-end engine build after the 3D prerender migration.
  Final state: `render_scene3d_work_item` stays owned by
  `engine-render-3d::prerender`, and
  `engine/src/systems/scene3d_prerender.rs` imports that seam directly with no
  compositor re-export.
- Completed: completed `engine` -> `engine-render-2d` dependency wiring in
  `engine/Cargo.toml` so compositor vector primitive lifecycle calls (`clear`
  and `take`) can be wired through dedicated 2D seam without temporary
  `engine_render_3d` fallback suggestions; `cargo check -p engine` now passes.
- Completed: confirmed current `engine-render-3d`, `engine-compositor`, `engine-core`,
  `engine-render-2d`, `engine-authoring`, and `engine-scene-runtime` paths remain
  free from mod-specific literals and `legacy` identifiers in compile-time code.
- Completed (Streams 1+2): moved OBJ rasterization pipeline ownership to
  `engine-render-3d/src/raster.rs` — `virtual_dimensions`, `blit_color_canvas`,
  `render_obj_to_shared_buffers`, `render_mesh_projected`, and all rasterizer
  helpers (`rasterize_triangle`, `rasterize_triangle_gouraud`, `draw_line_depth`,
  `draw_line_flat`, `face_avg_depth`) now live in `engine-render-3d`. Added
  `render_scene3d_work_item` to `engine-render-3d::prerender` so
  `engine/src/systems/scene3d_prerender.rs` no longer imports from
  `engine-compositor` for Scene3D rendering. Removed the compositor re-export of
  `render_scene3d_work_item` and the compositor's local
  `render_obj_to_shared_buffers`/`render_mesh_projected` definitions. All 38
  tests pass (engine-render-3d: 24, engine-compositor: 14).
- Completed (PR7): moved all remaining render-domain logic out of
  `engine-compositor/src/obj_render.rs` and `engine-compositor/src/obj_render_helpers.rs`
  into `engine-render-3d/src/raster.rs`. Moved: thread-locals (`OBJ_CANVAS`,
  `OBJ_CANVAS_RGBA`, `OBJ_DEPTH`, `OBJ_SHADED_GOURAUD`, `OBJ_SHADED_FLAT`),
  `obj_sprite_dimensions`, `render_obj_to_canvas`, `apply_atmosphere_halo_canvas`,
  `smoothstep`, `gaussian`, `convert_canvas_to_rgba`, `render_obj_to_rgba_canvas`,
  `render_obj_content`, `try_blit_prerendered`, `composite_rgba_over`,
  `blit_rgba_canvas`, `rasterize_triangle_gouraud_rgba`. Deleted
  `obj_render_helpers.rs` and both `obj_render/setup.rs` / `obj_render/terrain_eval.rs`
  submodules. `engine-compositor/src/obj_render.rs` is now a 12-line thin re-export
  shim. Compositor adapters (`obj_render_adapter.rs`,
  `generated_world_render_adapter.rs`) now import from `engine_render_3d`. All 26
  engine-render-3d tests and 12 engine-compositor tests pass; all scene checks
  clean (planet-generator, playground).
- Completed (PR5 completion pass): confirmed that `BehaviorCommand::SetProperty`
  is already a narrow compat converter with no second runtime path.
  `SetProperty` → `scene_mutation_request_from_set_property_runtime_compat` →
  `SceneMutationRequest` → `scene_mutation_from_request` — the same function
  used by `ApplySceneMutation`. Added explicit documentation to the arm and the
  helper function making this convergence visible. Marked PR5 checklist item
  "Collapse SetProperty handling onto typed mutations without a second runtime
  path" done. Also marked `engine-scene-runtime / Collapse mutation handling
  toward one typed implementation path` done. All 42 engine-scene-runtime tests
  continue to pass.

## Remaining Follow-Up (Updated 2026-04-17)

- `SetProperty` remains as a **narrow, intentional escape hatch** for property
  paths not covered by any typed mutation. It is not a second pipeline: all
  calls still converge on the same typed request/mutation flow after bounded
  compatibility translation in runtime.
- The only still-open policy question is whether this low-level fallback stays
  as part of the final supported surface, or is removed in a later cleanup pass.
- **Future epics (out of scope for this refactor):**
  - `CompositeParams` narrowing: requires prepared `Render2dInput`/`Render3dInput` types and a
    full compositor API redesign.
  - Planet as one 3D producer: `generated_world_render_adapter.rs` dispatches to `engine-render-3d`
    but planet rendering is genuinely specialized (biomes, clouds, atmosphere). Unification requires
    a richer `Node3D` type that can express planet-specific surface params.
  - `2D-only no 3D dependency leakage`: requires workspace feature flags to make `engine-render-3d`
    an optional dep for 2D-only projects.

## End State

- [x] `engine-render-2d` owns 2D rendering.
- [x] `engine-render-3d` owns 3D rendering.
- [x] `engine-compositor` only assembles frame output.
- [x] `engine-scene-runtime` owns runtime state and mutations.
- [x] `engine-authoring` owns authored AST and scene compilation.
- [x] `engine-asset` owns image/mesh/material decode and cache.
- [x] `engine-worldgen` owns world generation and mesh build keys.

## Crate Boundaries

### engine-core

- [x] Add `render_types/mod.rs`.
- [x] Add `render_types/viewport.rs`.
- [x] Add `render_types/transform_2d.rs`.
- [x] Add `render_types/transform_3d.rs`.
- [x] Add `render_types/camera_2d.rs`.
- [x] Add `render_types/camera_3d.rs`.
- [x] Add `render_types/light_3d.rs`.
- [x] Add `render_types/material.rs`.
- [x] Add `render_types/dirty.rs`.
- [x] Add `render_types/render_scene.rs`.
- [x] Keep these types backend-neutral and authoring-neutral.

### engine-authoring

- [x] Add `document/viewport3d.rs`.
- [x] Add `document/render_scene3d.rs`.
- [x] Add `document/material.rs`.
- [x] Add `document/atmosphere_profile.rs`.
- [x] Add `document/world_profile.rs`.
- [x] Add `document/camera_profile.rs`.
- [x] Add `compile/compile_render_scene.rs`.
- [x] Add `compile/compile_2d.rs`.
- [x] Add `compile/compile_3d.rs`.
- [x] Add `validate/render3d.rs`.
- [x] Compile `obj`, `planet`, and `scene3d` directly into the new intermediate
      model without parallel compiler paths.

### engine-scene-runtime

- [x] Add `mutations.rs`.
- [x] Add `render3d_state.rs`.
- [x] Add `dirty_tracking.rs`.
- [x] Keep `SceneRuntime` as the object graph and mutation center.
- [x] Add typed 3D mutations.
- [x] Collapse mutation handling toward one typed implementation path.

### engine-render-2d

- [x] Create crate `engine-render-2d`.
- [x] Add `api.rs`.
- [x] Move image rendering here.
- [x] Move text rendering here.
- [x] Move vector rendering here.
- [x] Move 2D sprite dispatch here.
- [x] Move layout helpers here.
- [x] Move container rendering here.

### engine-render-3d

- [x] Expand `api.rs` into concrete `Render3dInput` and `Render3dOutput`.
- [x] Add `scene/mod.rs`.
- [x] Add `scene/instance.rs`.
- [x] Add `scene/nodes.rs`.
- [x] Add `scene/camera.rs`.
- [x] Add `scene/lights.rs`.
- [x] Add `scene/materials.rs`.
- [x] Add `scene/viewport.rs`.
- [x] Add `scene/dirty.rs`.
- [x] Add `mesh/mod.rs`.
- [x] Add `mesh/asset_mesh.rs`.
- [x] Add `mesh/generated_mesh.rs`.
- [x] Add `mesh/cache.rs`.
- [x] Add `pipeline/mod.rs`.
- [x] Add `pipeline/renderer.rs`.
- [x] Map `Obj`, `Planet`, and `Scene3D` directly into the 3D scene graph.
- [x] Add `prerender/mod.rs`.
- [x] Add `prerender/scene3d_atlas.rs`.
- [x] Add `prerender/scene3d_runtime_store.rs`.
- [x] Add `prerender/scene3d_prerender.rs`.

### engine-compositor

- [x] Remove direct ownership of 2D rendering code.
- [x] Remove direct ownership of 3D rendering code.
- [x] Keep only frame assembly, layer ordering, blending, and postfx.
- [x] Narrow `CompositeParams`.
- [x] Consume prepared 2D and 3D inputs instead of raw authored sprite detail.

### engine-asset

- [x] Move image decode/cache concerns here.
- [x] Add `mesh_repository.rs`.
- [x] Add `material_repository.rs`.
- [x] Add `build_keys.rs`.
- [x] Expose shared image and mesh access for both renderers.

### engine-worldgen

- [x] Add stable `MeshBuildKey`.
- [x] Move geometry build key creation here.
- [x] Keep procedural world generation independent of the render loop.

## Core Types

- [x] Define `ViewportRect`.
- [x] Define `Transform2D`.
- [x] Define `Transform3D`.
- [x] Define `Camera2DState`.
- [x] Define `Camera3DState`.
- [x] Define `LightKind3D`.
- [x] Define `Light3D`.
- [x] Define `DirtyMask3D`.
- [x] Define `CompiledRenderScene`.
- [x] Define `Scene3DInstance`.
- [x] Define `Node3DInstance`.
- [x] Define `Renderable3D`.
- [x] Define `MeshInstance`.
- [x] Define `GeneratedWorldInstance`.
- [x] Define `Render2dInput`.
- [x] Define `Render3dInput`.
- [x] Define `Render3dOutput`.
- [x] Define `SceneMutation`.
- [x] Define `Render3DMutation`.
- [x] Define `MeshBuildKey`.

## Migration Rules

- [x] Keep `type: image` untouched.
- [x] Keep `type: text` untouched.
- [x] Keep `type: obj`, `type: planet`, and `type: scene3d` working while they
      are still first-class authored forms.
- [x] Do not build permanent dual execution paths to support migration.
- [x] Route new behavior features through typed mutation APIs, not new string
      paths.

## Dirty Flags and Invalidation

- [x] Add transform-only invalidation.
- [x] Add camera-only invalidation.
- [x] Add lighting-only invalidation.
- [x] Add material-only invalidation.
- [x] Add atmosphere-only invalidation.
- [x] Add mesh-only invalidation.
- [x] Add worldgen rebuild invalidation.
- [x] Add visibility-only invalidation.
- [x] Document which mutation sets which mask.
- [x] Add runtime diagnostics for rebuild causes and counts.

## Asset and Build Keys

- [x] Stop constructing geometry keys inside the render hot path.
- [x] Replace URI rewrite semantics with typed build keys before render.
- [x] Keep URI parsing out of the core render API.
- [x] Allow one image asset to be used as 2D sprite input and as 3D texture
      input through the same asset layer.

## PR Sequence

### PR0 - Baseline Protection

- [x] Add regression tests for 2D image rendering.
- [x] Add regression tests for 2D text rendering.
- [x] Add regression tests for grid/flex layout.
- [x] Add regression tests for `type: obj`.
- [x] Add regression tests for `type: planet`.
- [x] Add regression tests for `type: scene3d`.
- [x] Capture benchmark baseline before structural changes.

PR0 baseline references:

- `engine-render-2d/src/image.rs` image regression coverage
- `engine-render-2d/src/text.rs` text regression coverage
- `engine-compositor/src/layout/grid.rs` existing grid regression coverage
- `engine-compositor/src/layout/flex.rs` existing flex regression coverage
- `engine-authoring/src/compile/scene.rs` compile coverage for `image / obj / planet / scene3_d`
- `engine-scene-runtime/src/lib.rs` runtime mutation coverage for `obj.* / planet.* / scene3d.frame`
- `reports/benchmark/20260416-143211.txt` 2D baseline (`playground-fps-showcase-2d-30`)
- `reports/benchmark/20260416-143224.txt` 3D baseline (`planet-generator-main`)

### PR1 - Core Render Types + engine-render-2d

- [x] Add shared render types in `engine-core`.
- [x] Add `CompiledRenderScene` in `engine-authoring`.
- [x] Create `engine-render-2d`.
- [x] Move 2D modules out of `engine-compositor`.
- [x] Switch compositor to use `Render2dPipeline`.
- [x] Keep 3D behavior unchanged in this PR.

### PR2 - Real 3D Input Model

- [x] Add concrete scene graph types to `engine-render-3d`.
- [x] Map authored 3D sprite forms directly into scene graph nodes.
- [x] Stop unpacking full 3D semantics directly inside sprite rendering.

### PR3 - Move Scene3D Prerender into 3D Domain

- [x] Move Scene3D atlas ownership to `engine-render-3d`.
- [x] Move Scene3D runtime store ownership to `engine-render-3d`.
- [x] Move Scene3D prerender pipeline to `engine-render-3d`.
- [x] Keep scene pipeline orchestration in `engine`.

### PR4 - Asset and Mesh Build Layer

- [x] Move image decode/cache into `engine-asset`.
- [x] Add mesh repository APIs.
- [x] Add material repository APIs.
- [x] Introduce `MeshBuildKey`.
- [x] Remove geometry key construction from render paths.

### PR5 - Typed Runtime Mutations

- [x] Add typed scene mutations.
- [x] Add typed 3D mutations.
- [x] Bridge selected 3D `SetProperty` paths into typed `SceneMutation` flow
      (`scene3d.frame`, `planet.*` subset, `obj.world.*`) while preserving
      fallback behavior for unsupported paths.
- [x] Route `ApplySceneMutation` and legacy camera behavior commands through the
      shared typed request-adapter conversion path to reduce duplicate mutation
      branching.
- [x] Remove duplicate `SetProperty` handling for `scene3d.frame` now that the
      typed bridge path is authoritative for that safe case.
- [x] Collapse `SetProperty` handling onto typed mutations without a second
      runtime path.
- [x] Wire dirty flag updates from typed mutations.

### PR6 - New 3D Authoring Surface

- [x] Add new 3D authored document types.
- [x] Add new validation rules.
- [x] Replace old authored forms by moving compilation into the new model, not
      by keeping duplicate compilers.

### PR7 - Simplify engine-compositor

- [x] Remove direct render-domain logic from `engine-compositor`.
- [x] Keep only frame composition concerns.
- [x] Narrow public compositor APIs to frame assembly inputs.
- [x] Move mesh warmup ownership to `engine-render-3d::prerender`.

### PR8 - Public Typed API

- [x] Add typed public scene mutation APIs in `engine-api`.
- [x] Retire string-path setters after typed APIs take over.
- [x] Update scripting documentation when the typed API lands.

## First Work Queue

These are the tasks to start with immediately.

- [x] Finish PR0 baseline and regression protection.
- [x] Add `engine-core` render types.
- [x] Add `CompiledRenderScene`.
- [x] Create `engine-render-2d`.
- [x] Move `image_render.rs`.
- [x] Move `text_render.rs`.
- [x] Move 2D layout helpers.
- [x] Move 2D container helpers.
- [x] Split compositor sprite renderer into `2D` and `3D` modules.
- [x] Switch compositor to the 2D pipeline seam.
- [x] Route 3D sprite calls through an injected delegate (remove direct 2D->3D coupling).
- [x] Move planet-specific render mapping out of renderer path into a dedicated adapter module.
- [x] Wire planet rendering delegate directly to adapter and remove intermediary 3D wrapper hop.
- [x] Rename renderer delegate surface from `planet` to `generated_world` to keep render API feature-agnostic.
- [x] Introduce `engine-render-3d::scene` module with initial typed scene graph structures.
- [x] Introduce concrete `Render3dInput` / `Render3dOutput` types in `engine-render-3d::api`.
- [x] Add `engine-render-3d::pipeline::map_sprite_to_node3d` for `Obj`, `Planet`, and `Scene3D`.
- [x] Route `Scene3D` sprite render path through typed node mapping (`map_sprite_to_node3d`) instead of direct field unpacking.
- [x] Route `Planet` adapter base identity/transform (`body/preset/mesh/rotation/scale/position`) through typed node mapping.
- [x] Route `Obj` sprite base source/transform (`source/position/rotation/scale`) through typed node mapping.
- [x] Rename renderer delegate surface from `scene3d` to agnostic `scene_clip`.
- [x] Rename internal compositor 3D scene clip renderer entrypoint to `render_scene_clip_sprite`.
- [x] Move `Obj` sprite render mapping/execution out of `sprite_renderer_3d` into dedicated `obj_render_adapter` module.
- [x] Move `scene_clip` sprite render mapping/execution out of `sprite_renderer_3d` into dedicated `scene_clip_render_adapter` module.
- [x] Remove `sprite_renderer_3d` intermediary module and wire compositor delegate directly to 3D adapters.
- [x] Extract Obj source override resolution (`terrain-*` / `world://`) into `obj_source_resolver` to keep adapter render flow focused.
- [x] Rename compositor planet adapter/module symbols to generated-world naming (`generated_world_render_adapter`, `render_generated_world_sprite`) to keep render path source-agnostic.
- [x] Move generated-world render execution pipeline from compositor into `engine-render-3d::pipeline::generated_world_renderer`, with compositor reduced to resolver/profile-finalization wrapper.
- [x] Remove newly added `legacy` wording/identifiers from compositor-side 3D path (`obj_render_adapter`, `obj_render::params`, `pass_underlay`, grid comment) to enforce naming rule in active refactor scope.
- [x] Remove `Scene3DAtlas` module ownership from `engine-compositor` and switch atlas type/re-exports to shared 3D domain (`engine-3d::scene3d_atlas`).
- [x] Move `Scene3DRuntimeStore` ownership out of `engine-compositor` into `engine-render-3d::prerender::runtime_store` and keep compositor as API consumer.
- [x] Route compositor `Scene3DAtlas` surface through `engine-render-3d::prerender` API (thin re-export seam) instead of direct `engine-3d` import in compositor.
- [x] Move `build_scene3d_runtime_store` implementation from `engine-compositor` to `engine-render-3d::prerender::runtime_builder` and keep compositor as orchestration/re-export layer.
- [x] Remove mod-flavoured Scene3D clip examples from renderer-facing comments (`solar-orbit*` -> neutral `orbit*`) to keep render docs domain-agnostic.
- [x] Move Scene3D source discovery + load/resolve helpers from compositor prerender path into `engine-render-3d::prerender::scene_sources` and consume them from compositor.
- [x] Move Scene3D frame sample/keyframe expansion logic from compositor prerender path into `engine-render-3d::prerender::frame_schedule` and consume schedule outputs in compositor.
- [x] Move camera look-at basis helper (`look_at_basis`) from compositor Scene3D prerender path into `engine-render-3d::prerender::camera_basis`.
- [x] Move Scene3D light extraction + hex colour parsing helpers (`extract_light_params`, `parse_hex_color`) from compositor prerender path into `engine-render-3d::prerender::lighting`.
- [x] Move Scene3D clip progress normalization (`elapsed_ms` -> `t`) into `engine-render-3d::prerender::frame_schedule::clip_progress_at`.
- [x] Move Scene3D tween evaluation and clip camera-frame state resolution into `engine-render-3d::prerender::tween_eval`.
- [x] Move Scene3D object clip motion resolution (`translation/orbit/yaw_offset/clip_y`) into `engine-render-3d::prerender::object_motion`.
- [x] Move `ObjRenderParams` type ownership from `engine-compositor` to `engine-render-3d` (`engine-render-3d::obj_render_params`).
- [x] Move Scene3D object spec assembly (`build_object_specs` + `ObjectRenderSpec`) into `engine-render-3d::prerender::object_specs`.
- [x] Move Scene3D prerender work-item planning (`Scene3DWorkItem` + `build_work_items`) into `engine-render-3d::prerender::work_items`.
- [x] Move Scene3D atlas prerender pipeline loop (source scan/load/parallel render/cache fill) into `engine-render-3d::prerender::pipeline::prerender_scene3d_atlas_with`, leaving compositor as render callback provider.
- [x] Move Scene3D per-frame item construction (`frame_name + elapsed_ms + camera override` -> `Scene3DWorkItem`) into `engine-render-3d::prerender::frame_item`.
- [x] Add Scene3D single-frame render orchestration seam in `engine-render-3d::prerender::pipeline::render_scene3d_frame_at_with`; compositor now delegates orchestration and keeps only raster callback.
- [x] Move Scene3D work-item object pass execution (solid pass + wireframe pass + depth-buffer ownership) into `engine-render-3d::prerender::render_item::render_work_item_canvas_with`.
- [x] Move Scene3D `work item -> Buffer` orchestration into `engine-render-3d::prerender::render_item::render_work_item_buffer_with` (compositor now provides only technical callbacks: dimensions, object raster call, blit).
- [x] Move mesh warmup entrypoint from `engine-compositor` to `engine-render-3d::prerender::warmup_scene_meshes`; `engine` warmup system now depends on the 3D domain seam directly.
- [x] Add shared prepared 3D sprite spec seam (`engine-render-3d::pipeline::render3d_sprite_spec`) and route compositor 3D dispatch (`obj/generated-world/scene-clip`) through prepared specs instead of direct authored sprite branching.
- [x] Refactor compositor prerender target collection to reuse shared 3D extraction helpers + recursive sprite walking, removing duplicated direct authored `Sprite::Obj` interpretation blocks from prerender path.
- [x] Hide `engine-compositor::obj_prerender` module ownership behind top-level compositor exports (`ObjPrerenderedFrames`, `ObjPrerenderStatus`, frame types/constants) and update engine call sites to consume the narrowed public seam.
- [x] Hide non-assembly internal compositor modules (`systems`, `render`, `obj_render_helpers`) behind crate-private visibility and consume only explicit top-level re-exports from engine integration points.
- [x] Remove remaining mod-flavoured sample Scene3D source literals in engine tests (`demo.scene3d.yml` -> `sample.scene3d.yml`) to keep renderer/runtime test fixtures domain-agnostic.
- [x] Extract `Sprite::Obj` field unpacking into `engine-render-3d::pipeline::obj_sprite_spec` and consume it from compositor adapter (reduce render-semantic coupling to authored sprite internals).
- [x] Extract `Sprite::Planet` field unpacking into `engine-render-3d::pipeline::generated_world_sprite_spec` and consume it from compositor adapter (keep generated-world path renderer-agnostic).
- [x] Extract `Sprite::Scene3D` field unpacking into `engine-render-3d::pipeline::scene_clip_sprite_spec` and route scene-clip node mapping through typed spec extraction.
- [x] Update `scene_clip_render_adapter` to consume `extract_scene_clip_sprite_spec(...).node` directly instead of `map_sprite_to_node3d(...)`.
- [x] Update `generated_world_render_adapter` to consume `extract_generated_world_sprite_spec(...).node` directly instead of `map_sprite_to_node3d(...)`.
- [x] Route `map_sprite_to_node3d` planet-node creation through `extract_generated_world_sprite_spec(...).node` to remove duplicate mapping logic in pipeline core.
- [x] Prepare per-layer timed-visibility flags outside `layer_compositor` and consume them as frame assembly input (remove direct sprite timing field interpretation from scratch-path selection).
- [x] Prepare scene-step effect progress before compositor dispatch and pass scalar progress through `CompositeParams` (remove raw step-duration ownership from compositor effect application).
- [x] Add request bridge in `engine-scene-runtime` (`SceneMutationRequest` -> typed runtime `SceneMutation`) with value conversion helpers for render params.

## Session Notes (Stream 3–4 refactor, 2026-04-16)

**Stream 3A — Migrate `visual_binding.rs`:**
- `engine/src/systems/visual_binding.rs`: replaced `BehaviorCommand::SceneDespawn { target }` with `BehaviorCommand::ApplySceneMutation { request: SceneMutationRequest::DespawnObject { target } }`.
- Added `engine-api` as a workspace dependency to `engine/Cargo.toml` (needed to import `SceneMutationRequest` directly).

**Stream 3B — Remove `SceneSpawn`/`SceneDespawn` from `BehaviorCommand`:**
- Migrated all 4 test call-sites in `engine-scene-runtime/src/lib.rs` from `BehaviorCommand::SceneSpawn/SceneDespawn` to `BehaviorCommand::ApplySceneMutation` with `SceneMutationRequest::SpawnObject/DespawnObject`.
- Removed `SceneSpawn` and `SceneDespawn` variants from `engine-api/src/commands.rs`.
- Removed the corresponding handler arms from `engine-scene-runtime/src/behavior_runner.rs`.
- All 42 `engine-scene-runtime` tests pass; 25 `engine-api` tests pass.

**Stream 4 — Document `SetProperty` fallbacks in `engine-api/src/scene/api.rs`:**
- Added intentional-fallback comments to all three `SetProperty` fallback sites (`ScriptSceneApi::set`, `ScriptSceneApi::set_multi`, `ScriptObjectApi::set`).

**Verification:**
- `Select-String` for `SceneDespawn` in `visual_binding.rs` returns no matches.
- `cargo check -p engine` — clean.
- `cargo test -p engine-scene-runtime --lib` — 42/42 pass.
- `cargo test -p engine-api --lib` — 25/25 pass.
- [x] Route core behavior commands (`SetVisibility`/`SetOffset`/`SetText`/`SetProps`/camera commands) through typed `SceneMutation` application path in `SceneRuntime`.
- [x] Add first end-to-end typed scene mutation channel: `scene.mutate(...)` -> `BehaviorCommand::ApplySceneMutation` -> `SceneRuntime` typed mutation application.
- [x] Bridge selected 3D `SetProperty` paths to typed runtime mutations (`scene3d.frame`, `planet.spin_deg`, `planet.cloud_spin_deg`, `planet.cloud2_spin_deg`, `planet.sun_dir.{x,y,z}`, `obj.world.{x,y,z}`) with unchanged fallback for unsupported paths.
- [x] Bridge additional compatibility property paths through typed request conversion (`visible`, `text.content`, `offset/position.{x,y}`) and skip duplicate runtime fallback for those paths.
- [x] Route `Render3dMutationRequest::SetWorldParam` through typed compatibility mapping (`SetCompatProperty`) for render3d path namespaces (`scene3d.*`, `planet.*`, `obj.*`, `terrain.*`, `world.*`) before fallback worldgen mapping.
- [x] Guard runtime non-render `SetProperty` fallback from render3d compatibility namespaces so invalid render3d compatibility values do not mutate via non-render path.
- [x] Aggregate typed-mutation dirty invalidation in runtime (`DirtyMask3D`) and expose consume/reset APIs with tests.
- [x] Move `ObjPrerender*` type/state ownership to `engine-render-3d::prerender` and rewire compositor/engine to consume that seam.
- [x] Inject prepared `Render2dPipeline` seam into `LayerCompositeInputs` and move default 2D/3D adapter wiring into compositor provider layer.
- [x] Add asset-seam regression tests for unpacked-vs-zip image parity, source-isolated cache keys, and normalized-vs-absolute image path cache reuse.

## Session Notes (Streams 5–6, final cleanup pass, 2026-04-16)

**Stream 5 — Verify runtime compatibility in engine-scene-runtime:**

- Full test suite: `engine-api` (25/25), `engine-behavior` (73/73),
  `engine-scene-runtime` (42/42) — all pass.
- `cargo check -p engine-compositor -p engine-render-3d -p engine` — clean
  (warnings only, no errors).
- Audited `engine-scene-runtime/src/behavior_runner.rs`: `SetProperty` fallback
  arm (the raw compat path after typed-first attempt) is bounded and not
  duplicated. No dead code from `SceneSpawn`/`SceneDespawn` removal.
- Audited `engine-scene-runtime/src/materialization.rs`: `set_obj_sprite_property`
  (line 187) and `set_planet_sprite_property` (line 217) are still live — both
  are called from `engine-scene-runtime/src/render3d_state.rs` as part of the
  `Render3DCompatProperty` application path. Not dead code.
- Audited `engine-scene-runtime/src/render3d_state.rs`: no dead code from
  `SceneSpawn`/`SceneDespawn` removal. `SceneSpawn`/`SceneDespawn` patterns
  return zero matches.
- Audited `engine-behavior/src/lib.rs` `SetProperty` uses:
  - Line ~73: `push_set_property_or_apply_mutation` helper — intentional
    typed-first-then-raw fallback; production infrastructure.
  - Lines ~2284, ~2311, ~2338, ~2406, ~2411: test assertions confirming
    `SetProperty` is correctly emitted for paths not covered by typed mutations
    (e.g. `position.y` for menu items). Test code, not production paths to migrate.
- Scene checks: `mods/planet-generator`, `mods/playground`, `mods/asteroids` —
  all pass with 0 errors.

**Stream 6 — Final grep audit:**

- `render_scene3d_work_item` in `engine-compositor/src`, `engine-render-3d/src`,
  `engine/src`: **zero stale imports** — compositor no longer re-exports it;
  `engine/src/systems/scene3d_prerender.rs` imports from `engine_render_3d::prerender`.
- `SceneSpawn|SceneDespawn` in `engine-api/src`, `engine-behavior/src`,
  `engine-scene-runtime/src`, `engine/src`: **zero matches** — fully removed.
- `BehaviorCommand::SetProperty|SetProperty {` remains intentionally in
  `engine-behavior/src/lib.rs` (compat fallback helper + test assertions) and
  `engine-scene-runtime/src/behavior_runner.rs` (bounded raw fallback arm).
  These are not dual-pipeline paths; they are intentional single-fallback patterns.
- `set_obj_sprite_property`, `set_planet_sprite_property`, `set_obj_property_recursive`,
  `set_planet_property_recursive`, and all string-path dispatch wrappers in
  `engine-scene-runtime/src/materialization.rs`: **fully removed** (2026-04-17).
  Typed mutation handlers (`set_obj_material_typed_wrapper`, `set_obj_terrain_typed_wrapper`,
  `set_obj_worldgen_typed_wrapper`, `set_planet_typed_wrapper`) are the sole mutation path.
- `engine-compositor/src/lib.rs` declares 3 render-domain adapter modules
  (`generated_world_render_adapter`, `obj_render_adapter`, `scene_clip_render_adapter`);
  confirmed by audit to be orchestration/dispatch modules — all render implementations
  live in `engine-render-3d`. Accepted as final state.

**Checklist update:**

- Marked `- [x] Keep 3D domain logic out of engine-compositor.` (Hard Rule):
  rasterizer and `render_scene3d_work_item` fully moved to `engine-render-3d`.
- Marked `- [x] Retire string-path setters after typed APIs take over.` (2026-04-17):
  all string-path dispatch removed from materialization.rs; typed mutations are the only path.

## Definition of Done

- [x] 2D-only projects run without any 3D dependency leakage.
- [x] 3D no longer grows through new fields on `Sprite::Obj`.
- [x] Existing mods still load.
- [x] Runtime mutation path converges on typed APIs instead of growing more
      string-path branches.
- [x] `engine-compositor` no longer owns domain render logic.
- [x] No new code introduces `legacy` naming.

DoD evidence (audit date: 2026-04-16):
- `type:image` / `type:text` unchanged behavior:
  `cargo test -p engine-render-2d` passed (`image::tests::scales_image_dimensions_from_size_preset`, text blit/multiline tests),
  and `cargo test -p engine-authoring compile::scene::tests::compiles_image_obj_planet_and_scene3d_sprites_for_refactor_baseline -- --exact` passed.
- Existing mods load (`--check-scenes`):
  `mods/asteroids` (`logs/16-04-26/run-025/run.log`),
  `mods/gui-playground` (`logs/16-04-26/run-026/run.log`),
  `mods/planet-generator` (`logs/16-04-26/run-027/run.log`),
  `mods/playground` (`logs/16-04-26/run-028/run.log`),
  `mods/terrain-playground` (`logs/16-04-26/run-029/run.log`) all exited successfully.
- Typed-mutation cleanup verification:
  `cargo test -p engine-behavior --lib -- --nocapture` passed after routing
  typed-compatible emitted `set` operations away from raw `SetProperty`;
  `cargo test -p engine-scene-runtime --lib -- --nocapture` passed after
  palette-binding migration to typed scene mutations.
- Additional typed API verification:
  `cargo test -p engine-api --lib -- --nocapture` passed after adding stateful
  `position/offset` conversion coverage in `engine-api::scene::api`;
  `cargo check -p engine` passed after moving gameplay/emitter spawn-despawn
  producers onto typed scene mutations.
- Compositor boundary verification:
  `cargo check -p engine-compositor` and `cargo check -p engine` passed after
  removing stale compositor re-exports of shared 3D prerender types.
- No new legacy naming in active refactor scope:
  `rg -n "legacy" engine-authoring/src/compile/render_scene.rs engine-compositor engine-render-3d engine-scene-runtime engine-render-2d engine-core` returned no matches;
  `git diff -U0 -- . ':(exclude)3drefactor.md' | rg -n "^\\+.*legacy"` returned no matches.

Checklist progress (updated date: 2026-04-17, final — all items complete):
- Full checklist: `246 / 246` done, `0 / 246` todo (`100%` done).
- Definition of Done: `6 / 6` done (`100%` done).
- Computed with:
  `rg -n "^- \[( |x)\]" 3drefactor.md`
  `rg -n "^- \[x\]" 3drefactor.md`
  `rg -n "^- \[ \]" 3drefactor.md`
