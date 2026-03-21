# OBJ Lighting Features

## Overview

Shell Quest supports dynamic lighting for 3D OBJ sprites with both smooth orbit and instant snap teleportation modes. This document describes the lighting API added in March 2026.

## Light Types

### 1. Directional Lights

Fixed direction lights (like sun):

```yaml
sprites:
  - type: obj
    id: portrait
    source: /assets/models/face.obj
    light-direction-x: 0.5
    light-direction-y: -1.0
    light-direction-z: 0.3
```

**Two directional lights available**:
- Primary: `light-direction-x/y/z` 
- Secondary: `light-2-direction-x/y/z` with `light-2-intensity`

### 2. Point Lights (Smooth Orbit)

Point lights that smoothly orbit around the model:

```yaml
sprites:
  - type: obj
    id: portrait
    source: /assets/models/face.obj
    light-point-x: 2.0
    light-point-y: 0.0
    light-point-z: 0.0
    light-point-intensity: 1.5
    light-point-colour: "#00ffff"
    light-point-orbit-hz: 0.1  # One full rotation every 10 seconds
```

**Fields**:
- `light-point-x/y/z` — initial position in 3D space
- `light-point-intensity` — light strength (default 1.0)
- `light-point-colour` — hex color (e.g., "#ff0000" for red)
- `light-point-flicker-hz` — flicker frequency (pulsing effect)
- `light-point-flicker-depth` — flicker amplitude (0.0-1.0)
- `light-point-orbit-hz` — rotation speed (Hz, 0 = static)

**Two point lights available**: 
- Light 1: `light-point-*`
- Light 2: `light-point-2-*`

### 3. Point Lights (Snap Teleport)

Point lights that **instantly jump** to pseudo-random positions:

```yaml
sprites:
  - type: obj
    id: portrait
    source: /assets/models/face.obj
    light-point-x: 2.0
    light-point-y: 0.0
    light-point-z: 0.0
    light-point-intensity: 1.5
    light-point-colour: "#ff00ff"
    light-point-snap-hz: 0.2  # Snap to new position 5 times per second
```

**Snap vs Orbit**:
- `light-point-orbit-hz` — smooth continuous rotation
- `light-point-snap-hz` — instant teleport jumps

**Priority**: snap > orbit > static  
If both are set, snap takes precedence.

## Snap Teleport Implementation

### Algorithm

Snap positions are deterministically generated from time:

```rust
// Calculate snap index from time
let snap_index = (elapsed_s * snap_hz).floor() as u32;

// Hash snap index to angle (different seed per light)
let hash = snap_index.wrapping_mul(2654435761u32).wrapping_add(seed);
let angle = (hash as f32 / u32::MAX as f32) * TAU;

// Convert to 3D position (orbit around origin)
let x = angle.cos() * radius;
let y = 0.0;  // Keep at model center height
let z = angle.sin() * radius;
```

**Seeds**:
- Light 1: `0x9e3779b9` (golden ratio)
- Light 2: `0x6c62272d` (different multiplier)

This ensures:
- Deterministic positions (same on replay)
- Different patterns per light
- No synchronized jumps

### Usage Example

Difficulty portraits with dual snap lights:

```yaml
# mods/shell-quest/objects/difficulty-menu.yml
sprites:
  - type: obj
    id: portrait-hard
    source: /assets/models/face-hard.obj
    
    # Colored base lighting
    light-direction-x: 0.3
    light-direction-y: -0.8
    light-direction-z: 0.5
    
    # Snap light 1 (red accent)
    light-point-x: 2.5
    light-point-y: 0.0
    light-point-z: 0.0
    light-point-intensity: 1.2
    light-point-colour: "#ff0000"
    light-point-snap-hz: 0.2  # ~5s intervals
    
    # Snap light 2 (blue accent)
    light-point-2-x: -2.5
    light-point-2-y: 0.0
    light-point-2-z: 0.0
    light-point-2-intensity: 1.0
    light-point-2-colour: "#0088ff"
    light-point-2-snap-hz: 0.25  # ~4s intervals
```

**Result**: Lights instantly jump to new positions at different intervals, creating dynamic neon-like illumination without smooth animation.

## Cel Shading

OBJ sprites support cel-shaded (toon) rendering:

```yaml
sprites:
  - type: obj
    id: portrait
    source: /assets/models/face.obj
    cel-levels: 4
    shadow-colour: "#000033"
    midtone-colour: "#333366"
    highlight-colour: "#6666ff"
    tone-mix: 0.8
```

**Fields**:
- `cel-levels` — number of brightness bands (2-8, default 3)
- `shadow-colour` — darkest band color
- `midtone-colour` — middle band color
- `highlight-colour` — brightest band color
- `tone-mix` — blend factor (0.0 = original, 1.0 = full cel)

**Cel + Snap Lights**: Combining cel shading with snap teleport lighting creates a striking silhouette effect with discrete color bands.

## Performance Notes

### Snap vs Orbit

**Snap teleport** (instant jumps):
- ✅ Zero interpolation overhead
- ✅ Deterministic (no float accumulation drift)
- ✅ Dramatic visual impact
- ❌ Jerky movement (by design)

**Smooth orbit** (continuous rotation):
- ✅ Smooth animation
- ❌ Per-frame sin/cos calculations
- ❌ Float accumulation can cause drift over long sessions

For **static portraits** (difficulty menu, character select), snap is preferred:
- More dramatic lighting changes
- No performance overhead from interpolation
- Deterministic timing

For **animated scenes** (cutscenes, interactive), orbit may be better:
- Smoother visual flow
- Less jarring transitions

## Schema Integration

Lighting fields are defined in:
- `schemas/scene.schema.yaml` — base sprite contracts
- `mods/*/schemas/sprites.yaml` — per-mod overlays (generated)

Schema generation validates all lighting parameters via metadata in `engine-core/src/scene/metadata.rs`.

## Implementation Files

**Model** (`engine-core/src/scene/sprite.rs`):
```rust
pub enum Sprite {
    Obj {
        // ... other fields
        light_point_orbit_hz: Option<f32>,
        light_point_snap_hz: Option<f32>,
        light_point_2_orbit_hz: Option<f32>,
        light_point_2_snap_hz: Option<f32>,
        // ...
    }
}
```

**Metadata** (`engine-core/src/scene/metadata.rs`):
```rust
metadata.insert("light-point-orbit-hz", param("f32")
    .with_description("Light orbit rotation frequency in Hz"));
metadata.insert("light-point-snap-hz", param("f32")
    .with_description("Light snap teleport frequency in Hz"));
```

**Renderer** (`engine/src/systems/compositor/obj_render.rs`):
```rust
fn snap_angle(elapsed_s: f64, snap_hz: f32, seed: u32) -> f32 {
    let snap_index = (elapsed_s * snap_hz as f64).floor() as u32;
    let hash = snap_index.wrapping_mul(2654435761u32).wrapping_add(seed);
    (hash as f32 / u32::MAX as f32) * TAU
}
```

**Priority logic**:
```rust
let angle = if snap_hz > 0.0 {
    snap_angle(elapsed_s, snap_hz, seed)
} else if orbit_hz > 0.0 {
    (elapsed_s * orbit_hz as f64 * TAU as f64) as f32
} else {
    0.0
};
```

## History

**March 2026**: Snap teleport lighting implemented
- Added `light-point-snap-hz` and `light-point-2-snap-hz` fields
- Snap takes priority over orbit in renderer
- Uses deterministic hash for pseudo-random positions
- Different seeds prevent synchronized jumps
- Applied to difficulty menu 3D portraits

**Commits**:
- Snap lighting implementation and difficulty portraits
- Boot choreography fixes (separate task)

## See Also

- `scene-centric-authoring.md` — Full sprite authoring contract
- `timeline-architecture.md` — Sprite visibility and timing
- `mods/shell-quest/objects/difficulty-menu.yml` — Snap lighting example
