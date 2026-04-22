# qwack-3d

Reference gameplay canary for a future Quake-like FPS mod on the explicit
euclidean 3D world contract.

## Current state

- multi-scene gameplay canary
- explicit `render-space: 3d`
- explicit `world-model: euclidean-3d`
- declared controller presets in `catalogs/presets.yaml`
- authored scenes select camera policy through canonical
  `controller-defaults + camera-rig`
- direct `input.*camera` blocks are not part of the authored canary path here;
  they only exist as lowered compatibility output during rollout
- scene graph documents arena, combat, and systems ownership without depending on the unfinished 3D gameplay stack

## Scenes

- `/scenes/main/scene.yml` — base euclidean 3D identity scene with authored
  `controller-defaults.camera-preset: qwack-fps` +
  `camera-rig.preset: qwack-fps`
- `/scenes/hooks/scene.yml` — arena contract (`qwack-fps`, `qwack-walker`,
  `qwack-hud`, room metrics, collision, spawn) authored through the FPS camera
  rig canary
- `/scenes/combat/scene.yml` — combat contract (`move`, `aim`, `fire`,
  `damage`, `pickup`) sharing the same authored camera rig contract
- `/scenes/chase/scene.yml` — chase-camera contract authored through
  `controller-defaults.camera-preset: qwack-chase` +
  `camera-rig.preset: qwack-chase`
- `/scenes/systems/scene.yml` — ownership split canary for future euclidean gameplay services

## Check

```bash
cargo run -p app -- --mod-source=mods/qwack-3d --check-scenes
```
