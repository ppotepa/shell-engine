# Asteroids Mod Improvements

## Changes Made

### 1. Orbital Flight Control (LATEST)
- **Orbital Model**: Sphere-based navigation — `sn` (position normal), `sf/sr` (forward/right tangents)
- **Translation**: Geodesic transport via Rodrigues rotation around cross(sn, velocity)
- **Rotation**: Yaw via RCS couples; independent from heading physics
- **Input Mapping**: `turn_left/right` (yaw), `strafe_left/right` (lateral), `thrust/brake` (prograde/retro)
- **Camera**: Inertial gimbal lag (`τ=0.68s` up-smoothing); sway from yaw rate; instant normal tracking
- **VFX**: 4-emitter RCS (main/bow/port/starboard); 3-phase main engine (ignition→sustain→fade)
- **Orbit Period**: 5 minutes (ORBIT_V0 = 2πR/T = 9.42 px/s @ R=450)
- **Feel Tuning**: Snappier yaw (YAW_ACCEL 1.8→1.95), heavier gimbal (CAM_UP_TAU 0.58→0.68), banking cues (CAM_SWAY_GAIN 0.24→0.28)
- **Bug Fixes**: dt cap removed; camera 1-frame lag fixed; bullet carries full tangential velocity
- **Files**: `game-loop.rhai` (~1000 LOC orbital state machine), `scripts/rcs.rhai` (~245 LOC VFX pipeline)

### 2. HUD Redesign
- **Before**: Cramped top-left layout, overlapping game-over text
- **After**: Corner-anchored layout
  - Top-left: SCORE
  - Top-right: WAVE
  - Bottom-left: LIVES
  - Bottom-right: ESC hint
  - Game-over: Centered overlay on separate layer (z:100)

### 3. Transparent HUD Panels
- HUD corner panels no longer render a dark background box
- `bg_colour` omitted → engine uses `Color::Reset` → lower layers show through
- Only the game-over overlay retains an explicit `bg_colour` (intentional dimming)

### 4. Scene3D Solar Background Prototype
The repository still contains an experimental large-scale solar background:
- `solar-scene3d-layer.yml`: one `scene3_d` sprite intended to render a distant
  nebula / sun / planet composition
- `solar-system.scene3d.yml`: `solar-orbit` clip (`24s`) with tween-driven live motion

Current status:
- these assets remain in the repo as prototype/reference content
- the active `scenes/game/scene.yml` does **not** currently wire this layer into gameplay
- the live Asteroids scene uses stars → planet OBJ layer → gameplay → HUD

### 5. Retro Pixel-Art Life Icons
- Replaced smooth vector-polygon hearts with `generic:3` pixel-art `♥` glyphs
- Each heart: `font: "generic:3"`, `scale-x: 2.0`, `scale-y: 2.0` → 24×28 px
- Properly centred in 154×50 lives panel (padding 6 → inner 142×38): y=11, x=23/64/105
- IDs `heart-1/2/3` preserved — Rhai visibility control unchanged

### 6. Pause Menu (NEW)
- **Location**: scenes/pause/
- **Trigger**: ESC key during gameplay
- **Options**: Resume / Return to Title / Quit
- **Navigation**: Arrow keys + ENTER, ESC to resume

### 7. Game-Over Improvements
- Separate overlay layer (no longer mixed with HUD)
- Shows final score
- Clear actions: SPACE to restart, ESC to menu

### 8. Title Menu Cleanup
- Removed redundant menu-options declaration
- Simplified Rhai navigation logic
- Cleaner palette display in hint text

## Files Changed

### Orbital Control (Latest)
- `scenes/game/game-loop.rhai` — orbital flight model, state machine, RCS dispatcher, feel constants
- `scripts/rcs.rhai` — 4-emitter RCS pipeline, 3-phase main engine, rotation couple logic
- `catalogs/input-profiles.yaml` — yaw/strafe/thrust action mappings
- `catalogs/emitters.yaml` — orbital ship emitters (main/bow/port/starboard) with gravity config
- `scenes/game/layers/planet-bg-layer.yml` — planet OBJ layer with biome/cloud shading
- `scenes/game/scene.yml` — active layer wiring (stars, planet, gameplay, HUD)

### Previous Changes
- `scenes/game/layers/hud-grid.yml` — transparent panels, retro pixel-art hearts
- `assets/3d/solar-system.scene3d.yml` — full solar-system prefab scene
- `scenes/game/layers/solar-scene3d-layer.yml` — prototype background `scene3_d` sprite kept in repo
- `palettes/neon.yml`, `classic.yml`, `teal.yml` — added `planet_body`, `planet_rim`
- `scenes/pause/scene.yml` — pause menu scene
- `scenes/pause/pause.rhai` — pause menu navigation
- `scenes/title/scene.yml` — removed menu-options
- `scenes/title/title.rhai` — simplified menu selection

## Testing
All scenes pass validation (`--check-scenes`). Ready to play!

## Notes After Flicker Investigation

- The top gameplay layer flicker was not a z-index bug. It came from runtime
  clone target resolution and immediate-mode visual state resets.
- Free camera movement made the failure much more visible because any missed
  visual-sync write dropped the gameplay layer back to `(0,0)` before camera
  subtraction.
- The engine/runtime docs now describe the relevant contract:
  runtime clone target aliases belong to the parent layer, and camera/parallax
  state must be written every frame.
