# Planet Generator Mod

Procedural planet viewer with a compact modernized multi-tab tool panel and separated live value readouts.

The mod renders the world pass at `640x360`, then composites HUD/UI at native
`1280x720`. The tool panel is authored directly in `1280x720` UI space so the
controls stay denser and more readable than the underlying 3D scene.

## Running

```bash
SHELL_ENGINE_MOD_SOURCE=mods/planet-generator cargo run -p app
```

## Controls

| Key / Input | Action |
|-------------|--------|
| `1` / `2` / `3` / `4` (or mouse click) | Switch model: Planet / Sphere / Cube / Suzanne |
| `Q` / `W` / `E` / `Y` / `U` (or mouse click) | Switch tab: Continents / Mountains / Climate / Visual / Atmosphere |
| Slider drag | Adjust parameter value with mouse |
| `F1`–`F7` | Load preset: Earth / Mars / Ocean / Desert / Ice / Volcanic / Archipelago |
| Preset dropdown | Open compact preset list and select with mouse |
| `Randomize` button | Randomize all parameters |
| `R` | Toggle planet auto-rotation on/off |
| `Delete` | Reset to Earth defaults |
| `Ctrl+F` | Toggle orbit / free-look camera (WASD move, Q/E altitude) |
| `F10` or `Vehicle` button | Package the current generator state and launch the configured vehicle scene |
| `F9` / `C` | Toggle vehicle profile for the launch handoff: `arcade` / `sim-lite` |
| `H` / `J` | Toggle launch assists: `HOLD ALT` / `HOLD HDG` |

## Runtime modes

- `generator` mode is the default authoring/view mode; sliders, tabs, presets, randomize, and reset stay active.
- `F10` or `Vehicle` writes a canonical vehicle launch packet into `/mods/planet-generator/vehicle/handoff` and jumps cross-mod to the configured vehicle consumer.
- The default target is now `mods/vehicle-playground` scene `vehicle-playground-vehicle`.
- Launch routing is controlled by:
  - `/mods/planet-generator/vehicle/target_mod_ref`
  - `/mods/planet-generator/vehicle/target_scene_id`
- `vehicle profile` can be toggled with `F9` or `C` here to configure the handoff packet:
  - `arcade` and `sim-lite` are handed off as typed profile ids only.
  - Runtime handling of those profiles lives in the vehicle consumer / `engine-vehicle`, not in this scene.
- `vehicle assists` can be toggled here to configure the handoff packet:
  - `HOLD ALT`
  - `HOLD HDG`
- The generator is treated as a producer of generated-planet data and vehicle launch intent, not the place to evolve vehicle runtime behavior or packet kind/version semantics.
- `engine-vehicle` owns the launch/return packet contract; this mod only supplies vehicle-domain environment, profile, assist, and UI state.
- Return packets are applied here only to restore generator-facing state such as environment parameters, launch profile/assists, next spawn altitude, and the telemetry strip.
- `vehicle profile` and both assist toggles are persisted in game state and restored on next run, so the next launch resumes the last vehicle setup.
- `world.body_patch("generated-planet", ...)` now keeps the celestial runtime body in sync with the generator state every frame.
- Free-look surface mode now follows the scene `focus-body` render shell from the
  runtime body patch. In this mod the patched body keeps `radius_px = 1.0` for the
  preview shell while `surface_radius` stays in simulation world units.

## Scene structure

- `scenes/main/scene.yml` — single scene, orbit-camera + free-look-camera (surface mode enabled)
- `scenes/main/layers/planet.yml` — OBJ planet mesh (`world://32`)
- `scenes/main/layers/hud-tabs.yml` — tab bar (top-right, authored as `type: tabs`)
- `scenes/main/layers/hud-models.yml` — model selector row (Planet/Sphere/Cube/Suzanne, authored as `type: segmented-control`)
- `scenes/main/layers/hud-panel.yml` — right-side parameter rail background in native `1280x720` UI space
- `scenes/main/layers/hud-sliders.yml` — compact slider layer with active-tab header, summary, widened tracks, and right-aligned live values
- `scenes/main/layers/hud-actions.yml` — Randomize / Vehicle / Reset + compact bounded handoff/profile/assist status row
- `scenes/main/layers/hud-presets.yml` — compact preset dropdown + popup list
- `scenes/main/layers/hud-stats.yml` — compact bounded telemetry strip (bottom-left)
- `scenes/main/main.rhai` — thin scene orchestrator; imports the generator modules below
- `scripts/std/bootstrap.rhai` — local bootstrap helper
- `scripts/std/math.rhai` — shared numeric helpers used by generator render throttling
- `scripts/std/runtime_scene.rhai` — scene object text/fg/visible helpers over `runtime.scene.objects.find(...)`
- `scripts/generator/state.rhai` — generator bootstrap and derived radius state
- `scripts/generator/input.rhai` — tab/model/actions/preset input flow
- `scripts/generator/presets.rhai` — preset data and preset/randomize/reset application
- `scripts/generator/params.rhai` — parameter schema, normalize/denormalize, per-tab labels and values
- `scripts/generator/gui_sync.rhai` — local state <-> slider widget sync
- `scripts/generator/body_sync.rhai` — generated body builder and runtime `world.body_patch("generated-planet", ...)`
- `scripts/generator/vehicle_handoff.rhai` — vehicle profile/assist persistence plus launch/return packet flow
- `scripts/generator/hud.rhai` — tab/model/action highlights, parameter labels, telemetry strip
- `scripts/generator/render_push.rhai` — mesh/generator push plus visual/atmosphere/light updates; `local.render_push_throttle_ms` controls generator push throttling and now defaults to realtime (`0.0`)
- `mods/vehicle-playground/scenes/vehicle/*` — canonical vehicle consumer, camera rig, HUD, body patch, and return flow

## Parameters

### Continents tab
- **SEED** — world generation seed (0–9999)
- **OCEAN** — ocean coverage fraction (1–99%)
- **CONTINENT SZ** — continent scale (0.5–10)
- **COAST CHAOS** — continent warp / coastline complexity (0–2)
- **OCTAVES** — noise octaves for continents (1–8)

### Mountains tab
- **MTN SPACING** — mountain scale / frequency (1–15)
- **MTN HEIGHT** — mountain strength (0–100%)
- **RIDGE DETAIL** — ridge octaves (1–8)

### Climate tab
- **MOISTURE SZ** — moisture scale (0.5–8)
- **ICE CAPS** — polar ice strength (0–3)
- **ALT COOLING** — altitude lapse rate (0–1.5)
- **RAIN SHADOW** — rain shadow effect (0–1)

### Visual tab
- **RESOLUTION** — mesh subdivisions (32/64/128/256/512, power-of-2 steps)
- **DISPLACEMENT** — surface displacement scale (0–60%)
- **COLORING** — biome / elevation / moisture
- **ROTATION** — rotation speed deg/sec (0–10)
- **SUN AZIMUTH** — sun angle around Y axis (0–360°)
- **SUN ELEVATION** — sun elevation angle (-10–85°)
- **AMBIENT** — ambient light level (0–50%)

## Presets

| F-key | Preset | Description |
|-------|--------|-------------|
| F1 | Earth | Balanced continents, biome climate |
| F2 | Mars | Mostly dry, high mountains, red palette |
| F3 | Ocean | High ocean coverage, tropical moisture |
| F4 | Desert | Low ocean, minimal rainfall |
| F5 | Ice | Strong polar caps, cold lapse rate |
| F6 | Volcanic | Extreme terrain displacement, high ridges |
| F7 | Archipelago | High ocean, island chains |

## Performance presets (practical)

Use these as starting points when tuning quality vs FPS on the current CPU software 3D path.

- **Balanced (default gameplay target)**:
  - `RESOLUTION`: `64` or `128`
  - `DISPLACEMENT`: `0.12–0.22`
  - Atmosphere sliders: moderate (`density/height` mid range)
  - Good compromise for interactive camera movement and editing.

- **Look-dev / hero stills**:
  - `RESOLUTION`: `128` (or `256` for static shots)
  - Atmosphere: higher haze/rayleigh values
  - Best visual fidelity, but expect clearly lower FPS while moving.

- **Fast iteration / scripting**:
  - `RESOLUTION`: `32` or `64`
  - Lower displacement, lighter atmosphere
  - Highest responsiveness for authoring and parameter exploration.

### Benchmark smoke command

```bash
cargo run -p app -- --mod-source=mods/planet-generator --bench 5 --opt --skip-splash
```
