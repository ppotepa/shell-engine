# Planet Generator Mod

Procedural planet viewer with a multi-tab parameter UI.

## Running

```bash
SHELL_QUEST_MOD_SOURCE=mods/planet-generator cargo run -p app
```

## Controls

| Key / Input | Action |
|-------------|--------|
| `1` / `2` / `3` / `4` (or mouse click) | Switch tab: Continents / Mountains / Climate / Visual |
| Slider drag | Adjust parameter value with mouse |
| `F1`–`F7` | Load preset: Earth / Mars / Ocean / Desert / Ice / Volcanic / Archipelago |
| `R` | Randomize all parameters |
| `Delete` | Reset to Earth defaults |
| `Ctrl+F` | Toggle orbit / free-look camera |

## Scene structure

- `scenes/main/scene.yml` — single scene, orbit-camera + free-look-camera
- `scenes/main/layers/planet.yml` — OBJ planet mesh (`world://32`)
- `scenes/main/layers/hud-tabs.yml` — tab bar (top-right)
- `scenes/main/layers/hud-panel.yml` — parameter panel background
- `scenes/main/layers/hud-sliders.yml` — flat absolute-positioned slider layer (7 multiplexed widgets)
- `scenes/main/layers/hud-tabs.yml` — tab bar (top-right)
- `scenes/main/layers/hud-actions.yml` — Randomize / Reset buttons
- `scenes/main/layers/hud-presets.yml` — preset name strip (bottom-right)
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
