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

### 2. Pause Menu (NEW)
- **Location**: scenes/pause/
- **Trigger**: ESC key during gameplay
- **Options**: Resume / Return to Title / Quit
- **Navigation**: Arrow keys + ENTER, ESC to resume

### 3. Game-Over Improvements
- Separate overlay layer (no longer mixed with HUD)
- Shows final score
- Clear actions: SPACE to restart, ESC to menu

### 4. Title Menu Cleanup
- Removed redundant menu-options declaration
- Simplified Rhai navigation logic
- Cleaner palette display in hint text

## Files Changed
- scenes/game/layers/hud.yml — Reorganized HUD layout
- scenes/game/game-loop.rhai — Added pause/ESC handling
- scenes/pause/scene.yml — New pause menu scene
- scenes/pause/pause.rhai — Pause menu navigation
- scenes/title/scene.yml — Removed menu-options
- scenes/title/title.rhai — Simplified menu selection

## Testing
All scenes pass validation (--check-scenes). Ready to play!
