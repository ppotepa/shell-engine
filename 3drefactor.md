# 3D Refactor Checklist

This file is the working checklist for the 2D/3D architecture refactor.

The goal is to keep 2D as a first-class path, make 3D a real subsystem, and
stop pushing rendering semantics through `Sprite::Obj`.

## Hard Rules

- [ ] Do not introduce new identifiers, modules, files, or types with `legacy`
      in the name.
- [ ] Do not keep permanent compatibility layers, duplicate pipelines, or
      shadow implementations in the engine.
- [ ] Keep `type: image`, `type: text`, and existing 2D layout behavior fully
      supported.
- [ ] Keep existing YAML scenes and mods loading during the migration.
- [ ] Treat planets as one producer of 3D data, not as a special renderer.
- [ ] Keep 3D domain logic out of `engine-compositor`.
- [ ] Keep asset loading semantics unchanged for unpacked mods and zip mods.
- [ ] Do not let runtime mutation semantics depend on stringly-typed render
      internals going forward.

## End State

- [ ] `engine-render-2d` owns 2D rendering.
- [ ] `engine-render-3d` owns 3D rendering.
- [ ] `engine-compositor` only assembles frame output.
- [ ] `engine-scene-runtime` owns runtime state and mutations.
- [ ] `engine-authoring` owns authored AST and scene compilation.
- [ ] `engine-asset` owns image/mesh/material decode and cache.
- [ ] `engine-worldgen` owns world generation and mesh build keys.

## Crate Boundaries

### engine-core

- [ ] Add `render_types/mod.rs`.
- [ ] Add `render_types/viewport.rs`.
- [ ] Add `render_types/transform_2d.rs`.
- [ ] Add `render_types/transform_3d.rs`.
- [ ] Add `render_types/camera_2d.rs`.
- [ ] Add `render_types/camera_3d.rs`.
- [ ] Add `render_types/light_3d.rs`.
- [ ] Add `render_types/material.rs`.
- [ ] Add `render_types/dirty.rs`.
- [ ] Add `render_types/render_scene.rs`.
- [ ] Keep these types backend-neutral and authoring-neutral.

### engine-authoring

- [ ] Add `document/viewport3d.rs`.
- [ ] Add `document/render_scene3d.rs`.
- [ ] Add `document/material.rs`.
- [ ] Add `document/atmosphere_profile.rs`.
- [ ] Add `document/world_profile.rs`.
- [ ] Add `document/camera_profile.rs`.
- [ ] Add `compile/compile_render_scene.rs`.
- [ ] Add `compile/compile_2d.rs`.
- [ ] Add `compile/compile_3d.rs`.
- [ ] Add `validate/render3d.rs`.
- [ ] Compile `obj`, `planet`, and `scene3d` directly into the new intermediate
      model without parallel compiler paths.

### engine-scene-runtime

- [ ] Add `mutations.rs`.
- [ ] Add `render3d_state.rs`.
- [ ] Add `dirty_tracking.rs`.
- [ ] Keep `SceneRuntime` as the object graph and mutation center.
- [ ] Add typed 3D mutations.
- [ ] Collapse mutation handling toward one typed implementation path.

### engine-render-2d

- [ ] Create crate `engine-render-2d`.
- [ ] Add `api.rs`.
- [ ] Move image rendering here.
- [ ] Move text rendering here.
- [ ] Move vector rendering here.
- [ ] Move 2D sprite dispatch here.
- [ ] Move layout helpers here.
- [ ] Move container rendering here.

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
- [ ] Add `mesh/mod.rs`.
- [ ] Add `mesh/asset_mesh.rs`.
- [ ] Add `mesh/generated_mesh.rs`.
- [ ] Add `mesh/cache.rs`.
- [x] Add `pipeline/mod.rs`.
- [ ] Add `pipeline/renderer.rs`.
- [ ] Map `Obj`, `Planet`, and `Scene3D` directly into the 3D scene graph.
- [ ] Add `prerender/mod.rs`.
- [ ] Add `prerender/scene3d_atlas.rs`.
- [ ] Add `prerender/scene3d_runtime_store.rs`.
- [ ] Add `prerender/scene3d_prerender.rs`.

### engine-compositor

- [ ] Remove direct ownership of 2D rendering code.
- [ ] Remove direct ownership of 3D rendering code.
- [ ] Keep only frame assembly, layer ordering, blending, and postfx.
- [ ] Narrow `CompositeParams`.
- [ ] Consume prepared 2D and 3D inputs instead of raw authored sprite detail.

### engine-asset

- [ ] Move image decode/cache concerns here.
- [ ] Add `mesh_repository.rs`.
- [ ] Add `material_repository.rs`.
- [ ] Add `build_keys.rs`.
- [ ] Expose shared image and mesh access for both renderers.

### engine-worldgen

- [ ] Add stable `MeshBuildKey`.
- [ ] Move geometry build key creation here.
- [ ] Keep procedural world generation independent of the render loop.

## Core Types

- [ ] Define `ViewportRect`.
- [ ] Define `Transform2D`.
- [ ] Define `Transform3D`.
- [ ] Define `Camera2DState`.
- [ ] Define `Camera3DState`.
- [ ] Define `LightKind3D`.
- [ ] Define `Light3D`.
- [ ] Define `DirtyMask3D`.
- [ ] Define `CompiledRenderScene`.
- [ ] Define `Scene3DInstance`.
- [ ] Define `Node3DInstance`.
- [ ] Define `Renderable3D`.
- [ ] Define `MeshInstance`.
- [ ] Define `GeneratedWorldInstance`.
- [ ] Define `Render2dInput`.
- [ ] Define `Render3dInput`.
- [ ] Define `Render3dOutput`.
- [ ] Define `SceneMutation`.
- [ ] Define `Render3DMutation`.
- [ ] Define `MeshBuildKey`.

## Migration Rules

- [ ] Keep `type: image` untouched.
- [ ] Keep `type: text` untouched.
- [ ] Keep `type: obj`, `type: planet`, and `type: scene3d` working while they
      are still first-class authored forms.
- [ ] Do not build permanent dual execution paths to support migration.
- [ ] Route new behavior features through typed mutation APIs, not new string
      paths.

## Dirty Flags and Invalidation

- [ ] Add transform-only invalidation.
- [ ] Add camera-only invalidation.
- [ ] Add lighting-only invalidation.
- [ ] Add material-only invalidation.
- [ ] Add atmosphere-only invalidation.
- [ ] Add mesh-only invalidation.
- [ ] Add worldgen rebuild invalidation.
- [ ] Add visibility-only invalidation.
- [ ] Document which mutation sets which mask.
- [ ] Add runtime diagnostics for rebuild causes and counts.

## Asset and Build Keys

- [ ] Stop constructing geometry keys inside the render hot path.
- [ ] Replace URI rewrite semantics with typed build keys before render.
- [ ] Keep URI parsing out of the core render API.
- [ ] Allow one image asset to be used as 2D sprite input and as 3D texture
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
- [ ] Keep 3D behavior unchanged in this PR.

### PR2 - Real 3D Input Model

- [x] Add concrete scene graph types to `engine-render-3d`.
- [x] Map authored 3D sprite forms directly into scene graph nodes.
- [ ] Stop unpacking full 3D semantics directly inside sprite rendering.

### PR3 - Move Scene3D Prerender into 3D Domain

- [ ] Move Scene3D atlas ownership to `engine-render-3d`.
- [ ] Move Scene3D runtime store ownership to `engine-render-3d`.
- [ ] Move Scene3D prerender pipeline to `engine-render-3d`.
- [ ] Keep scene pipeline orchestration in `engine`.

### PR4 - Asset and Mesh Build Layer

- [ ] Move image decode/cache into `engine-asset`.
- [ ] Add mesh repository APIs.
- [ ] Add material repository APIs.
- [ ] Introduce `MeshBuildKey`.
- [ ] Remove geometry key construction from render paths.

### PR5 - Typed Runtime Mutations

- [ ] Add typed scene mutations.
- [ ] Add typed 3D mutations.
- [ ] Collapse `SetProperty` handling onto typed mutations without a second
      runtime path.
- [ ] Wire dirty flag updates from typed mutations.

### PR6 - New 3D Authoring Surface

- [ ] Add new 3D authored document types.
- [ ] Add new validation rules.
- [ ] Replace old authored forms by moving compilation into the new model, not
      by keeping duplicate compilers.

### PR7 - Simplify engine-compositor

- [ ] Remove direct render-domain logic from `engine-compositor`.
- [ ] Keep only frame composition concerns.
- [ ] Narrow public compositor APIs to frame assembly inputs.

### PR8 - Public Typed API

- [ ] Add typed public scene mutation APIs in `engine-api`.
- [ ] Retire string-path setters after typed APIs take over.
- [ ] Update scripting documentation when the typed API lands.

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

## Definition of Done

- [ ] 2D-only projects run without any 3D dependency leakage.
- [ ] 3D no longer grows through new fields on `Sprite::Obj`.
- [ ] Existing mods still load.
- [ ] Runtime mutation path converges on typed APIs instead of growing more
      string-path branches.
- [ ] `engine-compositor` no longer owns domain render logic.
- [ ] No new code introduces `legacy` naming.
