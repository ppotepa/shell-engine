# Refactor Handoff (Leftover Work)

Scope: what is still open after current batch.

## 1) 2D-only Leakage Validation (DoD [ ] not done)

- Add a small regression test suite that loads a pure-2D scene (no `obj`/`planet`/`scene3d`) and asserts:
  - no 3D preparators are scheduled,
  - no 3D atlas/mesh prerender artifacts are requested from runtime,
  - `composite_scene` can execute without `engine-3d` feature side effects.
- Candidate files:
  - `engine/src/systems/scene_lifecycle/mod.rs` (scene boot hooks),
  - `engine/src/systems/compositor/mod.rs` (runtime dispatch points),
  - `engine-compositor/src/compositor.rs` (prepared layer path should tolerate empty 3D buckets),
  - tests in `engine-scene-runtime`/`engine-compositor` as currently used for existing compile/regression checks.

## 2) Legacy Script API Retirement (DoD [ ] not done)

- Current behavior is typed-first, but raw `scene.set(path, value)` compatibility remains:
  - `engine-api/src/scene/api.rs`,
  - `engine-scene-runtime/src/behavior_runner.rs`.
- To fully retire:
  - inventory remaining unsupported `SetProperty` routes,
  - add typed API equivalents in `engine-api`,
  - keep temporary deprecation window, then gate removal by migration completion.

## 3) Docs and Status Consistency (DoD [x] mostly done)

- Keep these aligned after further edits:
  - `3drefactor.md`,
  - `engine-scene-runtime/README.md`,
  - `engine-api/README.md`,
  - `AUTHORING.md`,
  - `CHANGELOG.md`.
- Current update completed:
  - `3drefactor.md` reflects that typed path is primary and compatibility is API-edge only.

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

