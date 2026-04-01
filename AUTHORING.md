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
    spawn_offset: 6.0        # Distance from entity
    backward_speed: 0.35     # Relative speed to entity
    ttl_ms: 520              # Particle lifetime (ms)
    radius: 3                # Visual particle size
    follow_local_x: -6.0     # Optional owner-local follow offset
    follow_local_y: 0.0
    follow_inherit_heading: true
```

### Script State and Cross-Script Communication

`local[]` storage belongs to a single behavior instance. Two Rhai behavior files
attached to the same scene do **not** share the same `local[]` map.

Use persistent game state for cross-script handoff:

```rhai
// game-loop.rhai
game.set("/my-mod/player_id", ship_id);

// render-sync.rhai
let ship_id = game.get_i("/my-mod/player_id", 0);
```

Use `local[]` for frame-to-frame state that is private to one behavior script,
and `game.set/get` when another behavior or scene needs to read it.

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

| Variable    | Type          | Contents                                             |
|-------------|---------------|------------------------------------------------------|
| `time`      | map           | `elapsed_ms`, `delta_ms`, `stage_elapsed_ms`         |
| `params`    | map           | Effect/behavior parameters from YAML                 |
| `regions`   | map           | Named layout regions                                 |
| `objects`   | map           | Scene object instances                               |
| `state`     | dynamic       | Persistent key-value state (JSON pointer paths)      |
| `ui`        | UiApi         | Focus, visibility, submit/change events              |
| `game`      | GameApi       | Global game state + scene transitions                |
| `level`     | LevelApi      | Active level payload + level catalog                 |
| `world`     | WorldApi      | Gameplay entity world (spawn/query/physics)          |
| `collision` | CollisionApi  | Collision event queries for the current frame        |
| `effects`   | EffectsApi    | Runtime-triggerable visual effects (shake, flash)    |
| `audio`     | AudioApi      | SFX events, cues, sequenced songs                    |
| `input`     | InputApi      | Key state + named action bindings                    |
| `scene`     | SceneApi      | Scene object mutations (text, visibility, vector)    |
| `persist`   | PersistApi    | On-disk save state                                   |
| `diag`      | DebugApi      | Debug logging + diagnostics                          |
| `key`       | map           | Current key event (`code`, `char`, `kind`)           |
| `menu`      | map           | Menu state (`index`, `items`, `selection`)           |

### `world.*` — Entity World

Spawn, query, mutate, and despawn gameplay entities.

```rhai
// Spawn
world.spawn_prefab(name, args_map)            // spawn from prefab catalog; returns id
world.spawn_visual(kind, template, data_map)  // atomic: create entity + visual + binding
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

// Transform & physics
world.transform(id)                           // #{x, y, heading} map
world.set_transform(id, x, y, heading)
world.physics(id)                             // #{vx, vy, ax, ay, drag, max_speed}
world.set_physics(id, vx, vy, ax, ay, drag, max_speed)

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
scene.get(target)                           // read a scene object value
scene.set(target, path, value)             // write a value
scene.set_visible(id, bool)                 // show/hide a sprite or layer
scene.set_vector(id, points, fg, bg)        // set all vector props at once
scene.batch(id, map)                        // set multiple props: #{fg:.., bg:.., points:..}
scene.spawn_object(template, target)        // clone a scene object/layer template at runtime
scene.despawn_object(target)                // soft-despawn a scene object/layer
```

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
