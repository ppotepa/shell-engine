# Content Authoring Guide

Shell Quest content is metadata-driven. Effect metadata, scene schemas, CLI
tools, and the TUI editor all derive from a single source of truth defined in
`engine-core`. When metadata is correct, everything else follows.

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
+-- prefabs.yaml            Entity templates (ship, asteroid, bullet, etc.)
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
let profile = input.load_profile("asteroids.default");
// Emits BindInputAction commands; engine input system receives bindings
```

**Catalog format:**

```yaml
# mods/asteroids/catalogs/input-profiles.yaml
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
# mods/asteroids/catalogs/prefabs.yaml
prefabs:
  ship:
    kind: "ship"
    sprite_template: "ship"
    init_fields:
      x: 0
      y: 0
  asteroid:
    kind: "asteroid"
    sprite_template: "asteroid-template"
    init_fields:
      x: 0
      y: 0
      vx: 0
      vy: 0
      shape: 0
      size: 1
```

#### Spawn Groups

```rhai
// Spawn batch of entities (e.g., initial asteroids)
world.spawn_group("asteroids.initial", "asteroid");
```

**Catalog format:**

```yaml
# mods/asteroids/catalogs/spawners.yaml
groups:
  asteroids.initial:
    prefab: "asteroid"
    spawns:
      - {x: -300, y: -210, vx: 2.0, vy: 0.0, shape: 0, size: 2}
      - {x: 300, y: -210, vx: 0.0, vy: 2.0, shape: 1, size: 3}
      # ... more spawn specs
```

#### Spawn Waves

```rhai
// Dynamic spawn wave (e.g., asteroid fragments from destroyed parent)
world.spawn_wave("asteroids.dynamic", {
  parent_x: 100,
  parent_y: 100,
  count: 2,
  ship_id: some_ship_id
});
```

**Catalog format:**

```yaml
# mods/asteroids/catalogs/spawners.yaml
waves:
  asteroids.dynamic:
    prefab: "asteroid"
    size_distribution:
      # Large asteroids (idx 0-2) split into 2 medium asteroids (size 3)
      - {min_idx: 0, max_idx: 2, size: 3}
      # Medium asteroids (idx 2-5) split into 2 small asteroids (size 2)
      - {min_idx: 2, max_idx: 5, size: 2}
      # Small asteroids (idx 5+) don't split
      - {min_idx: 5, size: 1}
```

#### Weapons

```rhai
// Fire weapon with catalog-defined projectile rules
world.try_fire_weapon("asteroids.ship", ship_id, {
  // Optional overrides (all fields have catalog defaults)
  max_bullets: 8,
  bullet_ttl_ms: 900,
  cooldown_ms: 120
});
```

**Catalog format:**

```yaml
# mods/asteroids/catalogs/weapons.yaml
weapons:
  asteroids.ship:
    max_bullets: 8           # Max bullets on screen
    bullet_kind: "bullet"    # Prefab to spawn
    bullet_ttl_ms: 900       # Bullet lifetime (ms)
    cooldown_ms: 120         # Fire rate (ms between shots)
    cooldown_name: "fire"    # State key for cooldown tracking
    spawn_offset: 20.0       # Distance from ship center
    speed_scale: 60.0        # Bullet speed multiplier
```

#### Emitters

```rhai
// Emit particles (smoke, sparks, explosion, etc.)
fx.emit("asteroids.ship_thrust_smoke", {
  x: ship_x,
  y: ship_y,
  vx: ship_vx,
  vy: ship_vy
});
```

**Catalog format:**

```yaml
# mods/asteroids/catalogs/emitters.yaml
emitters:
  asteroids.ship_thrust_smoke:
    max_count: 40            # Max particles in pool
    cooldown_name: "smoke"   # State key for spawn throttle
    cooldown_ms: 48          # Emit every Nms
    spawn_offset: 6.0        # Distance from entity
    backward_speed: 0.35     # Relative speed to entity
    ttl_ms: 520              # Particle lifetime (ms)
    radius: 3                # Visual particle size
    velocity_scale: 60.0     # Speed multiplier
```

### Backward Compatibility

All gameplay functions (spawn_prefab, try_fire_weapon, emit, etc.) retain
hardcoded fallbacks. If a catalog entry is not found, the function falls back
to its hardcoded Rust implementation. This allows gradual migration from
hardcoded gameplay to catalog-driven gameplay.

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
| Images       | assets/images/     | image_loader.rs      | PNG, GIF (animated), static    |
| Fonts        | assets/fonts/      | font_loader.rs       | Manifest-based, generic:* built-in |
| OBJ meshes   | assets/3d/ or scenes/ | obj_loader.rs     | Wavefront OBJ + MTL           |
| YAML prefabs | objects/, layers/  | engine-authoring     | Reusable authored resources    |

`mod.yaml` can define `terminal.default_font`. Then any text sprite using
`font: "default"` resolves to that spec; if unset, engine fallback generic is used.

Audio authoring is mod-root based:
- `audio/sfx.yaml` defines semantic events such as `ui.menu.select`
- `audio/songs/*.yml` defines sequenced tracks/patterns
- `audio/synth/*.yml` defines generated cues; the engine synthesizes these into in-memory buffers at startup

---

## Sprite Types

### Core Types

| Type  | Purpose             | Key Fields                                      | Asset-backed? |
|-------|---------------------|-------------------------------------------------|---------------|
| text  | Terminal/raster text | content, font, fg, bg, reveal_ms, glow          | Only with named fonts |
| image | Display image       | source, width, height, stretch-to-area           | Yes           |
| obj   | 3D mesh render      | source, scale, yaw/pitch/roll, surface-mode      | Yes           |
| grid  | Layout container    | columns, rows, gap-x/y, children                 | No            |
| flex  | Stack container     | direction, gap, children                         | No            |

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
| PostFX      | postfx (ordered list of post-processing passes)           |
| UI          | ui.enabled, ui.persist, ui.theme, ui.focus-order          |
| Routing     | next, menu-options (each with `to`)                       |
| Input       | input profiles (terminal-shell, menu, custom)             |
| Prerender   | prerender hooks                                           |

---

## PostFX Pipeline

PostFX passes execute after the compositor and before terminal flush.
Order matters — passes apply sequentially to the composited buffer.

| Pass            | Purpose                           |
|-----------------|-----------------------------------|
| crt-underlay    | Soft glow under content           |
| crt-distort     | Tube curvature + margins          |
| crt-scan-glitch | Scanline sweep + chroma glitch    |
| crt-ruby        | Ruby tint + edge reveal           |
| terminal-crt    | Legacy alias                      |

---

## OBJ Lighting

Scenes can define directional lights (primary + secondary), point lights, and
cel shading for 3D objects.

### Light Types

| Type        | Fields                        | Behavior                        |
|-------------|-------------------------------|---------------------------------|
| Directional | direction, color, intensity   | Infinite parallel rays          |
| Point       | position, color, radius       | Orbit or snap animation         |
| Cel shading | steps, edge-threshold         | Posterized shading bands        |

### Point Light Animation

| Mode     | Field    | Behavior                                      |
|----------|----------|-----------------------------------------------|
| Snap     | snap-hz  | Instant position jumps (deterministic hash)   |
| Orbit    | orbit-hz | Smooth continuous rotation                    |
| Static   | (none)   | Fixed position                                |

Priority: snap > orbit > static. When `snap-hz` is set, `orbit-hz` is ignored.

---

## Terminal HUD Authoring

### Window

`type: window` compiles to a panel with three slots: title, body, footer.
Slot layout respects the active font height for vertical sizing.

### Terminal Input

`type: terminal-input` is a specialized window for interactive prompts.

### Shell Input Profile

The `input.terminal-shell` section binds a shell prompt to sprites:

| Field           | Purpose                                    |
|-----------------|--------------------------------------------|
| prompt-sprite-id | Sprite displaying the prompt text         |
| output-sprite-id | Sprite displaying command output          |
| prompt-panel-id  | Panel containing the prompt               |
| prompt-wrap      | Enable line wrapping in prompt            |
| prompt-auto-grow | Panel grows with input length             |

In scripted mode the engine skips built-in commands entirely; Rhai handles
all input processing and output rendering.

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

### Scope Variables

| Variable   | Contents                                  |
|------------|-------------------------------------------|
| menu.*     | Menu state (index, items, selection)      |
| time.*     | Elapsed time, delta, stage progress       |
| params     | Effect/behavior parameters                |
| regions    | Named regions from layout                 |
| objects    | Object instances in the scene             |
| state      | Persistent key-value state                |
| ui         | UI state (focus, visibility)              |
| game       | Global game state                         |
| level      | Active level payload + level catalog      |
| world      | Gameplay entity world                     |
| key        | Current key event                         |
| collisions | Collision hits for the current frame      |

Gameplay helpers on `world` (component-backed):
- `world.set_transform(id, x, y, heading)` and `world.transform(id)`
- `world.set_physics(id, vx, vy, ax, ay, drag, max_speed)` and `world.physics(id)`
- `world.set_collider_circle(id, radius, layer, mask)`
- `world.set_lifetime(id, ttl_ms)`
- `world.set_visual(id, visual_id)` to bind scene runtime target for cleanup on entity expiration
- `world.collision_enters/stays/exits(kind_a, kind_b)` → kind-filtered, named-field maps
- `world.enable_wrap_bounds(id)` / `world.set_world_bounds(...)` — toroidal wrap
- `world.rand_i(min, max)` — engine-managed RNG
- `world.any_alive(kind)` — sugar for count > 0
- `world.distance(a, b)` — distance between entity transforms

### Commands

Scripts emit commands to mutate the scene:

- Visibility: show/hide sprites and layers
- Set-text: update sprite content
- Position: move sprites
- Style: change fg/bg/font/glow

### Object API

```
scene.get(target)              // read a value
scene.set(target, path, value) // write a value
scene.set_vector(id, points, fg, bg) // set all vector props at once
scene.set_visible(id, bool)    // sugar for set(id,"vector.visible",bool)
scene.batch(id, map)           // set multiple props: #{fg:.., bg:.., points:..}
scene.spawn_object(template, target)   // clone a scene object/layer template at runtime
scene.despawn_object(target)           // soft-despawn a scene object/layer
audio.cue(cue_id)              // play direct cue id (asset stem)
audio.cue(cue_id, volume)      // play direct cue id with volume scale
audio.event(event_id)          // play semantic sfx event from audio/sfx.yaml
audio.event(event_id, gain)    // semantic event with gain scale
audio.play_song(song_id)       // start sequenced song from audio/songs/*.yml
audio.stop_song()              // stop currently active sequenced song

game.get(path)                 // read global game state
game.set(path, value)          // write global game state
game.get_i(path, fallback)     // typed int getter
game.get_s(path, fallback)     // typed string getter
game.get_b(path, fallback)     // typed bool getter
game.get_f(path, fallback)     // typed float getter
game.jump(scene_id)            // scene transition
persist.get(path)              // read on-disk persistent state
persist.set(path, value)       // write on-disk persistent state
level.current()                // active level id
level.ids()                    // available level ids
level.select(level_id)         // switch active level
level.get(path)                // read active level payload

world.spawn_visual(kind, template, data)  // atomic: create entity + visual + binding
world.entity(id)               // entity handle API for cleaner repeated access
world.query_kind(kind)         // find ids by kind
world.query_tag(tag)           // find ids by tag

// entity handle API
entity.id()                    // numeric id
entity.exists()                // check entity existence
entity.get(path)               // read path (JSON pointer)
entity.get_i(path, fallback)   // read integer with fallback
entity.get_f(path, fallback)   // read float with fallback
entity.get_s(path, fallback)   // read string with fallback
entity.get_b(path, fallback)   // read bool with fallback
entity.flag(name)              // sugar: get_b("/name", false)
entity.set_flag(name, bool)    // sugar: set("/name", val)
entity.set(path, value)        // write path
entity.set_many(map)           // bulk write
entity.data()                  // full JSON data map
entity.despawn()               // despawn + auto-clean all bound visuals

// Input
input.bind_action(name, keys)  // register named action binding
input.action_down(name)        // query named action

// Key constants for bind_action
// KEY_LEFT, KEY_RIGHT, KEY_UP, KEY_DOWN, KEY_SPACE, KEY_ESC, KEY_ENTER, KEY_F1..F12
input.bind_action("thrust", [KEY_UP, "w"]);
input.bind_action("fire",   [KEY_SPACE, "f"]);

// Math / geometry helpers
abs_i(v)                       // absolute value for integers
sign_i(v, fallback)            // sign with fallback when v == 0
clamp_i(v, min_v, max_v)       // clamp integer
clamp_f(v, min_v, max_v)       // clamp float
to_i(v) / to_float(v)          // type conversions
sin32(idx)                     // 32-step integer sine lookup (-1024..1024)
unit_vec32(heading)            // #{x, y} unit vector for heading 0-31
asteroid_points(shape, size)   // asteroid polygon points
rotate_points(points, heading) // rotate a point array around 0,0 using 32-step heading
asteroid_radius(size)          // helper radius by asteroid size tier
asteroid_score(size)           // score value by asteroid size tier
```

Mod-level named behaviors can live in `behaviors/*.yml` and reference external Rhai:

```yaml
kind: behavior
name: asteroids-game-loop
src: ./asteroids-game-loop.rhai
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
5. Validate   timeline checks (debug mode)
       |
6. Execute    lifecycle -> input -> compositor -> postfx -> render
```

---

## Author Checklist

1. Every YAML file has a correct `$schema` reference.
2. `next` and each `menu-options[].to` point to existing scenes.
3. All `ref` / `use` references resolve to valid targets.
4. `./refresh-schemas.sh` and `cargo run -p schema-gen -- --all-mods --check` pass.
5. Sprite timing falls within scene duration.
6. A smoke run (`cargo run -p app`) starts without compile errors.
7. `cargo run -p app -- --mod-source=mods/<mod> --check-scenes` reports zero warnings before merge.

---

## Daily Workflow

1. Edit YAML files under `mods/<mod>/`.
2. Run `./refresh-schemas.sh` to regenerate schemas.
3. Continue authoring — editor completions reflect the updated schemas.
4. Run `cargo run -p schema-gen -- --all-mods --check` before merge.
