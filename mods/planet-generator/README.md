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
| `F10` or `Fly Around` button | Toggle `generator` / `flight` mode |
| `Esc` | Exit `flight` mode and return to `generator` mode |
| `F9` / `C` (flight mode) | Toggle flight profile: `arcade` / `sim-lite` |
| `H` / `J` (flight mode) | Toggle assists: `HOLD ALT` / `HOLD HDG` |
| Flight mode controls | `Up/Down` or `W/S` tangent thrust, `Left/Right` or `A/D` yaw, `Q/E` radial climb/descent, `X` boost, `Z` brake |

## Runtime modes

- `generator` mode is the default authoring/view mode; sliders, tabs, presets, randomize, and reset stay active.
- `flight` mode is toggled by `F10` or `Fly Around`; the script syncs generated body runtime data, ensures one ship entity, and drives camera basis from ship-relative motion.
- `flight profile` can be toggled with `F9` or `C` while flying:
  - `arcade` keeps stronger response and easier correction.
  - `sim-lite` keeps smoother input response and gentler stabilization.
- `flight assists` are active in runtime and reflected in HUD:
  - `HOLD ALT`
  - `HOLD HDG`
- `flight profile` and both assist toggles are persisted in game state and restored on next run, so the next launch resumes the last flight setup.
- `F10` or `Esc` returns to `generator` mode without resetting current generator parameters.

## Scene structure

- `scenes/main/scene.yml` — single scene, orbit-camera + free-look-camera (surface mode enabled)
- `scenes/main/layers/planet.yml` — OBJ planet mesh (`world://32`)
- `scenes/main/layers/hud-tabs.yml` — tab bar (top-right, authored as `type: tabs`)
- `scenes/main/layers/hud-models.yml` — model selector row (Planet/Sphere/Cube/Suzanne, authored as `type: segmented-control`)
- `scenes/main/layers/hud-panel.yml` — right-side parameter rail background in native `1280x720` UI space
- `scenes/main/layers/hud-sliders.yml` — compact slider layer with active-tab header, summary, widened tracks, and right-aligned live values
- `scenes/main/layers/hud-actions.yml` — Randomize / Fly Around / Reset + flight hints
- `scenes/main/layers/hud-presets.yml` — compact preset dropdown + popup list
- `scenes/main/layers/hud-stats.yml` — live stats strip (bottom-left)
- `scenes/main/main.rhai` — tab switching, mouse-drag slider input, preset loading, world param push with debounce

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
