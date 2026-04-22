# Planet Generator Mod

Procedural planet viewer with a compact modernized multi-tab tool panel and separated live value readouts.

The mod renders the world pass at `640x360`, then composites HUD/UI at native
`1280x720`. The tool panel is authored directly in `1280x720` UI space so the
controls stay denser and more readable than the underlying 3D scene.

Within the refactor rollout this mod is the heavier celestial canary: it
exercises generated-body authoring, shared scene-camera paths, and prefab merge
behavior in one authored package.

## Canary fixtures

`catalogs/prefabs.yaml` contains the prefab merge canary pair:

- `canary-probe-base` - base nested-component fixture
- `canary-probe-nested` - deep-override fixture that extends the base probe

These entries stay small on purpose so prefab merge behavior can be checked
without pulling in the full generator scene graph.

`catalogs/presets.yaml` declares the scene-level controller preset ids used by
the generator, flight, and cockpit scenes.

## Running

```bash
SHELL_ENGINE_MOD_SOURCE=mods/planet-generator cargo run -p app
```

Flight scene:

```bash
cargo run -p app -- --mod-source=mods/planet-generator --start-scene=/scenes/flight/scene.yml
```

Cockpit simulator scene:

```bash
cargo run -p app -- --mod-source=mods/planet-generator --start-scene=/scenes/3d-cockpitview/scene.yml
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
| `F10` | Enter the flight scene using the currently generated planet |
| `F11` | Enter the cockpit simulator scene using the currently generated planet |
| `Ctrl+F` | Toggle orbit / free-look camera (WASD move, Q/E altitude) |

### Flight Scene Controls

| Key / Input | Action |
|-------------|--------|
| Mouse | Steer the ship; thrust always follows the current look direction |
| `W` / `S` or `Up` / `Down` | Forward / reverse thrust |
| `A` / `D` or `Left` / `Right` | Strafe left / right in ship-local space |
| `Q` / `E` | Vertical thrust down / up |
| `Space` | Extra upward thrust |
| `F` | Toggle cockpit mesh on/off while keeping the same flight model |

## Runtime modes

- `generator` mode is the default authoring/view mode; sliders, tabs, presets, randomize, and reset stay active.
- `scripts/generator/render_push.rhai` is now the single runtime apply path for the generated planet. It pushes body, generator, material, light, and atmosphere changes through `world.apply_planet_spec(...)`, so scene entrypoints no longer own separate `body_sync` mutations.
- Free-look surface mode follows the scene `focus-body` render shell from the
  runtime body patch. The scene authors that path explicitly with
  `camera-rig.surface.mode: locked`, which lowers through the current
  scene-camera compatibility seam. In this mod the
  patched body keeps `radius_px = 1.0` for the preview shell while
  `surface_radius` stays in simulation world units.

Validation:

```bash
cargo run -p app -- --mod-source=mods/planet-generator --check-scenes
```

## Scene structure

- `scenes/main/scene.yml` — single scene, orbit/free-look camera controls authored through `camera-rig`, with an explicit surface-locked free-look contract on top of `render-space: 3d` + `world-model: celestial-3d`
- `scenes/flight/scene.yml` — first-person flight scene with generated planet, hidden proxy ship, and camera-anchored cockpit, authored as `world-model: celestial-3d`; the planet and cockpit meshes opt into the shared scene camera with `camera-source: scene`
- `scenes/3d-cockpitview/scene.yml` — static cockpit simulator scene with a camera-anchored procedural cockpit mesh and the generated planet outside, authored as `world-model: celestial-3d`; the cockpit and planet shells also use `camera-source: scene`

These scenes use the celestial runtime contract directly. Generator, flight,
and cockpit handoff logic stays isolated in their scene scripts so the scene
contracts themselves remain explicit and readable.

Current authored contract per scene:

- `scenes/main/scene.yml` — `world-model: celestial-3d`, `camera-preset: celestial-orbit-inspector`, `player-preset: generator-inspector`, `ui-preset: generator-hud`, plus an authored `camera-rig` that lowers orbit/free-look controls into the current runtime shape
- `scenes/flight/scene.yml` — `world-model: celestial-3d`,
  `camera-preset: cockpit-flight`, `player-preset: celestial-free-flight`,
  `ui-preset: flight-hud`, plus an authored `camera-rig` entry for the cockpit
  flight controller
- `scenes/3d-cockpitview/scene.yml` — `world-model: celestial-3d`,
  `camera-preset: cockpit-view`, `player-preset: cockpit-proxy`,
  `ui-preset: cockpit-hud`, plus an authored `camera-rig` entry for the fixed
  cockpit-view controller
- `scenes/main/layers/planet.yml` — OBJ planet mesh (`world://32`)
- `scenes/flight/layers/planet.yml` — shared procedural planet pass driven by the same generator render push
- `scenes/flight/layers/player.yml` — invisible ship proxy + camera-anchored 3D cockpit overlay
- `scenes/flight/layers/hud.yml` — minimal camera/status/controls HUD
- `scenes/3d-cockpitview/layers/cockpit.yml` — procedural `cockpit://` mesh attached to the shared scene camera
- `scenes/3d-cockpitview/layers/hud.yml` — cockpit simulator HUD with quick handoff/back controls
- `scenes/main/layers/hud-tabs.yml` — tab bar (top-right, authored as `type: tabs`)
- `scenes/main/layers/hud-models.yml` — model selector row (Planet/Sphere/Cube/Suzanne, authored as `type: segmented-control`)
- `scenes/main/layers/hud-panel.yml` — right-side parameter rail background in native `1280x720` UI space
- `scenes/main/layers/hud-sliders.yml` — compact slider layer with active-tab header, summary, widened tracks, and right-aligned live values
- `scenes/main/layers/hud-actions.yml` — Randomize / Vehicle / Reset + compact bounded handoff/profile/assist status row
- `scenes/main/layers/hud-presets.yml` — compact preset dropdown + popup list
- `scenes/main/layers/hud-stats.yml` — compact bounded telemetry strip (bottom-left)
- `scenes/main/main.rhai` — thin scene orchestrator; imports the generator modules below
- `scripts/std/math.rhai` — shared numeric helpers used by generator render throttling
- `scripts/std/runtime_scene.rhai` — scene object text/fg/visible helpers over `runtime.scene.objects.find(...)`
- `scripts/generator/state.rhai` — generator bootstrap from `planet_spec` defaults plus UI/render-push flags
- `scripts/generator/input.rhai` — tab/model/actions/preset input flow
- `scripts/generator/presets.rhai` — preset data and preset/randomize/reset application
- `scripts/generator/params.rhai` — parameter schema, normalize/denormalize, per-tab labels and values
- `scripts/generator/gui_sync.rhai` — local state <-> slider widget sync
- `scripts/generator/body_sync.rhai` — generated body builder/change detector used by the shared render/apply path
- `scripts/generator/hud.rhai` — tab/model/action highlights, parameter labels, telemetry strip
- `scripts/generator/render_push.rhai` — shared generated-planet apply path; pushes body, generator, visual, light, and atmosphere updates through `world.apply_planet_spec(...)`; `local.render_push_throttle_ms` controls generator push throttling and now defaults to realtime (`0.0`)
- `scripts/std/math3.rhai` — shared 3D angle/orientation helpers used by flight logic
- `scripts/std/flight_handoff.rhai` — persists the current generator planet and hands it off into the flight scene on `F10`
- `scripts/flight/state.rhai` — flight-scene bootstrap and fixed Earth-like defaults
- `scripts/flight/spawn.rhai` — planet surface spawn finder; resolves biome-preferred spawn normals and displaced surface radius
- `scripts/flight/flight.rhai` — script-driven inertial free-flight model; mouse steers orientation, `W/S/A/D/Q/E` thrust in view space, and the cockpit mesh stays glued to the camera
- `scripts/flight/hud.rhai` — minimal flight-scene HUD writes
- `scripts/cockpit/state.rhai` — cockpit-simulator bootstrap and launch snapshot restore
- `scripts/cockpit/view.rhai` — camera head-look, cockpit mesh anchoring, and scene handoff controls
- `scripts/cockpit/hud.rhai` — cockpit simulator HUD writes

## Parameters

### Continents tab
- **SEED** — world generation seed (0–9999)
- **OCEAN** — toggle ocean rendering on/off; lowlands still keep the same terrain shaping when disabled
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
| F3 | Ocean | Ocean enabled, tropical moisture |
| F4 | Desert | Ocean disabled, minimal rainfall |
| F5 | Ice | Strong polar caps, cold lapse rate |
| F6 | Volcanic | Extreme terrain displacement, high ridges |
| F7 | Archipelago | Ocean enabled, island-chain terrain bias |

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
