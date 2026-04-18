# Lighting Playground

Reusable 3D scene-lighting sandbox for validating:

- `view-profile`
- `lighting-profile`
- `space-environment-profile`
- scene-level atmosphere / grading / environment behavior across different 3D objects

This mod is intentionally separate from `planet-generator`.
It tests scene look and renderer reusability, not planet-only tuning.

## Run

```powershell
cargo run -p app -- --mod-source=mods/lighting-playground
```

## Validate

```powershell
cargo run -p app -- --mod-source=mods/lighting-playground --check-scenes
```

## Current v2

- static main scene
- scene-level `view-profile`
- single-model showcase in the center
- segmented model picker
- compact dropdown selectors for view / lighting / environment profiles
- right-side tool panel
- live sliders for scene-level lighting/environment overrides

## Controls

- `1 / 2 / 3 / 4` — switch model
- `Z / X / C` — switch view profile
- `Q / W / E` — switch lighting profile
- `A / S / D` — switch environment profile
- mouse click — use the segmented model picker and dropdown selectors
- mouse drag — change sliders

The first implementation step is intentionally static. Interactive profile
switching and camera presets will be layered on after the baseline scene is stable.
