# 3D Refactor Checklist

Status: `IN_PROGRESS`
Last updated: `2026-04-17`
Owner: `engine/render/runtime`

## 1. Governance / Architecture Rules

- [x] Do not keep permanent compatibility layers in renderer/compositor internals.
- [x] Treat planets as one producer of 3D data (renderer remains domain-agnostic).
- [x] Keep 3D domain logic out of `engine-compositor`.
- [x] Runtime mutation path is unified around typed APIs; supported `scene.set(...)` paths are translated at the API edge.
- [x] `engine-compositor` no longer owns domain render logic (3D logic lives in `engine-render-3d`).

## 2. Crate Ownership

- [x] `engine-render-2d` owns 2D rendering.
- [x] `engine-render-3d` owns 3D rendering.
- [x] `engine-compositor` assembles frame output.
- [x] `engine-scene-runtime` owns runtime state/mutations.
- [x] `engine-worldgen` owns worldgen + mesh build key policy.

## 3. 3D/2D Separation (PR7 Scope)

- [x] Scene composition routes through domain pipelines, not sprite-specific render logic in compositor.
- [x] Shared render contracts moved to typed seams (`render_types`, scene graph instances, pipeline adapters).
- [x] Final cleanup of stale comments/docs in active refactor tracking updated.

## 4. Runtime Mutation Convergence

- [x] Collapse mutation handling toward one typed implementation path.
- [x] Supported `scene.set(...)` paths route to typed mutation handling at the API edge.
- [x] Retire raw string-path runtime setters.
- [x] Avoid adding any new string-path branches.

## 5. Temporary / Dual Paths

- [x] Keep `type: obj` / `type: planet` / `type: scene3d` working during migration.
- [x] Do not build permanent dual execution paths.
- [x] Move compilation into new model instead of duplicating runtime compilers.

## 6. Final DoD Validation

- [x] 2D-only project validation pass (no 3D leakage).
- [x] 3D no longer grows through new `Sprite::Obj` fields.
- [x] Runtime mutation path fully converged on typed mutations.
- [x] `engine-compositor` no longer owns domain render logic.

## 7. Current Performance Workstream Link

- Active optimization execution and per-step status live in `planetgen.opt.impl.md`.
- LOD seam + worldgen LOD clamp helper: implemented.
- Cloud cadence/reuse path (generated-world): implemented, pending final benchmark matrix.
- Cloud spike smoothing: implemented one-expensive-cloud-refresh-per-frame guard + stale-cache fallback for second cloud layer.
- Added dedicated cloud-heavy bench scene for generated-world path:
  - `mods/asteroids/scenes/bench-cloud/scene.yml`
  - runtime command: `--mod asteroids --start-scene /scenes/bench-cloud/scene.yml --bench 3 --opt --skip-splash`
- Latest cloud-heavy benchmark (`reports/benchmark/20260417-120454.txt`) after LOD cap+bias tuning:
  - FPS ~16.6
  - compositor ~58.16ms
  - 3D tris ~425,927 avg
- Latest cloud-heavy benchmark after cloud mesh decimation (`reports/benchmark/20260417-121245.txt`):
  - FPS ~18.3
  - compositor ~60.79ms avg
  - 3D tris ~245,134 avg
- Latest 6s cloud-heavy benchmark after cloud spike smoothing (`reports/benchmark/20260417-121743.txt`):
  - FPS ~17.8
  - compositor ~54.11ms avg, p95 ~53.22ms
  - 3D cloud2 p50 remains low (~2.9ms) while avoiding double cloud re-render bursts on most frames
- Latest 6s cloud-heavy benchmark after startup surface LOD ramp (`reports/benchmark/20260417-121851.txt`):
  - FPS ~22.8
  - compositor ~39.36ms avg, p95 ~43.40ms
  - strong cold-start reduction: first-frame spikes still exist but shorter and less dominant
- Latest 10s cloud-heavy benchmark (`reports/benchmark/20260417-122026.txt`) confirms steady-state improvement:
  - FPS ~23.8
  - compositor ~36.31ms avg, p95 ~38.09ms
  - cloud passes steady-state low (`cloud1 p50 ~2.2ms`, `cloud2 p50 ~2.1ms`, `halo p50 ~2.5ms`)
- Recent stabilization fixes:
  - startup window and viewport scaling in app launch now auto-resolve `sdl-pixel-scale` from `display.render_size` when CLI does not override it (`app/src/main.rs`).
  - direct layer compositor now always uses scratch path for layers containing any 3D sprite to avoid stale black stripes under mixed 2D/3D composition.
- Planet-generator UI/model runtime bridge now supports runtime model source switching in `mods/planet-generator/scenes/main/main.rhai`:
  - number keys `1..4` and mouse buttons `model-btn-*` select renderer model source (`world://32`, `sphere.obj`, `cube.obj`, `suzanne.obj`).
  - model HUD strip added in `mods/planet-generator/scenes/main/layers/hud-models.yml` and wired into scene widgets.
  - worldgen pushes are now conditional (`world.*` only for world mode), preventing irrelevant updates when object modes are active.
- Planet-generator preset label sprites are now correctly bound to preset widgets (`preset-label-*` ↔ `preset-btn-*`) so startup check `gui-widget-bindings` passes with non-colliding HUD IDs.

## 8. Renderer-Agnostic Lighting Floor + Surface Flight (Implemented)

- [x] Scene-level `lighting.ambient-floor` introduced in `engine-core`, threaded through compositor and raster params.
- [x] `ambient-floor` now participates in scene schema (`schemas/scene.schema.yaml`) and docs.
- [x] Free-look surface mode controls added to `engine-core::scene::FreeLookCameraControls`, scene runtime, and planet-generator scene.
- [x] Planet-generator scene now documents and enables surface-mode controls for first-person-style walk/fly testing.
