# Remaining Follow-up

Scope: remaining work after the current refactor closure pass.

Context:
- `3drefactor.md` is now closed as complete.
- Items below are follow-up product/performance/visual work, not unfinished core refactor ownership cleanup.

## 1) 2D-only Leakage Validation (DoD [x] done)

- ✅ Done (this batch): Added pure-2D regression tests:
  - `engine/src/systems/compositor/mod.rs::scene_pipeline_2d_only_does_not_schedule_3d_preparation_steps`
  - `engine/src/systems/compositor/mod.rs::composite_2d_only_scene_runs_without_3d_world_resources`
- These tests assert:
  - no 3D preparators are scheduled,
  - no 3D atlas/mesh prerender artifacts are requested from runtime,
  - `composite_scene` can execute without `engine-3d` feature side effects.
- Status: validated with:
  - `cargo test -p engine scene_pipeline_2d_only_does_not_schedule_3d_preparation_steps -- --nocapture`
  - `cargo test -p engine composite_2d_only_scene_runs_without_3d_world_resources -- --nocapture`

Next follow-up:
- Kept and extended with feature-gating regression coverage:
  - added `scene_pipeline_3d_prerender_scene_schedules_obj_prepass_state`.

## 2) Script Mutation API

- Runtime-side raw `SetProperty` fallback is removed.
- Supported `scene.set(path, value)` routes are translated to typed mutations in:
  - `engine-api/src/commands.rs`,
  - `engine-api/src/scene/api.rs`.
- Remaining follow-up is only product/API design:
  - decide whether unsupported `scene.set(...)` paths should remain silent no-op,
  - or emit script diagnostics at API boundary.

## 3) Docs and Status Consistency (DoD [x] done)

- Keep these aligned after further edits:
  - `3drefactor.md`,
  - `engine-scene-runtime/README.md`,
  - `engine-api/README.md`,
  - `AUTHORING.md`,
  - `CHANGELOG.md`.
- Current state:
  - runtime mutation flow docs are aligned with the typed-only implementation path.

## 4) Performance Cleanup Passes (DoD [ ] partially done)

- Keep running the cloud/LOD bench matrix from `planetgen.opt.impl.md` after any changes.
- Current outstanding watch points:
  - ensure no regressions in FPS after lower resolution “reset” cases,
  - verify no recurring GPU/CPU bottlenecks from input/event thread or buffer region invalidation,
  - confirm sphere clip artifacts and full-screen bar artifacts are no longer reproducible at 16:9.
- Candidate files:
  - `mods/planet-generator/` scene runtime and shader params,
  - `engine-render-3d/src/{pipeline,raster}.rs`,
  - `engine-compositor/src/{layer_compositor.rs,scene_compositor.rs}`.

## 5) Lighting/visual coherence (DoD [x] in progress)

- Continue unifying dark-side behavior through scene-level `lighting` + surface profile in 3D renderer.
- Candidate files:
  - `engine-core/src/scene/model.rs` + `schemas/scene.schema.yaml`,
  - `engine/src/systems/compositor/mod.rs`,
  - `engine-render-3d/src/pipeline/generated_world_renderer.rs`.
