# playground

Development sandbox mod for experiments.

## Purpose

`playground/` is the mod intended for trying new scenes, renderer ideas,
effects, and authoring experiments without disturbing the main game content.
It is also the smallest maintained contract ladder for the runtime refactor.

## Contents

- `mod.yaml` — manifest and runtime defaults
- `assets/` — local experimental assets
- `objects/` — reusable authored objects
- `scenes/` — sandbox scenes
- `schemas/` — generated mod-local schema fragments
- `stages/` — stage definitions
- `catalogs/presets.yaml` — declared mod-local controller presets used by the ladder scenes; built-in camera presets such as `obj-viewer` and `surface-free-look` stay canonical without extra catalog entries

## Canary ladder

These scenes are intentionally minimal and stay in sync with scene-contract and
world-model changes:

- `CAN-01` - `/scenes/world-model-planar/scene.yml` - explicit `render-space: 2d` + `world-model: planar-2d`
- `CAN-02` - `/scenes/world-model-euclidean/scene.yml` - explicit `render-space: 3d` + `world-model: euclidean-3d` + canonical `controller-defaults + camera-rig` for the neutral inspector path
- `CAN-03` - `/scenes/world-model-celestial/scene.yml` - explicit `render-space: 3d` + `world-model: celestial-3d` + canonical `controller-defaults + camera-rig` for `surface-free-look` + `camera-rig.surface.mode: locked`
- `SCN-04` - `/scenes/world-model-contracts/scene.yml` - contract map for the planar / euclidean / celestial ladder and the explicit surface-locked camera rule
- `BASE-01` - `/scenes/menu/scene.yml` - planar 2D menu baseline
- `BASE-02` - `/scenes/3d-scene/scene.yml` - euclidean 3D OBJ baseline authored through canonical `controller-defaults.camera-preset: obj-viewer` + `camera-rig.preset: obj-viewer`

Use this mod when you need the smallest authored proof for:

- planar vs euclidean vs celestial separation
- `controller-defaults` as the canonical scene-level preset surface
- declared preset ids in `catalogs/presets.yaml`
- explicit camera-rig and camera-surface contracts
- baseline `objects:` expansion in a euclidean scene
- repo-wide compatibility coverage for low-level `input.*camera` without teaching it as the authored source of truth in this mod

`mods/planet-generator` remains the heavier celestial canary stack and carries
the prefab merge canary pair in `catalogs/prefabs.yaml`.

## Typical usage

```bash
SHELL_ENGINE_MOD_SOURCE=mods/playground cargo run -p app
```

Contract-only validation:

```bash
cargo run -p app -- --mod-source=mods/playground --check-scenes
```
