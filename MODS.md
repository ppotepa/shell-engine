# Shell Engine Mod System

## Overview

Mods are self-contained content packages loaded by the engine at startup.
A mod can be an unpacked directory or a `.zip` archive. The engine selects
which mod to load via:

- `--mod-source` CLI flag, or
- `SHELL_ENGINE_MOD_SOURCE` environment variable.

If neither is set, the default mod (`mods/shell-engine`) is used.

## Asteroids (SDL2 Mod)

Asteroids is a fast-paced orbital dogfight game with a cinematic planet-follow
camera and a 3D cockpit planet backdrop. Gameplay is built on sphere-based
orbital navigation (geodesic transport, Rodrigues rotation).

### Structure

```
mods/asteroids/
+-- mod.yaml
+-- assets/
|   +-- 3d/                  Ship/sphere OBJ meshes plus Scene3D definitions
+-- catalogs/
|   +-- emitters.yaml        Thruster, impact, debris, and orbital particle presets
|   +-- input-profiles.yaml  Yaw/strafe/thrust control mappings
|   +-- prefabs.yaml         Ship, bullet, asteroid, shrapnel, debris configs
+-- objects/
+-- palettes/
+-- scenes/
|   +-- mainmenu/
|   +-- game/
|   |   +-- scene.yml
|   |   +-- game-loop.rhai
|   |   +-- layers/
|   |       +-- stars-layer.yml
|   |       +-- planet-bg-layer.yml
|   |       +-- game-canvas.yml
|   |       +-- planets-layer.yml
|   |       +-- solar-scene3d-layer.yml
|   |       +-- hud-grid.yml
|   +-- scripts/
|   |   +-- rcs.rhai
|   +-- highscores/
```

### Orbital Flight Model

- **Position**: Sphere normal `sn` (unit vector from planet center to ship)
- **Orientation**: Forward/right tangents `sf`, `sr` (local ship frame on sphere)
- **Velocity**: `vrad` (radial), `vfwd` (prograde in `sf`), `vright` (strafe in `sr`)
- **Rotation**: `yaw_rate` rotates `sf/sr` around `sn` axis
- **Translation**: Geodesic transport + live orbital radius (`radius`) under gravity (`gravity_mu`)
- **Atmosphere**: altitude bands add drag + heat; severe reentry and impact can destroy the ship
- **Telemetry**: orbital HUD exposes `ALT`, `TSPD`, `RSPD`, `HEAT`, and `VXY`

### Controls

| Input | Action |
|-------|--------|
| W/↑ | Prograde thrust (increase `vfwd`) |
| S/↓ | Retro-brake (decrease `vfwd`) |
| A/← | Yaw left (via RCS, rotates `sf/sr` CCW) |
| D/→ | Yaw right (via RCS, rotates `sf/sr` CW) |
| Q | Strafe left (lateral in `-sr` direction) |
| E | Strafe right (lateral in `+sr` direction) |
| SPACE | Fire bullet (inherits full tangential velocity) |
| ESC | Pause menu |

### Camera & VFX

- **Camera**: Cockpit-follow planet rendering with inertial gimbal lag (`τ=0.68s`);
  yaw-linked sway for banking feedback; instant normal tracking, delayed up-vector smoothing.
- **RCS**: 4-emitter system (main rear, bow fore, port/starboard sides); rotation couple
  with visual intensity scaling; auto-brake on yaw release; linear trim corrections.
- **Main Engine**: 3-phase profile driven by thrust hold/release timers:
  - Ignition: hot white/cyan burst (0–150ms)
  - Transition: mid cyan (90–460ms)
  - Sustain: full blue steady burn (260ms+)
  - Fade: cool tail after release

### Runtime Characteristics

- `mod.yaml` sets `display:` block with 640x360 authored render size and `fit` presentation policy; SDL2 is the only renderer backend.
- The active gameplay scene currently loads 6 layers: stars → planet → asteroid scene slots → ship scene slot → gameplay canvas → HUD.
- `planet-bg-layer.yml` renders cockpit planet (OBJ sphere + two cloud shells with biome shading + transparency).
- `game-canvas.yml` is a support layer; ship/asteroid visuals are now scene-space OBJ slots.
- `hud-grid.yml` displays transparent corner panels: SCORE, WAVE, LIVES (as retro pixel-art ♥), ESC hint.
- Flight simulation runs on a hybrid orbital state model (`radius + vrad + vfwd + vright`) with atmosphere drag/heat.
- Asteroids spawn at world edges; ship orbits freely within 3200×1800 world bounds.
- Orbital HUD telemetry shows altitude, tangential speed, radial speed, heat, and world-frame velocity components.
- `solar-scene3d-layer.yml` and `planets-layer.yml` are retained in the repo as additional background assets, but they are not wired into the current `scenes/game/scene.yml`.

### Running

```bash
cargo run -p app -- --mod-source=mods/asteroids
```

Validation:

```bash
cargo run -p app -- --mod-source=mods/asteroids --check-scenes
```

### Feel Parameters (game-loop.rhai)

Current hybrid tuning (first realism pass):

```rhai
YAW_RESPONSE = 7.2      // target yaw-rate convergence
YAW_DAMP = 2.6          // damping coefficient (faster settle)
YAW_MAX = 1.1           // rad/s — rotational speed cap
ACC_FWD = 0.9           // px/s² — prograde/retro accel
ACC_SIDE = 0.6          // px/s² — strafe accel
LIN_DAMP = 0.08         // linear damping (vacuum feel)
SIDE_TRIM = 0.20        // base side-slip damping
SIDE_THRUST_TRIM = 1.0  // extra side-slip trim under thrust
MAX_SPD = 28.0          // px/s — speed cap (~3x baseline orbit speed)
MAX_VRAD = 20.0         // px/s — radial speed cap
ATMO_DRAG_MAX = 2.2     // atmosphere drag ceiling
HEAT_DAMAGE_START = 0.72
HEAT_KILL_THRESHOLD = 0.97
```

## Shell Engine (Main Mod)

The primary game content. Contains all intro sequences, menus, gameplay
scenes, assets, and the C# sidecar for the simulated CognitOS terminal.

### Structure

```
mods/shell-engine/
+-- mod.yaml
+-- assets/
|   +-- images/
|   +-- fonts/
|   +-- 3d/
|   +-- audio/
|   +-- linus/
|   +-- raw/                  gitignored staging area
+-- objects/
+-- behaviors/
+-- scenes/
|   +-- 00-intro-logo/
|   +-- 01-intro-date/
|   +-- 02-intro-boot/
|   +-- 03-intro-lab-enter/
|   +-- 04-difficulty-select/
|   +-- 05-intro-cpu-on/
|   +-- 06-intro-login/
|   +-- 3d/
|   +-- mainmenu/
+-- os/cognitOS/              C# sidecar (simulated MinixOS)
+-- schemas/
+-- docs/
```

### Scene Flow

| Scene | ID                       | Effects                  | Notes                  |
|-------|--------------------------|--------------------------|------------------------|
| 00    | 00.intro.logo            | CRT-on, shine, flash     | Splash animation       |
| 01    | 01.intro.date            | Scanlines                | Static date display    |
| 02    | 02.intro.boot            | Fade-in, scanlines       | BIOS boot sequence     |
| 03    | 03.intro.lab-enter       | Fade-in/out              | Environment setup      |
| 04    | 04.difficulty-select     | 4x PostFX, 3D portraits | Menu with OBJ renders  |
| 05    | 05.intro.cpu-on          | Fade-in                  | CPU power-on sequence  |
### Special Features

- Prerender pass for 3D scenes (OBJ model rasterization).
- IPC bridge to C# sidecar (`os/cognitOS/`) for CognitOS simulation.

## Shell Engine Tests (Benchmark Mod)

Automated testing variant of the main mod. All user-input triggers are
replaced with timeouts so scenes advance without interaction.

Assets, behaviors, objects, and schemas are symlinked back to `mods/shell-engine/`.

### Running

```bash
cargo run -p app -- --mod-source=mods/shell-engine-tests --bench 10
```

### Timeline Per Loop

| Scene | Duration | Trigger        | Compression |
|-------|----------|----------------|-------------|
| 00    | ~1680ms  | timeout 600ms  | 4.2x        |
| 01    | ~1900ms  | timeout 400ms  | 5.1x        |
| 02    | ~2180ms  | timeout 200ms  | 5.6x        |
| 03    | ~1120ms  | timeout 200ms  | 2.8x        |
| 04    | ~2550ms  | timeout 2000ms | 2.0x        |
| Total | ~9430ms  |                | 3.9x        |

Scene 04 loops back to 00 for continuous benchmarking.

## Playground (Dev Mod)

Development sandbox with reference scenes for isolated feature testing.
Contains scenes for 3d-scene, rhai-lab, rhai-time, and many more.

### Running

```bash
SHELL_ENGINE_MOD_SOURCE=mods/playground cargo run -p app
```

Navigation: Esc returns to the playground menu (does not quit the app).
Use Ctrl+C for hard quit.

## Planet Generator (Procedural Planet Viewer)

Standalone procedural planet playground with a full-screen world view and a
tabbed HUD for tuning all generation parameters in real time. Uses the
`world://` URI pipeline through `engine-terrain`.

### Structure

```
mods/planet-generator/
+-- mod.yaml
+-- scenes/
    +-- main/
        +-- scene.yml
        +-- main.rhai
        +-- layers/
            +-- planet.yml          OBJ mesh (world://32)
            +-- hud-tabs.yml        Tab bar (top-right)
            +-- hud-panel.yml       4 parameter panels
            +-- hud-actions.yml     Randomize / Reset buttons
            +-- hud-presets.yml     Preset name strip (bottom-right)
            +-- hud-stats.yml       Live biome stats (bottom-left)
```

### Controls

| Key | Action |
|-----|--------|
| 1 / 2 / 3 / 4 | Switch tab: Continents / Mountains / Climate / Visual |
| F1–F7 | Load preset: Earth / Mars / Ocean / Desert / Ice / Volcanic / Archipelago |
| R | Randomize all parameters |
| Delete | Reset to Earth defaults |
| ↑ / ↓ | Cycle selected parameter |
| ← / → | Adjust selected value |
| Ctrl+F | Toggle orbit / free-look camera |

### Parameters (4 tabs)

- **Continents**: seed, ocean %, continent size, coast chaos, octaves
- **Mountains**: mountain spacing, height, ridge detail
- **Climate**: moisture scale, ice caps, altitude cooling, rain shadow
- **Visual**: resolution (16–128), displacement, coloring mode, rotation, sun azimuth/elevation, ambient

### Running

```bash
cargo run -p app -- --mod-source=mods/planet-generator --sdl-window-ratio=16:9 --sdl-pixel-scale=2
```

### Runtime Characteristics

- `mod.yaml` sets 640×360 authored render size with `fit` presentation policy.
- World parameter updates are throttled (500ms) to avoid blocking the render
  thread with synchronous mesh rebuilds. Visual-only params (rotation, lighting)
  update every frame.
- Live stats bar shows ocean/forest/desert/snow/mountain coverage via
  `planet_last_stats()` Rhai function.

## GUI Playground (Widget Test-Bench)

Interactive test-bench for the `engine-gui` widget system. Exercises all four
control types (`Slider`, `Toggle`, `Button`, `Panel`) in an RGB color mixer
dashboard with real-time visual feedback.

### Structure

```
mods/gui-playground/
+-- mod.yaml
+-- scenes/
    +-- main/
        +-- scene.yml          gui: block with sliders/toggles/buttons
        +-- main.rhai          Reactive wiring script
        +-- layers/
            +-- bg.yml         Background panels
            +-- controls.yml   Tracks, handles, fill bars, labels
            +-- output.yml     Swatch, hex/RGB readout, state, log
```

### Features

- 3 sliders (R/G/B, 0–255) with engine-level handle positioning and fill tracks
- 3 toggles (Show Hex, Show Swatch, Invert Colors)
- 2 buttons (Reset All, Randomize) using `gui.set_widget_value()`
- 5-row color swatch, hex readout, state monitor, event log
- Persistent state via `local` (click count survives across frames)

### Running

```bash
cargo run -p app -- --mod-source=mods/gui-playground
```

## Creating a Custom Mod

### Minimum Structure

```
mods/my-mod/
+-- mod.yaml
+-- scenes/
    +-- hello/
        +-- scene.yml
        +-- layers/
            +-- main.yml
```

### mod.yaml

```yaml
name: my-mod
version: "0.1.0"
description: "My custom mod"
entrypoint: /scenes/hello/scene.yml
display:
  min_colours: 256
  min_width: 120
  min_height: 30
  render_size: 120x30
  presentation_policy: stretch
```

Use `render_size` for the authored in-memory canvas and `presentation_policy`
for how that canvas is shown on the real terminal/window:

- `stretch` fills the available output area,
- `fit` preserves aspect ratio with letterboxing,
- `strict` keeps 1:1 cells and centers/crops if needed.

### Running

```bash
cargo run -p app -- --mod-source=mods/my-mod
```

## Mod Asset Loading

- Paths use a leading `/`, resolved relative to the mod root.
- The same paths work for both directory and zip-packaged mods.
- `assets/raw/` is a gitignored staging area for work-in-progress assets.
- Named mod behaviors are loaded from top-level `behaviors/*.yml`; those YAML wrappers may point at external Rhai via `src`.

### Path Resolution Example

```
/assets/images/logo.png  -->  mods/my-mod/assets/images/logo.png  (directory)
/assets/images/logo.png  -->  my-mod.zip!/assets/images/logo.png  (zip archive)
```
