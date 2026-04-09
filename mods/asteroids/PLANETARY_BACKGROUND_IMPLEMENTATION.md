# Planetary Background Implementation Plan (Asteroids)

This plan translates the full planetary spec into engine-compatible delivery
steps for the current `scene3d` pipeline.

## Current Engine Constraints

- Background must be one `scene3d` sprite source.
- Clip animation is prerendered into an atlas before scene start.
- Runtime should only:
  - pick clip frame id
  - apply tiny root drift/parallax offset
- Real-time shadow maps are not available; use fake shadow proxies.

## Phase 1 — Foundation (Done)

- [x] Replace fragmented background layers with one prefab:
  - `/assets/3d/solar-system.scene3d.yml`
- [x] Keep gameplay and HUD as separate layers above the background.
- [x] Keep runtime background control minimal (`scene3d.frame`, `offset.x/y`).

## Phase 2 — Saturn Core (Done)

- [x] Saturn-like central planet with rings as annulus meshes.
- [x] Atmospheric shell / glow elements.
- [x] Dedicated ring meshes:
  - back/front visible rings
  - ring-shadow proxy variants
- [x] Fixed sun-like key/fill/rim light setup.

## Phase 3 — Moons + Belt (In Progress)

- [x] Multi-moon orbital system aligned to HTML reference (5 primary moons).
- [x] Belt rocks with increased density around Saturn orbit band.
- [x] Moon and ring fake shadows authored as proxies.
- [x] Belt composition split into explicit visual groups matching the final spec:
  - hero belt chunk
  - background belt chunk
  - dust plane
  - density shadow layer

## Phase 4 — Clip Quality & Stability (Done for current target)

- [x] Increase clip resolution to improve smoothness:
  - `solar-orbit` keyframes: `96`
- [x] Use a loop-safe frame selector from Rhai.
- [x] Keep transitions continuous (no visibility toggles/snap opacity to 0).

## Phase 5 — Performance Budgeting (In Progress)

- [x] Keep scene to a single prerendered source.
- [x] Validate scene checks and runtime logs after each major update.
- [ ] Add explicit triangle/draw-call budget table for authored meshes in this mod.
- [ ] Add a benchmark snapshot (before/after planetary pass) in docs.

## Phase 6 — Gameplay Integration (Pending)

- [ ] Expose belt region constants (inner/outer radius mapping) to gameplay logic.
- [ ] Hook belt region to:
  - ambience mix
  - HUD zone indicator
  - optional spawn policy

## Runtime Contract (Must Stay True)

- Do not animate child nodes from Rhai.
- Do not mutate per-object scene3d properties in gameplay loop.
- Do not spawn/despawn background elements at runtime.
- Only frame selection + root drift are allowed background mutations.

## Next Concrete Work Item

Author belt dust plane and belt density shadow as explicit scene objects, then
validate:

1. `cargo run -p app -- --mod-source=mods/asteroids --check-scenes`
2. `cargo run -p app -- --mod-source=mods/asteroids --start-scene /scenes/game/scene.yml --bench 2 --logs`
