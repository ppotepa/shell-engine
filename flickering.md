# Flickering Root-Cause Analysis & Fixes

## Executive Summary

The Asteroids mod exhibited persistent visual flickering/shimmer affecting the ship, particles, and star layers — everything **except** planets. After exhaustive static analysis of the full rendering pipeline (Rhai → BehaviorCommand → compositor → SDL2 presentation), five independent root causes were identified and fixed. All stem from the same fundamental design property: the engine uses **immediate-mode visual state** — every frame, all object states are zeroed and must be re-declared.

---

## Engine Background: Immediate-Mode Visual State

The engine calls `reset_frame_state()` ([engine-scene-runtime/src/ui_focus.rs:107-112](engine-scene-runtime/src/ui_focus.rs)) at the start of every frame's behavior system ([engine/src/systems/behavior.rs:69](engine/src/systems/behavior.rs)). This wipes **every** object's runtime state to defaults:

```rust
pub fn reset_frame_state(&mut self) {
    for state in self.object_states.values_mut() {
        *state = ObjectRuntimeState::default(); // offset_x=0, offset_y=0, visible=true, heading=0.0
    }
    self.ui_state.sidecar_io = SidecarIoFrameState::default();
}
```

After the reset, the frame proceeds through:

1. **behavior_system** — runs Rhai scripts; scripts re-declare layer offsets, obj properties, camera
2. **visual_sync_system** — pushes Transform2D (physics) positions → object_state offset_x/offset_y/heading
3. **compositor_system** — takes a snapshot of object_states, renders all layers onto a back buffer
4. **renderer_system** — diffs back vs front buffer, sends patches to SDL2

Properties NOT reset each frame: `camera_x/camera_y` (persisted on SceneRuntime), OBJ sprite properties (`roll_deg`, `pitch_deg`, etc. — stored on the Sprite struct, not in ObjectRuntimeState), and the scene layer/sprite tree itself.

**Implication**: If any code path that sets a visual property (offset, visibility, etc.) is gated behind a conditional that evaluates to false on some frames, that property silently reverts to `ObjectRuntimeState::default()` for those frames, causing a visible jump.

---

## Per-Frame System Execution Order

```
game_loop_tick:
  ├─ animator_system              # timeline progression
  ├─ gameplay_system              # inline physics (non-particle)
  ├─ particle_physics::start_async  # particle physics on rayon (concurrent)
  ├─ collision_system             # detect + resolve
  ├─ behavior_system              # ◄── reset_frame_state() then Rhai scripts
  ├─ particle_physics::collect_async  # join rayon results
  ├─ particle_ramp_system         # color/radius ramp → direct sprite mutation
  ├─ visual_sync_system           # Transform2D → object_state offset/heading
  ├─ compositor_system            # snapshot states → fill back buffer → render layers
  ├─ postfx_system                # (no-op for game scene)
  └─ renderer_system              # diff → SDL2 present → swap
```

Key invariant: **compositor always runs AFTER visual_sync**, so it sees restored positions/headings — not the zeroed-out reset values. The heading reset is not itself a flickering source.

---

## Root Cause 1 — Camera Truncation Jitter (±1 px whole-scene shake)

### The Problem

Entity positions are synced to integer pixels via `visual_sync_system`:

```rust
// engine-scene-runtime/src/materialization.rs:320-322
state.offset_x = *x as i32;   // truncation toward zero
state.offset_y = *y as i32;
```

The Rhai script computed camera position independently via float arithmetic:

```rhai
world.set_camera(ship_x - 320.0, ship_y - 180.0);
```

Which is stored as:

```rust
// engine-scene-runtime/src/behavior_runner.rs:676-677
self.camera_x = *x as i32;   // also truncation toward zero
self.camera_y = *y as i32;
```

The compositor computes screen position as:

```rust
total_origin_x = offset_x - camera_x   // for non-UI layers
```

### The Math

With `ship_x = 319.7`:
- `offset_x = floor(319.7) = 319`
- `camera_x = floor(319.7 - 320.0) = floor(-0.3) = 0`  (truncation toward zero)
- `screen_x = 319 - 0 = 319`

With `ship_x = 320.0`:
- `offset_x = floor(320.0) = 320`
- `camera_x = floor(320.0 - 320.0) = 0`
- `screen_x = 320 - 0 = 320`

**The ship jumps between 319 and 320** as it moves through that range. This ±1 pixel oscillation affects every non-UI element (ship, particles, asteroids) because camera_x is a global offset.

Planets are **UI layers** (`layer.ui = true`), so the compositor skips camera subtraction for them:

```rust
let total_origin_x = if layer.ui { base_x } else { base_x.saturating_sub(camera_x) };
```

This is why planets never flickered.

### The Fix

Quantize position FIRST, then subtract viewport centre:

```rhai
let ship_px = ship_x.round().to_int();   // matches visual_sync rounding
let cam_x = (ship_px - 320).to_float();
world.set_camera(cam_x, cam_y);
```

Now `offset - camera = round(pos) - (round(pos) - 320) = 320`, always.

**Files**: `mods/asteroids/scenes/game/game-loop.rhai` (camera computation), `engine-scene-runtime/src/materialization.rs` (changed `as i32` → `.round() as i32`)

---

## Root Cause 2 — Star/Parallax Offsets Zeroed During Respawn

### The Problem

Star parallax offsets and camera were set **inside** `if ship_id > 0 { ... }`:

```rhai
if ship_id > 0 {
    // ... camera setup ...
    world.set_camera(cam_x, cam_y);
    scene.set("stars-1", "offset.x", ...);
    scene.set("stars-2", "offset.x", ...);
    // ...
}
```

During the 3-second respawn window, `ship_id == 0`. Because `reset_frame_state()` already zeroed all layer offsets at frame start, and the Rhai code above never re-declares them, all star layers snap to `offset = (0, 0)` for every frame of the respawn. Stars appear to jump to random positions and back.

Planet drift code was **outside** the `if ship_id > 0` block, so planets were unaffected — consistent with the user's report.

### The Fix

Move camera + parallax code outside the ship-exists guard. Use `local.last_ship_x/y` as fallback during respawn:

```rhai
let ship_x = local.last_ship_x ?? 1600.0;   // default = world centre
let ship_y = local.last_ship_y ?? 900.0;

if ship_id > 0 {
    // update ship_x/ship_y from live transform
    ship_x = xf["x"] ?? ship_x;
    ship_y = xf["y"] ?? ship_y;
    local.last_ship_x = ship_x;
    local.last_ship_y = ship_y;
}

// Camera and parallax ALWAYS run
world.set_camera(...);
scene.set("stars-1", "offset.x", ...);
```

**File**: `mods/asteroids/scenes/game/game-loop.rhai`

---

## Root Cause 3 — Ship OBJ Cel-Shading Boundary Shimmer

### The Problem

The ship is a 3D OBJ model rendered with cel-shading. Every frame, the Rhai script syncs the 3D roll angle from the physics heading:

```rhai
scene.set("ship-" + ship_id, "obj.roll", -h * 57.29578);
```

Even when the ship appears stationary, the physics heading drifts by sub-degree amounts (floating-point noise, auto-brake oscillation). The OBJ renderer re-renders the full 3D mesh every frame with a slightly different roll angle (`engine-compositor/src/sprite_renderer.rs:1240` → `render_obj_content`). This shifts the 3D projection enough to cross cel-shading level boundaries — face normals that were just above a shading threshold now fall just below, flipping pixel colors between frames. The result is a visible shimmer.

Planets (spheres) have smooth, uniform normals — their cel-shading boundaries are stable regardless of minor angle changes.

The engine's prerender cache (`try_blit_prerendered`, tolerance ±2° yaw / ±1° pitch) doesn't help because the ship's roll accumulates past tolerance boundaries, causing alternation between cached and live renders with different cel-shading patterns.

### The Fix

Quantize the roll angle to nearest 1°:

```rhai
let roll_deg = (-h * 57.29578).round();
scene.set("ship-" + ship_id, "obj.roll", roll_deg);
```

This ensures the OBJ renderer receives one of 360 discrete angles. Consecutive frames with sub-degree heading changes render the same cel-shading pattern → no shimmer.

**File**: `mods/asteroids/scenes/game/game-loop.rhai`

---

## Root Cause 4 — Particle Color Ramp Index Oscillation

### The Problem

The particle ramp system samples a color ramp based on remaining lifetime:

```rust
// engine/src/systems/particle_ramp.rs:37
let idx = ((1.0 - life_ratio) * n as f32).floor() as usize;
```

When `life_ratio` is near a `1/N` boundary (e.g., exactly 0.5 with N=4 colors), floating-point arithmetic can produce values like `1.9999998` or `2.0000002`. `floor()` maps these to different indices (1 vs 2), causing the particle to flip between two colors on consecutive frames.

### The Fix

Scale to `(n-1)` and use `round()` instead of `floor()`:

```rust
let raw = (1.0 - life_ratio) * (n - 1) as f32;
let idx = raw.round() as usize;
```

This creates a stable mapping where each color occupies an equal band with ±0.5 hysteresis at boundaries.

**File**: `engine/src/systems/particle_ramp.rs`

---

## Root Cause 5 — Entity Position Truncation vs Rounding Mismatch

### The Problem

`visual_sync_system` sets entity screen positions via truncation:

```rust
state.offset_x = *x as i32;   // truncation toward zero
```

This causes ±1 pixel jitter for fast-moving entities. Example: a particle at x=100.4 renders at pixel 100, but at x=100.6 it renders at pixel 100 (truncated) — then at x=101.0 it jumps to 101. With `round()`, x=100.4→100 and x=100.6→101, producing smoother motion.

### The Fix

```rust
state.offset_x = x.round() as i32;
state.offset_y = y.round() as i32;
```

**File**: `engine-scene-runtime/src/materialization.rs` (in `apply_particle_visual_sync`)

---

## Why Planets Never Flickered

| Property | Planets | Ship / Particles / Stars |
|----------|---------|--------------------------|
| Layer type | `ui: true` | Non-UI (ship, particles) or UI with dynamic offsets (stars) |
| Camera subtraction | ❌ Skipped | ✅ Applied — triggers Root Cause 1 |
| Offset set unconditionally | ✅ Planet drift code always runs | ❌ Stars/camera inside `if ship_id > 0` — triggers Root Cause 2 |
| 3D model shape | Sphere (uniform normals) | Ship (angular faces) — triggers Root Cause 3 |
| Color ramp | None | Particle ramp — triggers Root Cause 4 |
| Position sync | Static (authored position + sinusoidal offset) | Physics-driven `as i32` — triggers Root Cause 5 |

---

## Key Takeaways for Future Mods

1. **Any code that sets visual state (offsets, visibility, camera) MUST run every frame unconditionally** — never gate it behind entity-exists or game-state checks. `reset_frame_state()` zeros everything.

2. **Camera position must be computed from the quantized entity position**, not independently from the float position. Formula: `cam = round(entity_pos) - viewport_half`.

3. **OBJ roll/yaw/pitch should be quantized** to ≥1° increments to prevent cel-shading boundary shimmer on angular meshes. Spherical meshes are immune.

4. **Color ramp indexing must use `round()` on an `(N-1)` scale**, not `floor()` on an `N` scale, to avoid float boundary oscillation.

5. **Entity position sync should use `round()`, not truncation** (`as i32`), for smoother sub-pixel motion.
