# Shell Engine Mod System

## Overview

Mods are self-contained content packages loaded by the engine at startup.
A mod can be an unpacked directory or a `.zip` archive. The engine selects
which mod to load via:

- `--mod <name>` to resolve `mods/<name>/`
- `--mod-source <path>` for an explicit directory or zip
- `SHELL_ENGINE_MOD_SOURCE` environment variable

If no mod is specified, `app` starts the interactive launcher.

For the runtime/component refactor rollout, use [oop.impl.md](oop.impl.md) as
the authoritative ledger and tag related work with its task ids.

## Current bundled mods

| Mod | Purpose | Run |
|-----|---------|-----|
| `playground` | general engine sandbox and scene-contract ladder | `cargo run -p app -- --mod playground` |
| `planet-generator` | procedural world/planet tuning + prefab merge and shared scene-camera canaries | `cargo run -p app -- --mod-source=mods/planet-generator` |
| `qwack-3d` | euclidean gameplay/camera canary with FPS and chase presets | `cargo run -p app -- --mod-source=mods/qwack-3d` |
| `qwack-3d-celestial` | celestial gameplay/camera canary with surface FPS and orbit presets | `cargo run -p app -- --mod-source=mods/qwack-3d-celestial` |
| `gui-playground` | widget and GUI scripting playground | `cargo run -p app -- --mod-source=mods/gui-playground` |
| `terrain-playground` | terrain/worldgen experiments | `cargo run -p app -- --mod-source=mods/terrain-playground` |
| `asteroids` | gameplay-heavy orbital prototype | `cargo run -p app -- --mod-source=mods/asteroids` |

## Mod Structure

Minimum structure:

```text
mods/my-mod/
+-- mod.yaml
+-- scenes/
    +-- hello/
        +-- scene.yml
        +-- layers/
            +-- main.yml
```

Typical larger mods also contain:

- `assets/`
- `objects/`
- `behaviors/`
- `catalogs/`
- `palettes/`
- `schemas/`

## `mod.yaml`

```yaml
name: my-mod
version: "0.1.0"
description: "My custom mod"
entrypoint: /scenes/hello/scene.yml
display:
  min_colours: 256
  min_width: 120
  min_height: 30
  world_render_size: 120x30
  presentation_policy: stretch
```

`world_render_size` defines the authored world canvas. `presentation_policy`
controls how that canvas is shown in the SDL2 window.

When a mod needs sharper HUD/UI than the world pass, also set:

```yaml
display:
  world_render_size: 640x360
  ui_render_size: 1280x720
  ui_layout_size: 1280x720
```

Use `ui_layout_size` equal to `ui_render_size` for a native higher-resolution
UI layout, or equal to `world_render_size` if you are temporarily preserving an
older HUD coordinate space.

## Asset Loading

- Asset paths use a leading `/`, resolved relative to the mod root.
- The same paths work for both directory and zip-packaged mods.
- Named mod behaviors are loaded from top-level `behaviors/*.yml`; those YAML
  wrappers may point at external Rhai via `src`.

Example:

```text
/assets/images/logo.png  -> mods/my-mod/assets/images/logo.png
/assets/images/logo.png  -> my-mod.zip!/assets/images/logo.png
```

## Validation

Check a mod without starting the full runtime:

```bash
cargo run -p app -- --mod-source=mods/my-mod --check-scenes
```

## Notes

- `playground` is the scene-contract ladder (`CAN-01` through `BASE-02`; see
  `mods/playground/README.md`).
- Its euclidean and celestial canaries are authored through
  `controller-defaults` + `camera-rig`; direct low-level camera input is now
  only part of repo-wide compatibility coverage outside the main canary ladder.
- `planet-generator` is the heavier celestial testbed and also carries the
  prefab merge canaries in `catalogs/prefabs.yaml` plus the shared scene-camera
  canaries used by the `camera-rig` / camera-surface contract.
- `qwack-3d` is the compact euclidean gameplay/camera canary; use
  `/scenes/chase/scene.yml` for authored chase-camera coverage through
  `controller-defaults` + `camera-rig`.
- `qwack-3d-celestial` is the celestial gameplay/camera sibling and keeps the
  authored `surface FPS` + `orbit` contracts separate from the euclidean chase canary.
- `planet-generator` and `terrain-playground` are the strongest current examples for
  current `world://` and generated-world flows.
- `gui-playground` is the strongest current example for the widget system.
- `playground` remains the general-purpose scene/render sandbox.
- Reusable scene look assets can live directly in mods under:
  - `/view-profiles/`
  - `/lighting-profiles/`
  - `/space-environment-profiles/`
- Those assets resolve into one effective scene-wide lighting/environment
  contract at runtime, so mods can author reusable 3D scene look presets
  without making renderers aware of gameplay concepts such as planets.
