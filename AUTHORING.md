# Content Authoring Guide

Shell Engine content is metadata-driven. Effect metadata, scene schemas, CLI
tools, and the TUI editor all derive from a single source of truth defined in
`engine-core`. When metadata is correct, everything else follows.

For the runtime/component refactor status, use [oop.impl.md](oop.impl.md) as
the execution ledger. When a content or authoring PR is part of that rollout,
tag it with the matching workstream ids such as `CAN-01`, `SCN-04`, or
`OBJ-04`.

Current metadata maturity:

| Area    | Coverage |
|---------|----------|
| Effects | ~80%     |
| PostFX  | ~70%     |
| Overall | ~45-55%  |

---

## Mod Structure

```
mods/<mod>/
+-- mod.yaml                 Mod manifest
+-- levels/                  Level payloads (*.yml|*.yaml|*.json)
+-- audio/
|   +-- sfx.yaml             Semantic SFX bank (events -> cue variants)
|   +-- songs/               Sequenced song files (*.yml|*.yaml)
|   +-- synth/               Note-sheet cue definitions (*.yml|*.yaml)
+-- assets/
|   +-- images/              Image assets (PNG, GIF)
|   +-- fonts/               Rasterized font manifests
|   +-- 3d/                  OBJ/MTL meshes
|   +-- raw/                 Local staging (gitignored)
+-- objects/                 Reusable prefabs (*.yml)
+-- stages/                  Reusable stage presets
+-- behaviors/               Mod-level behaviors (`script:` inline or `src:` external Rhai)
+-- scenes/
    +-- <scene>.yml          Single-file scene
    +-- <scene>/             Scene package
        +-- scene.yml
        +-- layers/*.yml
        +-- templates/*.yml
        +-- objects/*.yml
        +-- behaviors/*.yml
```

A scene is either a single YAML file or a package directory containing
`scene.yml` plus partials. Both forms are interchangeable at runtime.

### Scene space and camera model

Scenes should author a default `render-space` of `2d` or `3d`.
The older `space` field remains a compatibility alias.

- `render-space: 2d` keeps the existing world-camera behavior: non-screen layers are offset by `world.set_camera(x, y)`.
- `render-space: 3d` makes `3d` the default for inherited layers and enables the shared scene 3D camera driven by `world.set_camera_3d_look_at(...)` and `world.set_camera_3d_up(...)`.

Layers can override that default with `space: inherit | 2d | 3d | screen`.

- `2d` uses the scene's 2D camera.
- `3d` ignores the 2D camera and is intended for OBJ / Scene3D content.
- `screen` is fixed HUD space.

Scenes can also declare an optional `spatial:` block to set shared world-unit
scale and axis policy for runtime systems:

```yaml
spatial:
  meters-per-world-unit: 50.0
  virtual-pixels-per-world-unit: 8.0
  handedness: right
  up-axis: y
```

- `meters-per-world-unit` defines physical scale for `world_units -> meters`.
- `virtual-pixels-per-world-unit` is an optional 2D projection helper.
- `handedness` / `up-axis` define the scene's world-space axis convention.

OBJ, `planet`, and `scene3_d` sprites can opt into the shared scene camera with `camera-source: scene`. The default remains `camera-source: local`, which preserves authored per-sprite camera values.

Scenes should author an explicit camera rig through `camera-rig`.
Pair it with `controller-defaults`, which is the canonical scene-level preset
selection surface. Scene normalization lowers that authored contract into the
existing `input.*` runtime camera surface only as a compatibility bridge where
older runtime seams still consume those fields.

For a euclidean object-viewer scene, prefer:

```yaml
controller-defaults:
  camera-preset: obj-viewer

camera-rig:
  preset: obj-viewer
  obj-viewer:
    sprite_id: demo-mesh
```

```yaml
camera-rig:
  preset: surface-free-look
  surface:
    mode: locked
  free-look-camera: {}
```

`camera-rig.surface.mode: locked` is the explicit authored surface-lock
contract. The compiler lowers it to the current
`input.free-look-camera.surface-mode: true` runtime shape.
Direct `input.obj-viewer`, `input.orbit-camera`, and `input.free-look-camera`
blocks should be treated as compatibility-only authoring for older scenes.
New or refreshed content should prefer `controller-defaults + camera-rig`.

Pure preset-driven controllers do not need an extra low-level camera block.
If the preset catalog already declares the rig target and controller kind, keep
the scene contract at:

```yaml
controller-defaults:
  camera-preset: qwack-chase

camera-rig:
  preset: qwack-chase
```

For a concrete authored example, see `mods/qwack-3d/scenes/chase/scene.yml`.

Scenes can also bind themselves to a celestial scope:

```yaml
render-space: 3d

celestial:
  scope: system
  region: local-cluster
  system: sol
  focus-body: earth
  frame: focus-relative
  clock-source: scene
```

This block does not render anything by itself. It tells the engine which
celestial slice the scene is about, so future body/system/region views can
share one persistent world model instead of hardcoding scope in Rhai.

### Planet sprite

Planets can now be authored as one first-class sprite backed by the mod's body
and planet catalogs instead of hand-authoring multiple OBJ shells in the scene.

```yaml
- type: planet
  id: main-planet-view
  body-id: main-planet
  mesh-source: /assets/3d/sphere.obj   # optional; default is this path
  pitch-deg: -14
  camera-source: scene
  fov-degrees: 65
```

`body-id` resolves against `catalogs/celestial/bodies.yaml`. The body chooses the default visual preset via
`planet_type`, while the scene sprite owns framing and presentation. Rhai
should drive the runtime state through high-level paths such as
`planet.spin_deg`, `planet.cloud_spin_deg`, `planet.cloud2_spin_deg`,
`planet.observer_altitude_km`, and `planet.sun_dir.x/y/z`.

Celestial catalogs can now be grouped under `catalogs/celestial/`:

```text
catalogs/
  celestial/
    regions.yaml
    systems.yaml
    bodies.yaml
    planets.yaml
    sites.yaml
    routes.yaml
```

Object instances support both explicit entries and a repeat shorthand. The
`repeat` form expands at compile time and supports `{i}` token substitution
in `as`/`id` and string values inside `with`.

```yaml
objects:
  - repeat:
      count: 8
      ref: bullet-vector
      as: bullet-{i}
      with:
        id: bullet-{i}
        x: 0
        y: 0
```

---

## Gameplay Catalogs

Gameplay catalogs define reusable data templates for a mod's entities and actions.
Instead of hardcoding behavior logic in Rust, mod authors can declare gameplay
configs in YAML and reference them from Rhai scripts.

### Catalog Structure

Catalogs live in `mods/<mod>/catalogs/`:

```
mods/<mod>/catalogs/
+-- input-profiles.yaml     Player input key bindings
+-- prefabs.yaml            Entity templates (ship, enemy, bullet, etc.)
+-- weapons.yaml            Weapon configs (bullets, cooldown, speed)
+-- emitters.yaml           Particle emitter configs (smoke, sparks, etc.)
+-- spawners.yaml           Spawn groups and waves (initial setup, split behavior)
```

All files are optional. If a catalog is missing, scripts fall back to hardcoded defaults.
Catalogs are loaded during mod initialization and cached as a World resource.

### Loading Catalogs from Scripts

#### Input Profiles

```rhai
// Load player input key bindings
let profile = input.load_profile("game.default");
// Emits BindInputAction commands; engine input system receives bindings
```

**Catalog format:**

```yaml
# mods/my-game/catalogs/input-profiles.yaml
profiles:
  default:
    bindings:
      turn_left: ["Left", "a", "A"]
      turn_right: ["Right", "d", "D"]
      thrust: ["Up", "w", "W"]
      fire: [" ", "f", "F"]
```

#### Prefabs

```rhai
// Spawn entity from template
let ship_id = world.spawn_prefab("ship", { x: 100, y: 100, heading: 0 });
```

**Catalog format:**

```yaml
# mods/my-game/catalogs/prefabs.yaml
prefabs:
  ship:
    kind: "ship"
    sprite_template: "ship"
    init_fields:
      x: 0
      y: 0
  enemy:
    kind: "enemy"
    sprite_template: "enemy-template"
    init_fields:
      x: 0
      y: 0
      vx: 0
      vy: 0
```

Prefab entries may also `ref` another prefab and deep-merge nested maps, so
child prefabs can override only the component subfields they need.

#### Spawn Groups

```rhai
// Spawn batch of entities (e.g., initial enemies)
world.spawn_group("game.initial", "enemy");
```

**Catalog format:**

```yaml
# mods/my-game/catalogs/spawners.yaml
groups:
  game.initial:
    prefab: "enemy"
    spawns:
      - {x: -300, y: -210, vx: 2.0, vy: 0.0}
      - {x: 300, y: -210, vx: 0.0, vy: 2.0}
      # ... more spawn specs
```

#### Spawn Waves

```rhai
// Dynamic spawn wave near arena edges while keeping distance from the player
world.spawn_wave("game.dynamic", #{
  spawn_count: 6,
  ship_x: 0,
  ship_y: 0,
  min_x: -320.0,
  max_x: 320.0,
  min_y: -240.0,
  max_y: 240.0
});
```

**Catalog format:**

```yaml
# mods/my-game/catalogs/spawners.yaml
waves:
  game.dynamic:
    prefab: "enemy"
    size_distribution:
      # Large entities (idx 0-2)
      - {min_idx: 0, max_idx: 2, size: 3}
      # Medium entities (idx 2-5)
      - {min_idx: 2, max_idx: 5, size: 2}
      # Small entities (idx 5+)
      - {min_idx: 5, size: 1}
```

#### Weapons

```rhai
// Keep weapon policy in mod-side Rhai helpers built on generic engine primitives.
let bullet_id = h::fire_weapon(world, audio, ship_id, cfg);
```

**Catalog format:**

```yaml
# mods/my-game/catalogs/weapons.yaml
weapons:
  game.ship:
    max_bullets: 8           # Max bullets on screen
    bullet_kind: "bullet"    # Prefab to spawn
    cooldown_name: "fire"    # State key for cooldown tracking
    spawn_offset: 20.0       # Distance from ship center
```

#### Emitters

```rhai
// Keep emitter policy in mod-side Rhai helpers as well.
h::emit_thrust_smoke(world, ship_id, 350);
```

**Catalog format:**

```yaml
# mods/my-game/catalogs/emitters.yaml
emitters:
  game.ship_thrust_smoke:
    max_count: 10            # Max particles in pool
    cooldown_name: "smoke"   # State key for spawn throttle
    cooldown_ms: 48          # Base emit cadence
    min_cooldown_ms: 16      # Faster cadence at sustained thrust
    ramp_ms: 2000            # Time to reach min cadence
    lifecycle: "Ttl"         # Also supports OwnerBound / FollowOwner / TtlFollowOwner
    local_x: 0.0             # Precise owner-local anchor (+right)
    local_y: 6.0             # Precise owner-local anchor (+down)
    emission_local_x: 0.0    # Precise owner-local base direction (+right)
    emission_local_y: 1.0    # Precise owner-local base direction (+down)
    emission_angle: 0.0      # Extra base rotation (radians)
    backward_speed: 0.35     # Relative speed to entity
    ttl_ms: 520              # Particle lifetime (ms)
    radius: 3                # Visual particle size
    follow_local_x: -6.0     # Optional owner-local follow offset
    follow_local_y: 0.0
    follow_inherit_heading: true
```

Emitter precedence is deterministic:
- Anchor: args `local_x/local_y` → catalog `local_x/local_y` → catalog edge interpolation → legacy `spawn_offset/side_offset`
- Base direction: args `emission_local_x/y` → catalog `emission_local_x/y` → default backward axis
- Final direction: base direction rotated by catalog `emission_angle` and per-call `spread`

**Optional emitter runtime fields**:
- `thread_mode`: `light` (main thread), `physics` (worker thread + full particle physics), or `gravity` (worker thread + gravity-focused path)
- `collision` / `collision_mask`: enable tag-filtered particle collisions
- `gravity_scale`: scalar gravity multiplier
- `gravity_mode`: `flat` or `orbital`
- `gravity_center_x` / `gravity_center_y` / `gravity_constant`: orbital attractor for planet-centric particle motion
- `palette_ramp` / `color_ramp` / `radius_max` / `radius_min`: lifetime-driven particle colour and size ramps

Example orbital thruster exhaust:

```yaml
emitters:
  ship.main:
    local_x: 0.0
    local_y: 2.7
    emission_local_x: 0.0
    emission_local_y: 1.0
    lifecycle: TtlOwnerBound
    thread_mode: physics
    gravity_mode: orbital
    gravity_center_x: 1600.0
    gravity_center_y: 900.0
    gravity_constant: 2000000.0
    palette_ramp: thruster
    radius_max: 2
    radius_min: 0
```

### Thruster / RCS Emitter Pattern

The emitter system provides all primitives needed to build physics-accurate
thruster visualizations. Below is the recommended approach for any vehicle
with mounted engines (spacecraft, hovercraft, drones, etc.).

**Coordinate convention** — emitter `local_x/local_y` and `emission_local_x/y`
use the owner's body frame: `+x` = right, `+y` = down (screen space). The
engine automatically transforms these to world space using the owner's heading.

**Mount points**: Define each thruster as a separate emitter in the catalog.
Set `local_x/local_y` to the engine nozzle position relative to the entity
center. Set `emission_local_x/y` to the exhaust direction (away from thrust).

```yaml
emitters:
  # Main engine at stern — pushes forward, exhaust goes backward (+y)
  ship.main_engine:
    local_x: 0.0
    local_y: 6.0
    emission_local_x: 0.0
    emission_local_y: 1.0
    max_count: 120
    lifecycle: "Ttl"
    ttl_ms: 200

  # Nose brake — pushes backward, exhaust goes forward (-y)
  ship.nose_brake:
    local_x: 0.0
    local_y: -8.0
    emission_local_x: 0.0
    emission_local_y: -1.0

  # RCS front-left — pushes right, exhaust goes left (-x)
  ship.rcs_fl:
    local_x: -3.0
    local_y: -5.0
    emission_local_x: -1.0
    emission_local_y: 0.0
```

**Velocity decomposition in Rhai** — project world velocity onto ship-local axes:

```rhai
let hv = entity.heading_vector();
let fwd_x = hv["x"];   let fwd_y = hv["y"];
let right_x = -fwd_y;  let right_y = fwd_x;

let vel = entity.physics().velocity();
let v_fwd   = vel[0] * fwd_x   + vel[1] * fwd_y;    // >0 = forward
let v_right = vel[0] * right_x + vel[1] * right_y;   // >0 = rightward
```

**RCS mixing formula** — each thruster combines rotation torque + drift
compensation. Intensity ∈ [0, 1] drives emit frequency / particle count:

```rhai
fn clamp_f(v, lo, hi) { if v < lo { lo } else if v > hi { hi } else { v } }

let left_brake  = clamp_f( v_right / MAX_SIDE, 0.0, 1.0);  // drifting right → push left
let right_brake = clamp_f(-v_right / MAX_SIDE, 0.0, 1.0);  // drifting left  → push right
let turn_left   = if input_turn < 0 { 1.0 } else { 0.0 };
let turn_right  = if input_turn > 0 { 1.0 } else { 0.0 };

// Front-left nozzle fires for: CW rotation + brake leftward drift
let rcs_fl = clamp_f(turn_right + right_brake, 0.0, 1.0);
// Front-right fires for: CCW rotation + brake rightward drift
let rcs_fr = clamp_f(turn_left  + left_brake,  0.0, 1.0);
```

The pattern extends to any number of thrusters: nose-brake (kills forward
velocity), rear correction (kills backward velocity), lateral pairs (kill
side drift + generate rotation torque). When no input is active, auto-braking
emitters fire proportionally to residual velocity components.

### Script State and Cross-Script Communication

Treat `local` as one per-behavior state slot. New code should keep one owned
state object there, typically `local.state`, and keep raw `local["..."]`
mutation inside a dedicated `state.rhai` or `handoff.rhai` module instead of
scattering keys through the scene entrypoint.

`local.state` belongs to a single behavior instance. Two Rhai behavior files
attached to the same scene do **not** share it.

Use persistent game state for cross-script handoff:

```rhai
// game-loop.rhai
game.set("/my-mod/player_id", ship_id);

// render-sync.rhai
let ship_id = game.get_i("/my-mod/player_id", 0);
```

Use `local.state` for frame-to-frame state that is private to one behavior
script, and `game.set/get` when another behavior or scene needs to read it.

Keep `main.rhai` thin: import domain modules, load/update/store `local.state`,
and delegate gameplay/HUD/render/persist work to `mods/<mod>/scripts/...`
modules.

### World Bounds and Wrapping

Use the natural argument order when setting script-visible world bounds:

```rhai
world.set_world_bounds(-320.0, -240.0, 320.0, 240.0);
```

The order is:

```text
min_x, min_y, max_x, max_y
```

### Engine vs Mod Responsibilities

Keep the engine-level Rhai surface generic. Mod-specific gameplay policy such as
weapon firing rules, enemy split behavior, ship-hit reactions, emitter logic,
or shape construction belongs in shared Rhai modules inside the mod.

---

## Asset System

Mental model (each level builds on the previous):

```
Asset (file data)
  -> Sprite (drawable node)
    -> Object (reusable prefab)
      -> Layer (visual slice)
        -> Scene (flow + composition)
```

Asset paths use a leading `/` and resolve relative to the mod root. The same
paths work for both unpacked directories and zip-packaged mods.

### Asset Categories

| Category     | Location           | Loader              | Notes                          |
|--------------|--------------------|----------------------|--------------------------------|
| Images       | assets/images/     | image_assets.rs      | PNG, GIF (animated), static    |
| Fonts        | assets/fonts/      | font_loader.rs       | Manifest-based, generic:* built-in |
| OBJ meshes   | assets/3d/ or scenes/ | mesh_assets.rs    | Wavefront OBJ + MTL           |
| YAML prefabs | objects/, layers/  | engine-authoring     | Reusable authored resources    |

`mod.yaml` can define `display.default_font`. Then any text sprite using
`font: "default"` resolves to that spec; if unset, the engine fallback font is
used. For readable GUI/HUD text, prefer `generic:2`; reserve `generic:1` for
very dense compact readouts only.

For sharper GUI/HUD text without increasing world rendering cost, set an
explicit world/UI split in `mod.yaml`:

```yaml
display:
  world_render_size: 640x360
  ui_render_size: 1280x720
  ui_layout_size: 1280x720
```

This keeps world rendering authored at 640x360 while compositing UI on a denser
1280x720 target. `ui_layout_size` controls the coordinate space used by UI
layout and hit-testing:

- match `ui_render_size` when you want a native higher-density UI layout with more room for controls,
- match `world_render_size` when you need to keep an older HUD coordinate space intact while the runtime root surface is being aligned.

Audio authoring is mod-root based:
- `audio/sfx.yaml` defines semantic events such as `ui.menu.select`
- `audio/songs/*.yml` defines sequenced tracks/patterns
- `audio/synth/*.yml` defines generated cues; the engine synthesizes these into in-memory buffers at startup

---

## Sprite Types

### Core Types

| Type   | Purpose              | Key Fields                                                          | Asset-backed? |
|--------|----------------------|---------------------------------------------------------------------|---------------|
| text   | Terminal/raster text | content, font, fg, bg, scale-x, scale-y, reveal_ms, glow, text-transform, max-width, overflow-mode, wrap-mode, line-clamp, reserve-width-ch, line-height | Only with named fonts |
| image  | Display image        | source, width, height, stretch-to-area                              | Yes           |
| obj    | 3D mesh render       | source, scale, yaw/pitch/roll, surface-mode                         | Yes           |
| grid   | Layout container     | columns, rows, gap-x/y, children                                    | No            |
| flex   | Stack container      | direction, gap, children                                            | No            |
| panel  | UI panel box         | width, height, padding, bg_colour (omit for transparent), children  | No            |
| vector | Polygon / polyline   | points, closed, draw-char, fg_colour, bg_colour                     | No            |

`surface-mode` on `obj` sprites controls OBJ shading style only. It is separate
from `camera-rig.surface.mode` and the lowered
`input.free-look-camera.surface-mode`, which lock the authored camera to a
spherical surface.

`scale-x` / `scale-y` on `text` sprites are float multipliers applied during
buffer blit (1.0 = identity). They work with all raster font paths (`generic:*`
and named bitmap fonts) and are ignored on native terminal text.

For HUD-style text, prefer bounded layout over long sentence rows:
- `max-width` constrains the visible line width in cells.
- `overflow-mode: clip | ellipsis` controls truncation at the width bound.
  `ellipsis` only makes sense when text is bounded by `max-width` or `line-clamp`.
- `wrap-mode: none | word | char` controls how constrained text breaks lines.
- `line-clamp` limits the number of visible lines after wrapping.
  Use it with `wrap-mode: word` or `wrap-mode: char`; with `wrap-mode: none` it has no authored multi-line contract.
- `reserve-width-ch` keeps value fields visually stable by reserving width in character cells.
- `line-height` is a per-line advance multiplier for wrapped/rasterized text.
  `1` = normal line advance, `2` = double-spaced advance.

**Transparent panels** — omit `bg_colour` entirely (or set `bg_colour: "reset"`)
to make a panel box fully transparent. The engine will skip writing background
cells and let lower z-layers show through. Do _not_ set `bg_colour: "#000"` or
any solid colour on HUD corner panels that should overlay the game field.

### Built-in Generic Font (`generic:*`)

The engine ships a 5×7 pixel bitmap font with several rendering modes:

| Font spec       | Glyph size (px) | Notes                            |
|-----------------|-----------------|----------------------------------|
| `generic:1`     | 4×5             | Compact tiny — dense micro readouts only |
| `generic:2`     | 6×7 (scale 1)   | Standard readable default for GUI / HUD / overlays |
| `generic:3`     | 12×14 (scale 2) | Large — titles, life icons       |

Supported special glyphs: `♥` (retro 5×7 heart), `→←↑↓`, `·•…` and all
printable ASCII. To render chunky retro icon sprites (e.g. life icons), use
`generic:3` + `scale-x: 2.0` + `scale-y: 2.0` → 24×28 px per glyph.

### Sugar Types

These compile down to core types during the authoring pipeline:

| Sugar          | Compiles To  | Purpose                                 |
|----------------|--------------|------------------------------------------|
| window         | panel        | UI window with title/body/footer slots   |
| terminal-input | window       | Prompt widget with hint/input slots      |
| scroll-list    | grid         | Scrollable list with menu-carousel       |
| frame-sequence | timed images | Stop-motion animation                    |

---

## Scene Contract

A `scene.yml` controls the following concerns:

| Concern     | Fields                                                    |
|-------------|-----------------------------------------------------------|
| Identity    | id, title                                                 |
| Lifecycle   | stages, stages-ref                                        |
| Composition | layers (ordered list of visual slices)                    |
| Lighting    | lighting.ambient-floor                                     |
| PostFX      | postfx (ordered list of post-processing passes)           |
| UI          | ui.enabled, ui.persist, ui.theme, ui.scale, ui.font-scale, ui.focus-order |
| Routing     | next, menu-options (each with `to`)                       |
| Input       | input profiles (obj-viewer, free-look-camera, orbit-camera) |
| Prerender   | prerender hooks                                           |

`ui.scale` multiplies GUI widget geometry for `scene.gui.widgets` controls, so hitboxes,
popup rows, and authored control rects can be resized scene-wide without rewriting each widget.
`ui.font-scale` multiplies rasterized text scale for layers marked with `ui: true`.
Use them together for scene-wide HUD readability tuning before hand-authoring per-sprite
`scale-x/scale-y` overrides.

Playground ladder entries to keep in sync with scene-contract changes:

| ID | Path | Purpose |
|----|------|---------|
| CAN-01 | `mods/playground/scenes/world-model-planar/scene.yml` | planar 2D world-model contract |
| CAN-02 | `mods/playground/scenes/world-model-euclidean/scene.yml` | neutral 3D world-model contract authored through `controller-defaults` + `camera-rig` |
| CAN-03 | `mods/playground/scenes/world-model-celestial/scene.yml` | celestial 3D world-model contract with canonical `controller-defaults.camera-preset: surface-free-look`, `camera-rig.preset: surface-free-look`, and `camera-rig.surface.mode: locked` |
| SCN-04 | `mods/playground/scenes/world-model-contracts/scene.yml` | contract ledger / scene map for the planar, euclidean, and celestial ladder plus the surface-locked camera rule |
| BASE-01 | `mods/playground/scenes/menu/scene.yml` | baseline menu scene |
| BASE-02 | `mods/playground/scenes/3d-scene/scene.yml` | baseline OBJ wireframe scene authored through canonical `controller-defaults.camera-preset: obj-viewer` + `camera-rig.preset: obj-viewer` |

`mods/planet-generator/catalogs/prefabs.yaml` keeps the prefab merge canary
pair (`canary-probe-base` and `canary-probe-nested`) for nested override checks.

For a repo-wide snapshot that measures scene contract, world-model,
controller-default, camera-input, object-ref usage, and the playground ladder,
run:

```bash
cargo run -p devtool -- snapshot --all-mods
```

---

## PostFX Pipeline

PostFX passes execute after the compositor and before final frame presentation.
Order matters — passes apply sequentially to the composited buffer.

| Pass            | Purpose                           |
|-----------------|-----------------------------------|
| crt-underlay    | Soft glow under content           |
| crt-distort     | Tube curvature + margins          |
| crt-scan-glitch | Scanline sweep + chroma glitch    |
| crt-ruby        | Ruby tint + edge reveal           |
| lens-blur       | Separable Gaussian blur / soft focus haze |
| crt-filter      | Full-screen CRT display pass      |

---

## Scene Lighting

Scene-level lighting supports:

- `lighting.ambient-floor` (0.0..1.0): minimum base illumination floor for all
  rasterized geometry. Use `0.0` for full black night-side, higher values to lift
  darkness globally.
- `view.profile`: preferred top-level reusable 3D scene-view preset.
- `view.lighting-profile`: optional lower-level lighting profile reference.
- `view.space-environment-profile`: optional lower-level observation-environment
  reference used by the resolved 3D scene view.

Profile references resolve in this order:

- built-in engine preset ids such as `orbit-realistic` or `space-hard-vacuum`
- asset-backed mod profiles by explicit path, e.g. `/view-profiles/orbit.yml`
- asset-backed mod profiles by conventional id lookup:
  - `/view-profiles/<id>.yml|yaml`
  - `/lighting-profiles/<id>.yml|yaml`
  - `/space-environment-profiles/<id>.yml|yaml`

## Free-Look Camera Surface Mode

`camera-rig.free-look-camera` and the lowered `input.free-look-camera`
compatibility bridge both accept surface-constrained controls for planet
navigation:

- `surface-mode`: `true` to keep the camera on a spherical shell.
- `surface-center-x/y/z`: sphere center in world units.
- `surface-radius`: base planet radius.
- `surface-altitude`: initial altitude above surface.
- `surface-min-altitude` / `surface-max-altitude`: altitude clamp.
- `surface-vertical-speed`: climb/descent speed for `q` / `e`.

If you pair the rig with `camera-rig.preset: surface-free-look`, set
`camera-rig.surface.mode: locked` explicitly. The compiler then lowers that
surface contract through the current `input.free-look-camera.surface-mode: true`
compatibility seam.

## OBJ Lighting

Scene3D assets and OBJ sprites can combine directional, point, and ambient
lighting with cel-shaded tonal bands.

### Light Types

| Type        | Fields                                              | Behavior |
|-------------|-----------------------------------------------------|----------|
| Directional | `direction`, `colour`, `intensity`                  | Infinite parallel rays |
| Point       | `position`, `colour`, `intensity`, `falloff_constant`, `snap_hz` | Local light with quadratic attenuation and optional snap/strobe timing |
| Ambient     | `intensity`, `colour`                               | Flat base illumination |
| Cel shading | `cel-levels`, `tone-mix`, `shadow/midtone/highlight-colour` | Posterized tonal bands |

`falloff_constant` uses the attenuation curve `1 / (1 + k * dist²)`. Small
unit-scale scenes work with the default `0.7`; large background scenes usually
need values closer to `0.001`.

### Live Scene3D Clips

Static Scene3D frames are still pre-rendered into the atlas at scene load. Clip
frames additionally stay available as parsed runtime definitions, so a sprite
may keep `frame: solar-orbit` and let the compositor evaluate the clip at the
current timeline position each frame.

Clip tweens currently cover:

- `yaw_offset`
- `opacity`
- `translation_x/y/z`
- `orbit_angle_deg`
- `orbit_center_x/y/z`
- `orbit_radius`
- `orbit_phase_deg`
- `clip_y_min` / `clip_y_max`

### OBJ Planet / Cloud Authoring

Large planet-style OBJ sprites can now be authored directly in scene YAML using
procedural biome controls:

- `terrain-color`, `terrain-threshold`, `terrain-noise-scale`, `terrain-noise-octaves`, `marble-depth`
- `below-threshold-transparent` for cloud shells and other overlay-only layers
- `polar-ice-color`, `polar-ice-start`, `polar-ice-end`
- `desert-color`, `desert-strength`
- `atmo-color`, `atmo-strength`, `atmo-rim-power`
- `night-light-color`, `night-light-threshold`, `night-light-intensity`

Prefer the builtin `type: planet` sprite for body-backed worlds. Raw OBJ planet
authoring remains useful for one-off art shots, background shells, or effects
that are intentionally not tied to a body catalog.

For cockpit-follow or parallax-heavy planet shots, Rhai can also drive
per-sprite OBJ camera/view overrides via `runtime.scene.objects.find(id).set("obj.cam.wx", ...)`,
`obj.cam.wy/wz`, and `obj.view.rx/ry/rz`, `obj.view.ux/uy/uz`,
`obj.view.fx/fy/fz`.

### World-Generated Planets (`world://` URI)

The `world://` URI scheme generates procedural planets via `engine-terrain`.
Unlike `type: planet` (which references a body catalog), `world://` sprites
are fully parameterised and driven by Rhai at runtime.

**Scene YAML:**

```yaml
- type: obj
  id: planet-mesh
  source: "world://32"
  prerender: false
  ambient: 0.12
  rotation-speed: 3.0
```

The `32` in `world://32` sets the mesh subdivision level. Higher values
produce more detailed terrain but take longer to generate. Recommended
values: 32 (default), 64 (good quality), 128 (high resolution), 256–512
(very high res — expect multi-second generation at 512).

The optional `world-base` YAML field selects the sphere topology:

```yaml
- type: obj
  id: planet-mesh
  source: "world://64"
  world-base: cube      # cube (default) | uv | tetra | octa | icosa
```

| Value | Topology |
|-------|---------|
| `cube` | Cube-sphere — uniform triangles, no pole singularity (default) |
| `uv` | UV sphere — classic lat/lon topology |
| `tetra` | Tetrahedron recursively subdivided |
| `octa` | Octahedron recursively subdivided |
| `icosa` | Icosahedron recursively subdivided (icosphere) |

**Rhai parameter control:**

```rhai
let mesh = runtime.scene.objects.find("planet-mesh");
mesh.set("world.seed", 42);
mesh.set("world.ocean_fraction", 0.55);
mesh.set("world.continent_scale", 2.5);
mesh.set("world.displacement_scale", 0.22);
mesh.set("world.coloring", "biome");
mesh.set("world.subdivisions", 64);
```

Any `world.*` property change rebuilds the URI key and triggers mesh
regeneration on the next compositor pass. Changes are typically throttled
in the script to avoid blocking the render thread.

**Available `world.*` properties:**

| Property | Default | Description |
|----------|---------|-------------|
| `world.seed` | 0 | Random seed (0–9999) |
| `world.ocean_fraction` | 0.55 | Ocean coverage (0.01–0.99) |
| `world.continent_scale` | 2.5 | Continent size (0.5–10) |
| `world.continent_warp` | 0.65 | Coastline chaos (0–2) |
| `world.continent_octaves` | 5 | Continent noise detail (1–8) |
| `world.mountain_scale` | 6.0 | Mountain spacing (1–15) |
| `world.mountain_strength` | 0.45 | Mountain height (0–1) |
| `world.mountain_ridge_octaves` | 5 | Ridge detail (1–8) |
| `world.moisture_scale` | 3.0 | Moisture pattern size (0.5–8) |
| `world.ice_cap_strength` | 1.0 | Polar ice intensity (0–3) |
| `world.lapse_rate` | 0.6 | Altitude cooling rate (0–1.5) |
| `world.rain_shadow` | 0.35 | Rain shadow (0–1) |
| `world.displacement_scale` | 0.22 | Surface displacement (0–0.6) |
| `world.subdivisions` | 32 | Mesh resolution (32/64/128/256/512) |
| `world.coloring` | biome | `"biome"` / `"altitude"` / `"moisture"` / `"none"` |
| `world.base` | cube | Sphere topology: `"cube"` / `"uv"` / `"tetra"` / `"octa"` / `"icosa"` |

Visual-only properties that don't trigger mesh regeneration:

| Property | Default | Description |
|----------|---------|-------------|
| `obj.rotation-speed` | 3.0 | Rotation speed (deg/sec) |
| `obj.ambient` | 0.12 | Ambient light level (0–0.5) |
| `obj.light.x/y/z` | — | Directional light vector |
| `obj.atmo.color` | — | Atmosphere rim color (color name or `"none"` to disable) |
| `obj.atmo.strength` | — | Atmosphere rim blend strength (0.0–1.0) |
| `obj.atmo.rim_power` | — | Rim falloff exponent (0.1–16.0) |
| `obj.atmo.haze_strength` | — | Haze blend strength (0.0–1.0) |
| `obj.atmo.haze_power` | — | Haze falloff exponent (0.1–8.0) |

**Biome stats readback:**

```rhai
let stats = planet_last_stats();
let ocean_pct = stats["ocean"];        // 0.0–1.0
let forest_pct = stats["forest"];
let desert_pct = stats["desert"];
```

---

## Terminal HUD Authoring

### Window

`type: window` compiles to a panel with three slots: title, body, footer.
Slot layout respects the active font height for vertical sizing.

### Terminal Input

`type: terminal-input` is a specialized window for interactive prompts.


### GUI Widgets (`gui:` block)

Scenes can declare a `gui:` block that defines interactive controls. The engine
processes input automatically; Rhai scripts read state via the `gui` API.

```yaml
gui:
  widgets:
    - type: slider
      id: slider-volume
      x: 50
      y: 100
      w: 200
      h: 12
      min: 0.0
      max: 100.0
      value: 50.0
      handle: volume-handle     # sprite alias for engine-level positioning
      hit-padding: 4            # expand clickable area (pixels)

    - type: toggle
      id: toggle-mute
      x: 50
      y: 120
      w: 20
      h: 12

    - type: button
      id: btn-apply
      x: 50
      y: 140
      w: 80
      h: 16

    - type: panel
      id: panel-settings
      sprite: panel-settings
      visible: true

    - type: radio-group
      id: model-picker
      sprite: model-picker-track
      x: 50
      y: 170
      w: 160
      h: 18
      selected: 0
      options:
        - { value: world, label: Planet }
        - { value: sphere, label: Sphere }
        - { value: cube, label: Cube }
      selected-sprites: [sel-world, sel-sphere, sel-cube]

    - type: segmented-control
      id: display-mode
      sprite: display-mode-track
      x: 50
      y: 194
      w: 160
      h: 18
      selected: 0
      options:
        - { value: surface, label: Surface }
        - { value: wire,    label: Wire }
        - { value: heat,    label: Heat }

    - type: tabs
      id: top-tabs
      sprite: top-tabs-track
      x: 50
      y: 220
      w: 220
      h: 18
      selected: 0
      options:
        - { value: land, label: Land }
        - { value: climate, label: Climate }
        - { value: atmo, label: Atmosphere }

    - type: dropdown
      id: preset-picker
      sprite: preset-trigger
      x: 50
      y: 200
      w: 120
      h: 18
      selected: 0
      label-sprite: preset-label
      popup-sprite: preset-popup
      popup-above: true
      options:
        - { value: earth, label: Earth }
        - { value: mars, label: Mars }
```

**Key fields:**
- `handle` (slider only) — sprite ID whose `offset_x` the engine sets automatically
  based on slider value fraction. No Rhai `runtime.scene.objects.find("handle").set("position.x", ...)`
  needed.
- `hit-padding` (slider only) — expands the clickable area beyond the track bounds
  (useful for thin track sprites).
- `selected-sprites` (radio-group) — optional per-option visual markers that the
  engine toggles based on the selected index.
- `type: segmented-control` — neutral sugar over `radio-group` for short
  mutually-exclusive mode choosers that are not semantically tabs.
- `type: tabs` — authoring sugar over the same runtime as `radio-group`, for
  panel/view switching UIs where tab semantics are clearer than a generic chooser.
- `label-sprite` / `popup-sprite` (dropdown) — optional trigger label + popup
  visuals driven directly by GUI runtime state.
- `popup-above` (dropdown) — opens the popup list upward instead of below the
  trigger; useful for bottom HUD rows.
- `text-input` — editable single-line text field with `text-sprite`,
  `placeholder`, `value`, and `max-length`.
- `number-input` — numeric text field on the same focus/keyboard path, with
  optional `min`, `max`, `step`, `text-sprite`, and `placeholder`.

Visual rendering is separate: create Panel/Vector/Text sprites in layer YAMLs
and use Rhai to wire `gui.*` state to visual updates (text content, colors,
visibility). The engine now also supports native visual sync for:
- slider handle movement,
- radio-group selected marker visibility,
- dropdown trigger label text and popup visibility.


### Action Map

The `action_map` section in `mod.yaml` defines named input actions that scripts can query:

```yaml
action_map:
  actions:
    move_left:
      key: a
      repeat: true
    move_right:
      key: d
      repeat: true
    jump:
      key: space
      repeat: false
```

Action names must be valid identifiers (start with letter or `_`, contain only alphanumeric or `_`).
The `key` property is required and specifies the input key code.
The optional `repeat` property (default `false`) indicates whether the action repeats while held.

---

## Rhai Scripting

### Canonical Runtime Model

Write new Rhai against `runtime` as the root namespace.

- `runtime.scene` owns live scene handles, snapshots, scene graph operations,
  and typed scene mutations.
- `runtime.world` owns gameplay-object lookup and entity/world mutation.
- `runtime.services` groups transient engine services such as input, audio,
  effects, collision, diagnostics, palette, and UI.
- `runtime.stores` groups longer-lived stores such as game/session state,
  level state, and persistent save state.

Current builds may still inject convenience aliases such as `scene`, `world`,
`game`, `level`, `persist`, `input`, and `audio`. Treat those as shorthand
entry points, not as the canonical ownership model. Some domains are still
reached through those shorthands in current builds while the runtime root
surface is being aligned.

### Recommended Module Layout

Keep `main.rhai` as a thin orchestration file and move reusable logic into
`mods/<mod>/scripts/`.

```text
mods/<mod>/
  scenes/<scene>/main.rhai
  scripts/
    std/bootstrap.rhai
    std/math.rhai
    <domain>/state.rhai
    <domain>/hud.rhai
    <domain>/render.rhai
```

Recommended entrypoint shape:

```rhai
import "std/bootstrap" as bootstrap;
import "vehicle/state" as state;
import "vehicle/hud" as hud;

local = bootstrap::ensure(local);
let s = state::load(local);
s = state::step(s, runtime, input, frame_ms);
hud::render(s, runtime);
local = state::store(local, s);
```

### Scope Variables

| Variable           | Type          | Preferred use |
|--------------------|---------------|---------------|
| `runtime`          | `RuntimeApi`  | Canonical root namespace for scene, world, service, and store access |
| `scene`            | `SceneApi`    | Concise scene-root shorthand; prefer the runtime-root mental model |
| `world`            | `WorldApi`    | Gameplay shorthand; canonical ownership lives under `runtime.world` |
| `game`             | `GameApi`     | Session store shorthand; conceptually under `runtime.stores` |
| `level`            | `LevelApi`    | Level store shorthand; conceptually under `runtime.stores` |
| `persist`          | `PersistApi`  | Persistent store shorthand; conceptually under `runtime.stores` |
| `input`            | `InputApi`    | Service shorthand; conceptually under `runtime.services` |
| `audio`            | `AudioApi`    | Service shorthand; conceptually under `runtime.services` |
| `effects`          | `EffectsApi`  | Service shorthand; conceptually under `runtime.services` |
| `collision`        | `CollisionApi`| Service shorthand; conceptually under `runtime.services` |
| `ui`               | `UiApi`       | UI service shorthand |
| `diag`             | `DebugApi`    | Diagnostics service shorthand |
| `time`             | `TimeApi`     | Typed frame/stage timing helpers |
| `params`           | `map`         | Effect/behavior parameters from YAML |
| `regions`          | `map`         | Named layout regions |
| `local`            | `dynamic`     | Per-behavior state slot; prefer a single owned `local.state` object |
| `frame_ms`         | `int`         | Actual frame delta in milliseconds |
| `scene_elapsed_ms` | `int`         | Total elapsed ms since scene start |
| `stage_elapsed_ms` | `int`         | Total elapsed ms since current stage start |

This guide intentionally omits compatibility maps such as `objects`, `state`,
`menu`, and `key` from the primary authoring surface.

### `world.*` — Entity World

Spawn, query, mutate, and despawn gameplay entities.

```rhai
// Spawn
world.spawn_prefab(name, args_map)            // spawn from prefab catalog; returns id
world.spawn_visual(kind, template, data_map)  // atomic: create entity + visual + physics + collider
world.spawn_batch(specs_array)                // batch spawn; returns array of ids

// Query
world.count()                                 // total entity count
world.count_kind(kind)                        // count by kind string
world.count_tag(tag)                          // count by tag string
world.any_alive(kind)                         // sugar: count_kind > 0
world.ids()                                   // all entity ids as array
world.query_kind(kind)                        // ids matching kind
world.query_tag(tag)                          // ids matching tag
world.first_kind(kind)                        // first id matching kind, or 0
world.first_tag(tag)                          // first id matching tag, or 0
world.exists(id)                              // is id alive?
world.kind(id)                                // kind string for id
world.tags(id)                                // tag array for id
world.distance(id_a, id_b)                    // euclidean distance between transforms

// Entity data (JSON pointer paths)
world.get(id, "/path")                        // read value
world.set(id, "/path", value)                 // write value
world.has(id, "/path")                        // path exists?
world.remove(id, "/path")                     // delete path
world.push(id, "/path", value)                // append to array at path

// Entity handle (cleaner API for repeated access)
let e = world.entity(id);
e.id()                                        // numeric id
e.exists()                                    // is entity alive?
e.kind()                                      // kind string
e.tags()                                      // tag array
e.get("/path")                                // read data
e.get_i("/path", fallback)                    // typed int
e.get_f("/path", fallback)                    // typed float
e.get_s("/path", fallback)                    // typed string
e.get_b("/path", fallback)                    // typed bool
e.flag(name)                                  // sugar: get_b("/name", false)
e.set_flag(name, value)                       // sugar: set("/name", value)
e.set("/path", value)                         // write data
e.set_many(map)                               // bulk write
e.data()                                      // full JSON data map
e.transform()                                 // #{x, y, heading} map
e.set_position(x, y)                          // move entity
e.set_heading(h)                              // set heading (0-31)
e.lifetime_remaining()                        // ms until expiry, or -1
e.despawn()                                   // despawn + auto-clean bound visuals

// Lookup-oriented gameplay object handle
let obj = world.objects.find("ship-shadow");  // bound visual id or numeric entity id
obj.exists()                                  // lookup succeeded?
obj.inspect()                                 // identity/data snapshot map
obj.get("hp")                                 // generic data read
obj.set("hp", 42)                             // generic data write

// Transform & physics
world.transform(id)                           // #{x, y, heading} map
world.set_transform(id, x, y, heading)
world.physics(id)                             // #{vx, vy, ax, ay, drag, max_speed}
world.set_physics(id, vx, vy, ax, ay, drag, max_speed)
world.body(body_id)                           // typed celestial body snapshot
world.body(body_id).inspect()                 // map snapshot only when typed fields are not enough
world.body_upsert(body_id, patch_map)         // runtime mutate/create celestial body fields
world.body_patch(body_id, patch_map)          // alias of body_upsert for patch-style usage
world.body_gravity(body_id, x, y)             // gravity vector at world position
world.find_planet_spawn(config, biomes)       // procedural spawn lookup -> #{normal_x/y/z, longitude_deg, latitude_deg, surface_radius_scale, biome, ...}

// Colliders
world.set_collider_circle(id, radius, layer_mask, collision_mask)
world.set_collider_polygon(id, points, layer_mask, collision_mask)

// Lifetime
world.set_lifetime(id, ttl_ms)

// Visual binding
world.set_visual(id, visual_id)               // bind entity to scene runtime target
world.bind_visual(id, visual_id)              // alias for set_visual

// Despawn
world.despawn(id)                             // despawn entity
world.despawn_children(parent_id)             // despawn all child entities

// Tags
world.tag_add(id, tag)
world.tag_remove(id, tag)
world.tag_has(id, tag) -> bool

// Toroidal wrap
world.enable_wrap_bounds(id)
world.disable_wrap(id)
world.set_world_bounds(min_x, min_y, max_x, max_y)
world.world_bounds()                          // #{min_x, min_y, max_x, max_y}

// Timers (fire once, identified by label string)
world.after_ms(label, delay_ms)               // arm a one-shot timer
world.timer_fired(label)                      // true once when timer expires
world.cancel_timer(label)                     // cancel pending timer

// Collision event queries (same data as `collision.*`, accessed via world)
world.collision_enters(kind_a, kind_b)        // enter events this frame
world.collision_stays(kind_a, kind_b)         // stay events this frame
world.collision_exits(kind_a, kind_b)         // exit events this frame
world.collisions_of(kind)                     // all hits involving kind (#{self, other})

// RNG (engine-managed, deterministic)
world.rand_i(min, max)                        // integer in [min, max)
world.rand_seed(seed)                         // re-seed the engine RNG

// Arcade controller (entity-level)
let e = world.entity(id)
e.attach_controller(#{ turn_step_ms: 80, thrust_power: 200.0, max_speed: 180.0, heading_bits: 32 })
e.set_turn(dir)                               // dir: -1 / 0 / 1
e.set_thrust(on)
e.heading()                                   // discrete heading index
e.heading_vector()                            // #{x, y} unit vector

// Diagnostics
world.diagnostic_info()                       // #{entity_count, ...} debug map
world.reset_dynamic_entities()                // despawn all non-static entities
```

### `collision.*` — Collision Queries

Dedicated collision namespace — same data as `world.collision_*` but with cleaner ergonomics.

```rhai
// Enter events (first frame two entities overlap)
collision.enters(kind_a, kind_b)              // → [{kind_a: id, kind_b: id}, ...]
collision.enters_of(kind)                     // → [{self: id, other: id}, ...]
collision.any_enter(kind_a, kind_b)           // → bool
collision.count_enters(kind_a, kind_b)        // → int

// Stay events (every frame while overlapping)
collision.stays(kind_a, kind_b)               // → [{kind_a: id, kind_b: id}, ...]
collision.stays_of(kind)                      // → [{self: id, other: id}, ...]

// Exit events (first frame after overlap ends)
collision.exits(kind_a, kind_b)               // → [{kind_a: id, kind_b: id}, ...]

// Raw (unfiltered, no kind lookup)
collision.all_enters()                        // → [{a: id, b: id}, ...]
```

Example:
```rhai
for hit in collision.enters("bullet", "asteroid") {
    let bullet_id   = hit["bullet"];
    let asteroid_id = hit["asteroid"];
    world.despawn(bullet_id);
    // handle split logic...
}
```

### `effects.*` — Runtime Visual Effects

Trigger screen effects from scripts independently of authored YAML steps.

```rhai
effects.shake(duration_ms, amp_x, amp_y, frequency)
// amp_x, amp_y in cells; frequency in oscillations over duration

effects.trigger(name, duration_ms, params_map)
// name: built-in effect name (e.g. "screen-shake", "flash")
// params_map: effect-specific parameters

effects.trigger_loop(name, duration_ms, params_map)
// same as trigger but loops until scene transition
```

Effect names (from engine-core built-ins): `"screen-shake"`, `"flash"`, `"vignette"`.

Example:
```rhai
effects.shake(300, 1.5, 0.5, 8.0);        // short shake on hit
effects.shake(500, 2.5, 1.0, 6.0);        // heavier shake on death
effects.trigger("flash", 200, #{intensity: 0.8});
```

### `audio.*` — Audio

```rhai
audio.cue(cue_id)                   // play direct cue id (asset stem)
audio.cue(cue_id, volume)           // play with volume scale (0.0-1.0)
audio.event(event_id)               // semantic sfx event from audio/sfx.yaml
audio.event(event_id, gain)         // semantic event with gain scale
audio.play_song(song_id)            // start sequenced song from audio/songs/*.yml
audio.stop_song()                   // stop currently active sequenced song
```

Audio event banks live at `<mod_root>/audio/sfx.yaml` (NOT `assets/audio/`).

### `scene.*` — Scene Object Mutations

```rhai
runtime.scene.objects.find("hud-score")       // primary live scene handle lookup
scene.object("hud-score")                     // concise live scene handle shorthand
scene.inspect(target)                         // stable snapshot map for last published scene state
scene.region(target)                          // stable runtime layout snapshot
runtime.scene.objects.find(target).set(path, value)
scene.object(target).set(path, value)
scene.mutate(#{ ... })                      // apply a typed scene mutation request
scene.instantiate(template, target)          // clone a scene object/layer template at runtime
scene.despawn(target)                        // soft-despawn a scene object/layer
```

Important runtime rules:

- `runtime.scene.objects.find(target)` returns the primary live scene handle.
  `scene.object(target)` is the concise root-scene shorthand for the same live
  handle.
- `scene.inspect(target)` and `scene.region(target)` return snapshots. Use them
  for stable inspection, diagnostics, or capability checks rather than for
  same-frame read-after-write flows.
- `world.objects.find(target)` is the gameplay lookup surface for resolving an
  entity from a runtime id or bound visual target. Use `world.entity(id)` when
  the script needs transform/physics/controller helpers instead of generic
  lookup and data access.
- Scene runtime state is immediate-mode. Offsets, visibility, and similar
  transient runtime values are reset before behavior execution each frame, so
  camera-relative parallax and other scripted visual state must be re-applied
  every frame.
- `scene.instantiate(template, target)` reserves `target` for the cloned layer
  or object itself. If the template contains child sprites, continue mutating
  those by their authored sprite `id` values rather than by reusing the runtime
  clone target name for both parent and child lookups.
- For camera-follow and parallax logic, keep values in float space inside Rhai
  and let the runtime perform the final rounding pass. Pre-truncating in script
  makes near layers and particle-bound visuals step more visibly on SDL2.
- `runtime.scene.objects.find(...).set(...)` is the primary per-object write
  path; `scene.object(target).set(...)` is the concise root-scene shorthand.
  New authoring guidance should lead with runtime handles, not with compatibility
  helper shorthands from older scripts.

### `game.*` — Global Game State & Navigation

```rhai
game.get(path)                      // read global game state (JSON pointer)
game.set(path, value)               // write global game state
game.get_i(path, fallback)          // typed int getter
game.get_s(path, fallback)          // typed string getter
game.get_b(path, fallback)          // typed bool getter
game.get_f(path, fallback)          // typed float getter
game.jump(scene_id)                 // scene transition
```

### `level.*` — Level Catalog

```rhai
level.current()                     // active level id
level.ids()                         // all available level ids
level.select(level_id)              // switch active level
level.get(path)                     // read active level payload (JSON pointer)
```

### `input.*` — Input Actions

```rhai
input.bind_action(name, keys)       // register named action binding
input.action_down(name)             // true while action key is held
input.action_just_pressed(name)     // true on the first frame of press
input.mouse_x                       // current mouse x in output-space pixels
input.mouse_y                       // current mouse y in output-space pixels
input.scroll_y                      // signed mouse-wheel delta for this frame
input.ctrl_scroll_y                 // signed mouse-wheel delta while Ctrl was held
```

Key constants for `bind_action` (strings or symbolic constants):
`KEY_LEFT`, `KEY_RIGHT`, `KEY_UP`, `KEY_DOWN`, `KEY_SPACE`, `KEY_ESC`, `KEY_ENTER`, `KEY_F1`…`KEY_F12`

Example:
```rhai
input.bind_action("thrust", [KEY_UP, "w"]);
input.bind_action("fire",   [KEY_SPACE, "f"]);
```

### `persist.*` — Persistent Save State

```rhai
persist.get(path)                   // read on-disk persistent state
persist.set(path, value)            // write on-disk persistent state
```

### Math / Geometry Helpers

Rhai's `Engine::new()` includes all standard math functions:
`sin(x)`, `cos(x)`, `tan(x)`, `atan(y, x)`, `sqrt(x)`, `abs(x)`, `floor(x)`, `ceil(x)`, `round(x)`, `min(a, b)`, `max(a, b)`, `PI`, `TAU`

Engine-specific helpers:
```rhai
abs_i(v)                            // absolute value for integers
sign_i(v, fallback)                 // sign with fallback when v == 0
clamp_i(v, min_v, max_v)            // clamp integer
clamp_f(v, min_v, max_v)            // clamp float
to_i(v) / to_float(v)              // type conversions
sin32(idx)                          // 32-step integer sine lookup (-1024..1024)
rotate_points(points, heading)      // rotate a point array around 0,0 using 32-step heading
regular_polygon(sides, radius)      // generate regular polygon point array
jitter_points(points, jitter, seed) // randomize point positions
dent_polygon(pts, ix, iy, str)     // push closest vertex toward centroid by str%
subtract_polygon(a, b)             // boolean difference: returns array of result polygons
polygon_area(points)                // absolute area of polygon (Shoelace formula)
```

Keep shape-specific helpers such as custom point generators or
similar gameplay geometry in shared Rhai modules inside the mod instead of as
engine-global functions.

### Commands

Scripts emit commands to mutate the scene:

- Visibility: show/hide sprites and layers
- Set-text: update sprite content
- Position: move sprites
- Style: change fg/bg/font/glow

Mod-level named behaviors can live in `behaviors/*.yml` and reference external Rhai:

```yaml
kind: behavior
name: game-loop
src: ./game-loop.rhai
```

Use `script:` for short inline behaviors and `src:` for reusable or larger scripts.

**Important:** Always use backtick strings for multiline text in Rhai:

```rhai
// correct
let msg = `line one
line two`;

// wrong — do not use escaped newlines
let msg = "line one\nline two";
```

---

## Compilation Pipeline

```
1. Load       scene YAML (single-file or package)
       |
2. Expand     refs, objects, stages-ref, cutscene-ref (engine-authoring)
       |
3. Normalize  expand aliases and shorthands
       |
4. Deserialize  into runtime Scene struct
       |
5. Validate   timeline + text-layout authoring checks (debug mode)
       |
6. Execute    lifecycle -> input -> compositor -> postfx -> render
```

---

## Author Checklist

1. Every YAML file has a correct `$schema` reference.
2. `next` and each `menu-options[].to` point to existing scenes.
3. All `ref` / `use` references resolve to valid targets.
4. `cargo run -p schema-gen -- --all-mods` and `cargo run -p schema-gen -- --all-mods --check` pass.
5. Sprite timing falls within scene duration.
6. A smoke run (`cargo run -p app`) starts without compile errors.
7. `cargo run -p app -- --mod-source=mods/<mod> --check-scenes` reports zero warnings before merge.
8. New Rhai uses `runtime` as the canonical root; top-level aliases are treated as shorthand, not as the primary model.
9. `main.rhai` stays thin and delegates domain logic to `mods/<mod>/scripts/...` modules.
10. Suggested migration grep gates for actively maintained mods stay clean:

```bash
rg -n "\\bscene\\.(get|set)\\(" mods AUTHORING.md ARCHITECTURE.md SCRIPTING-API.md
rg -n "type_of\\(local\\)" mods -g '*.rhai'
rg -n "runtime_scene_set\\(" mods -g '*.rhai'
```

As a policy, raw `local["..."]` access should stay inside dedicated
`state*.rhai` / `handoff*.rhai` modules rather than in scene entrypoints.

---

## Daily Workflow

1. Edit YAML files under `mods/<mod>/`.
2. Run `cargo run -p schema-gen -- --all-mods` to regenerate schemas.
3. Continue authoring — editor completions reflect the updated schemas.
4. Run `cargo run -p schema-gen -- --all-mods --check` before merge.
