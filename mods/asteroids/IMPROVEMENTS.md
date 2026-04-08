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

### 3. 3-Layer Background Composition
Draw order (z-indexed):
- `z=0` — `stars-layer.yml`: 22 text-sprite star field (5 gold `*`, 17 dim `.`)
- `z=1` — `planets-layer.yml`: 3 closed vector-polygon planets/moon with palette bindings
- `z=2` — gameplay entities (spawned at runtime)
- `z=10` — `hud-grid.yml`: transparent corner HUD

All 3 palettes (`neon`, `classic`, `teal`) include `planet_body` and `planet_rim` keys.

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
- `scenes/game/layers/stars-layer.yml` — NEW: z=0 star field
- `scenes/game/layers/planets-layer.yml` — NEW: z=1 planet/moon background
- `scenes/game/scene.yml` — added stars + planets layer refs
- `palettes/neon.yml`, `classic.yml`, `teal.yml` — added `planet_body`, `planet_rim`
- `scenes/game/game-loop.rhai` — heart visibility control (unchanged, uses heart-1/2/3 ids)
- `scenes/pause/scene.yml` — pause menu scene
- `scenes/pause/pause.rhai` — pause menu navigation
- `scenes/title/scene.yml` — removed menu-options
- `scenes/title/title.rhai` — simplified menu selection

## Testing
All scenes pass validation (`--check-scenes`). Ready to play!
