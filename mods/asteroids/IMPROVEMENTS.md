# Asteroids Mod Improvements

## Changes Made

### 1. HUD Redesign
- **Before**: Cramped top-left layout, overlapping game-over text
- **After**: Corner-anchored layout
  - Top-left: SCORE
  - Top-right: WAVE
  - Bottom-left: LIVES
  - Bottom-right: ESC hint
  - Game-over: Centered overlay on separate layer (z:100)

### 2. Transparent HUD Panels
- HUD corner panels no longer render a dark background box
- `bg_colour` omitted → engine uses `Color::Reset` → lower layers show through
- Only the game-over overlay retains an explicit `bg_colour` (intentional dimming)

### 3. Unified Scene3D Solar Background
Draw order (z-indexed):
- `z=-30` — `solar-scene3d-layer.yml`: one `scene3_d` sprite with full system
  composition (nebula, sun, planets, saturn-style ring disk, visible belt rocks)
- `z=0+` — gameplay entities (spawned at runtime)
- `z=10` — `hud-grid.yml`: transparent corner HUD

Background motion model:
- `solar-system.scene3d.yml` provides a `solar-orbit` clip (`24s`, `24` keyframes)
- Rhai selects `solar-orbit-${n}` per frame and applies a tiny camera-relative
  drift for depth, while keeping the entire background as one render target

### 4. Retro Pixel-Art Life Icons
- Replaced smooth vector-polygon hearts with `generic:3` pixel-art `♥` glyphs
- Each heart: `font: "generic:3"`, `scale-x: 2.0`, `scale-y: 2.0` → 24×28 px
- Properly centred in 154×50 lives panel (padding 6 → inner 142×38): y=11, x=23/64/105
- IDs `heart-1/2/3` preserved — Rhai visibility control unchanged

### 5. Pause Menu (NEW)
- **Location**: scenes/pause/
- **Trigger**: ESC key during gameplay
- **Options**: Resume / Return to Title / Quit
- **Navigation**: Arrow keys + ENTER, ESC to resume

### 6. Game-Over Improvements
- Separate overlay layer (no longer mixed with HUD)
- Shows final score
- Clear actions: SPACE to restart, ESC to menu

### 7. Title Menu Cleanup
- Removed redundant menu-options declaration
- Simplified Rhai navigation logic
- Cleaner palette display in hint text

## Files Changed
- `scenes/game/layers/hud-grid.yml` — transparent panels, retro pixel-art hearts
- `assets/3d/solar-system.scene3d.yml` — NEW: full solar-system prefab scene
- `assets/3d/saturn_ring_back.obj`, `assets/3d/saturn_ring_front.obj` — NEW: ring disk meshes
- `scenes/game/layers/solar-scene3d-layer.yml` — NEW: one background `scene3_d` sprite
- `scenes/game/scene.yml` — switched game background to unified `scene3_d` layer
- `palettes/neon.yml`, `classic.yml`, `teal.yml` — added `planet_body`, `planet_rim`
- `scenes/game/game-loop.rhai` — scene3d frame selection + camera-relative drift
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
