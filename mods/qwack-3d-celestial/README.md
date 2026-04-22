# qwack-3d-celestial

Reference gameplay canary for a future celestial variant of `qwack-3d`.

## Current state

- multi-scene gameplay canary
- explicit `render-space: 3d`
- explicit `world-model: celestial-3d`
- declared controller presets in `catalogs/presets.yaml`
- authored scenes select camera policy through canonical
  `controller-defaults + camera-rig`
- direct `input.*camera` blocks are not part of the authored canary path here;
  they only exist as lowered compatibility output during rollout
- euclidean chase-camera coverage now lives in `mods/qwack-3d/scenes/chase/scene.yml`;
  this celestial sibling stays focused on surface-FPS and orbit contracts
- local celestial catalogs included so authored `celestial:` bindings are validated during scene checks
- scene graph documents surface, orbit, and systems ownership without depending on the unfinished runtime stack

## Scenes

- `/scenes/main/scene.yml` — base celestial identity scene with explicit local
  `celestial:` binding plus canonical
  `controller-defaults.camera-preset: qwack-celestial-fps` +
  `camera-rig.preset: qwack-celestial-fps`
- `/scenes/hooks/scene.yml` — surface contract (`surface spawn`, `radial
  gravity`, `local horizon`, ground combat) sharing the authored FPS camera rig
- `/scenes/orbit/scene.yml` — orbit contract using explicit `focus-site` and
  canonical `controller-defaults.camera-preset: qwack-celestial-orbit` +
  `camera-rig.preset: qwack-celestial-orbit`
- `/scenes/systems/scene.yml` — ownership split canary for future celestial gameplay services

## Catalogs

- `/catalogs/celestial/bodies.yaml`
- `/catalogs/celestial/regions.yaml`
- `/catalogs/celestial/systems.yaml`
- `/catalogs/celestial/sites.yaml`
- `/catalogs/presets.yaml`

These are intentionally minimal and exist to validate authored celestial scene
contracts before gameplay/runtime systems are fully wired.

## Check

```bash
cargo run -p app -- --mod-source=mods/qwack-3d-celestial --check-scenes
```
