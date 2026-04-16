# Asteroids Mod — Project Status

> **Snapshot date**: 2026-04-12
> **Phase**: Surface-launch flight sim foundation (post-cleanup)
> **Last verified build**: `cargo build -p app` ✅ | `--check-scenes` ✅

---

## What This Project Is Now

A **surface-launch orbital flight simulator** built on top of the shell-engine engine.
Ship spawns on a procedurally-shaded Earth, lifts off with SPACE, orbits using realistic
Keplerian mechanics, and returns. No combat, no scoring, no game-over — just flight.

### Core Architecture
- **3D rendering**: Two composited layers — planet (Gouraud-shaded unit sphere) + ship (OBJ mesh)
- **Camera**: Altitude-adaptive 3D chase camera via `world.set_camera_3d_look_at()`
- **Physics**: Sphere-based orbital model — `sn/sf/sr` basis vectors, geodesic transport, Rodrigues rotation
- **Scale contract**: v_orbit = 7 km/s, T = 60s at surface, R = 66.85 km, gravity_mu = 87861 px³/s²

### Controls
| Key | Action |
|-----|--------|
| E | Main engine (forward thrust) |
| SPACE | Bottom engine (radial lift) |
| W/S | Prograde thrust / retro-brake |
| A/D | Yaw left / right (RCS) |
| Q | Lateral strafe left |

---

## File Inventory

### ✅ Active & Clean
| File | Purpose | Status |
|------|---------|--------|
| `mod.yaml` | Mod manifest, SDL2 output, 640×360 | ✅ Clean |
| `scenes/game/scene.yml` | Scene def: 1 object (ship), 4 layers | ✅ Clean |
| `scenes/game/game-loop.rhai` | ~490 LOC orbital state machine | ⚠️ Needs review (see below) |
| `scenes/game/layers/planet-bg-layer.yml` | 3D planet layer | ✅ Clean |
| `scenes/game/layers/ship-scene-layer.yml` | 3D ship mesh layer | ✅ Clean |
| `scenes/game/layers/hud-grid.yml` | HUD layout | ⚠️ Has stale elements |
| `scenes/mainmenu/scene.yml` | Main menu scene | ✅ Clean |
| `scenes/mainmenu/mainmenu.rhai` | Menu navigation (2 items) | ✅ Clean |
| `scenes/mainmenu/layers/main-grid.yml` | Menu layout | ✅ Clean |
| `objects/ship.yml` | Ship object (sprites: [] — mesh is on layer) | ✅ Clean |
| `catalogs/prefabs.yaml` | Ship prefab only | ✅ Clean |
| `catalogs/input-profiles.yaml` | Input bindings | ✅ Clean |
| `catalogs/celestial/bodies.yaml` | Earth definition + scale contract | ✅ Clean |
| `catalogs/celestial/planets.yaml` | Planet visual presets (earth_like, rocky_moon) | ✅ Clean |
| `catalogs/celestial/systems.yaml` | System definition | ✅ Clean |
| `catalogs/celestial/regions.yaml` | Region definition | ✅ Clean |
| `assets/3d/sphere.obj` | Planet mesh (3122 verts, 6240 faces) | ✅ Just regenerated |
| `assets/3d/ship.obj` | Ship mesh | ✅ Clean |
| `palettes/teal.yml` | Default palette | ⚠️ Has stale keys |
| `palettes/classic.yml` | Classic palette | ⚠️ Has stale keys |
| `palettes/neon.yml` | Neon palette | ⚠️ Has stale keys |

### ⚠️ Kept For Future Use (needs cleanup when reintroduced)
| File | Purpose | What's Stale |
|------|---------|--------------|
| `scripts/rcs.rhai` | RCS VFX pipeline (~257 LOC) | References emitters that exist; code is fine. **Keep.** |
| `catalogs/emitters.yaml` | 8 ship thruster emitters | `gravity_center_x/y: 1600/900` hardcoded — should be dynamic. **Keep.** |

### ❌ Should Be Deleted (dead / orphaned)
| File | Why Delete |
|------|------------|
| `scenes/game/layers/stars-layer.yml` | Duplicate of `stars-layer-1.yml` — both have identical stars, `stars-layer` is at z:-55, `stars-layer-1` at z:-56. Scene only references `stars-layer-1`. **Dead file.** |
| `assets/3d/solar-system.scene3d.yml` | Experimental solar system Scene3D asset (~30KB). No layer references it. Prototype from old phase. |
| `assets/3d/rock.obj` | Asteroid rock mesh. No asteroids exist anymore. |
| `IMPROVEMENTS.md` | Old changelog from combat-era. Describes waves, scoring, pause menu, highscores, solar prototype — all deleted. Completely outdated. |
| `saves/` | Empty directory. No save system implemented. |

### ⚠️ Needs Cleanup (stale content in active files)
| File | Issue |
|------|-------|
| **`audio/sfx.yaml`** | References `fire`, `explosion_small`, `explosion_large` — combat SFX that no longer exist. Only `thrust` is potentially useful. |
| **`audio/synth/asteroids.yml`** | Synth definitions for `fire`, `explosion_small`, `explosion_large` — combat sounds. Only `thrust` survives. |
| **`palettes/*.yml`** | Keys `asteroid`, `bullet`, `debris`, `fragment` (particle), `bullet_trail` (particle) — all reference deleted gameplay. Should be removed or repurposed. |
| **`hud-grid.yml`** | **SCORE** panel (hud-label-score, hud-score) — no scoring system. **WAVE** panel (hud-label-wave, hud-wave) — no wave system. **LIVES** panel (heart-1/2/3) — no lives system. **GAME OVER** overlay — no game-over state. Only the orbital telemetry bar is active. |
| **`catalogs/celestial/sites.yaml`** | `orbit-altitude-km: 20000.0` — wildly out of scale (surface is 66.85km radius). Probably meant 200km? |
| **`catalogs/emitters.yaml`** | `gravity_center_x/y: 1600/900` hardcoded to world center. Works because planet IS at world center, but fragile. |
| **`game-loop.rhai`** | Title comment says "Asteroids — Surface Launch Simulation" but scene title in `scene.yml` is still "Asteroids Game". Minor. |
| **`mainmenu`** | Title says "ASTEROIDS" — may want to rename to match new direction. |

---

## What Was Deleted This Session

### Files Removed
- `scenes/game/layers/stars-layer-2.yml` through `stars-layer-5.yml` (4 parallax star layers)
- `scenes/game/layers/asteroids-gen-layer.yml` (asteroid spawning grid)
- `scenes/game/layers/game-canvas.yml` (gameplay canvas)
- `scenes/game/layers/planets-layer.yml` (old planet rendering)
- `scenes/game/layers/solar-scene3d-layer.yml` (solar system 3D prototype)
- `objects/bullet.yml`, `objects/shrapnel.yml`, `objects/debris.yml`
- `objects/asteroid-large.yml`, `objects/asteroid-medium.yml`, `objects/asteroid-small.yml`
- `scenes/highscores/` (entire directory)

### Code Gutted
- `game-loop.rhai`: 1442 → ~490 lines. Removed: asteroids, waves, scoring, collision, shooting, respawn, highscores, game-over state machine
- `prefabs.yaml`: 6 → 1 prefab. Removed: asteroid-large/medium/small, bullet, shrapnel
- `emitters.yaml`: Removed all combat/asteroid/bullet/laser/destruction emitters
- `mainmenu.rhai`: Removed HIGH SCORES option (3→2 items)

### Engine Changes (prior sessions, still active)
- Camera zoom pipeline (`SetCameraZoom` command through engine-api → engine-scene-runtime → engine-behavior → engine-compositor → engine)
- OBJ world translation support (`obj.world.x/y/z` scene.set paths)

---

## Recommended Cleanup Actions

### 1. Delete Dead Files
```
DELETE  scenes/game/layers/stars-layer.yml     (orphan duplicate)
DELETE  assets/3d/solar-system.scene3d.yml     (unused prototype)
DELETE  assets/3d/rock.obj                     (no asteroids)
DELETE  IMPROVEMENTS.md                        (completely outdated)
DELETE  saves/                                 (empty)
```

### 2. Strip Combat Audio
In `audio/sfx.yaml`: remove `fire`, `explosion_small`, `explosion_large` events.
In `audio/synth/asteroids.yml`: remove `fire`, `explosion_small`, `explosion_large` sounds.
Keep `thrust` — it's relevant for engine VFX.

### 3. Clean Palettes
Remove from all 3 palette files: `asteroid`, `bullet`, `debris` color keys and `fragment`, `bullet_trail` particle ramps. These reference deleted content.

### 4. Gut the HUD
In `hud-grid.yml`:
- Remove SCORE panel (top-left)
- Remove WAVE panel (top-right)
- Remove LIVES panel (bottom-left hearts)
- Remove GAME OVER overlay layer entirely
- Keep: orbital telemetry (bottom-center), ESC hint (bottom-right)

### 5. Fix Celestial Sites
In `catalogs/celestial/sites.yaml`: change `orbit-altitude-km` from `20000.0` to something sane like `200.0`.

### 6. Minor Naming
- `scene.yml` title: "Asteroids Game" → "Flight Sim" or similar
- Main menu title: "ASTEROIDS" → TBD (user decision)

---

## What To Keep For Future Development

| Asset | Why Keep |
|-------|----------|
| `scripts/rcs.rhai` | Full RCS VFX pipeline — works with current emitters. Will be needed for visible thruster effects. |
| `catalogs/emitters.yaml` (8 ship emitters) | Ship thruster particle system. Gravity parameters may need tuning but the structure is solid. |
| `catalogs/celestial/planets.yaml` (rocky_moon) | Second planet type — useful when adding Moon or other bodies. |
| `palettes/*.yml` (ship/HUD/particle colors) | Ship, HUD, and thruster particle colors are all active. |
| `assets/fonts/*` | Orbitron + Space Mono — used by HUD and menus. |
| `input-profiles.yaml` | Current control scheme with all bindings. |
| `thrust` synth sound | Engine rumble SFX. |

---

## Technical Debt / Known Issues

1. **Emitter gravity center hardcoded**: `gravity_center_x/y: 1600/900` in emitters.yaml matches planet center but isn't dynamic.
2. **2D camera still present**: `world.set_camera()` + `world.set_camera_zoom()` drive a 2D viewport that isn't visually meaningful in 3D-only mode. May cause subtle compositor issues.
3. **Ship prefab has physics components**: Collider radius 10, mass 8, gravity_mode point — these are for the 2D physics engine. The actual orbital mechanics are all in Rhai. The prefab physics is only used for `world.set_physics()` position driving.
4. **No surface co-rotation**: Ship doesn't spin with the planet when grounded. It sits at a fixed point while Earth rotates under it.
5. ~~**Terminal rendering path**~~: resolved — `mod.yaml` now uses the `display:` block; `output: sdl2` and all terminal `min_*` settings removed during SDL2-only cleanup.
