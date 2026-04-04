# Scripting API Reference

This document describes the complete Rhai scripting API available to scene behaviors.
Scripts run every frame during `on_idle` (or once during `on_enter`/`on_leave`).

All types and function names listed here are exact Rhai identifiers.

---

## Script Structure

Scripts are plain Rhai files. Functions defined at the top are hoisted. The
script body runs each frame. A map returned from the top-level body is treated
as the `local` state and is passed back in on the next frame.

```rhai
// Functions are hoisted — safe to define before use
fn helper(x) { x * 2 }

// Guard: local is () until first run
if type_of(local) == "()" { local = #{}; }

// One-time init
if !(local.initialized ?? false) {
    local.counter = 0;
    local.initialized = true;
}

local.counter += 1;

// Return updated state — will be `local` next frame
#{ state: local }
```

> **Multiline strings**: use backtick templates `` `Hello ${name}` `` — never `"...\n..."`.

---

## Scope Variables

Every frame, the engine injects the following variables into the script scope:

| Variable    | Type              | Description                                        |
|-------------|-------------------|----------------------------------------------------|
| `world`     | `WorldApi`        | Entity lifecycle, queries, physics, collisions     |
| `collision` | `CollisionApi`    | Structured collision event queries                 |
| `scene`     | `SceneApi`        | Scene graph property reads and writes              |
| `input`     | `InputApi`        | Key/action queries and action binding              |
| `audio`     | `AudioApi`        | Sound cue playback and music                       |
| `effects`   | `EffectsApi`      | Screen shake and post-FX trigger                   |
| `game`      | `GameApi`         | Cross-scene state store + scene transitions        |
| `level`     | `LevelApi`        | Level list management                              |
| `time`      | `TimeApi`         | Elapsed time, stage name                           |
| `persist`   | `PersistApi`      | Disk-backed persistent key-value store             |
| `debug`     | `DebugApi`        | Debug log output                                   |
| `ui`        | `UiApi`           | TUI input widget queries                           |
| `terminal`  | `TerminalApi`     | Terminal shell output (text push/clear)            |
| `local`     | `Dynamic`         | Per-script frame-to-frame state (`#{}` or `()`)   |

---

## `world` — Entity World

### Lifecycle & Spawning

```rhai
world.spawn(kind, payload)              // → int  Spawn raw entity; payload: #{} or #{ key: val }
world.spawn_visual(kind, template, #{}) // → int  Spawn with scene clone (see spawn args below)
world.spawn_prefab(name, #{})           // → int  Spawn from catalog prefab
world.spawn_child(parent_id, kind, template, #{}) // → int  Spawn entity parented to another
world.spawn_batch([#{...}, ...])        // → []   Bulk spawn; returns array of ids
world.spawn_group(group_name, prefab)   // → []   Spawn a named group of prefabs

world.despawn(id)                       // → bool Remove entity by id
world.despawn_children(parent_id)       // Remove all children of entity
world.reset_dynamic_entities()          // Despawn all non-persistent entities (scene reset)
world.clear()                           // Remove ALL entities
```

**`spawn_visual` data map keys** (all optional):

| Key                | Type    | Description                                     |
|--------------------|---------|-------------------------------------------------|
| `x`, `y`           | float   | Initial world position                          |
| `heading`          | float   | Initial heading in radians                      |
| `vx`, `vy`         | float   | Initial velocity (creates physics body)         |
| `ax`, `ay`         | float   | Initial acceleration                            |
| `drag`             | float   | Linear drag coefficient                         |
| `max_speed`        | float   | Speed cap (0 = unlimited)                       |
| `collider_radius`  | float   | Attach a circle collider                        |
| `collider_polygon` | `[]`    | Attach a polygon collider (array of `[x,y]`)   |
| `collider_layer`   | int     | Collision layer bitmask                         |
| `collider_mask`    | int     | Collision mask bitmask                          |
| `lifetime_ms`      | int     | Auto-despawn after N milliseconds               |

### Query

```rhai
world.exists(id)              // → bool
world.count()                 // → int  Total entity count
world.count_kind(kind)        // → int  Count by kind string
world.count_tag(tag)          // → int  Count by tag
world.ids()                   // → []   All entity ids
world.query_kind(kind)        // → []   Ids matching kind
world.query_tag(tag)          // → []   Ids matching tag
world.first_kind(kind)        // → int  First id of kind, or 0
world.first_tag(tag)          // → int  First id with tag, or 0
world.any_alive(kind)         // → bool True if any entity of kind exists
world.kind(id)                // → str  Entity kind string
world.tags(id)                // → []   Entity tags array
world.distance(a_id, b_id)    // → float  World-space distance between two entities
world.diagnostic_info()       // → #{}  Debug snapshot (entity counts by kind)
```

### Entity Data

```rhai
world.get(id, "path")         // → Dynamic  JSON-path read
world.set(id, "path", value)  // → bool     JSON-path write
world.has(id, "path")         // → bool
world.remove(id, "path")      // → bool
world.push(id, "path", value) // → bool     Append to JSON array field
world.entity(id)              // → EntityApi  Per-entity method object (see below)
```

### Tags

```rhai
world.tag_add(id, "tag")      // → bool
world.tag_remove(id, "tag")   // → bool
world.tag_has(id, "tag")      // → bool
```

### Transform & Physics

```rhai
world.transform(id)           // → #{ x, y, heading }  or ()
world.set_transform(id, x, y, heading)  // → bool
world.physics(id)             // → #{ vx, vy, ax, ay, drag, max_speed } or ()
world.set_physics(id, vx, vy, ax, ay, drag, max_speed)  // → bool
world.set_lifetime(id, ttl_ms) // → bool
```

### Collision (Legacy Flat API)

> Prefer the `collision.*` namespace for new code.

```rhai
world.collisions()                          // → []  All active collision pairs
world.collisions_between("a", "b")          // → []  All pairs (enter+stay) between kinds
world.collisions_of("kind")                 // → []  All pairs involving kind
world.collision_enters("a", "b")            // → []  Just-entered pairs this frame
world.collision_stays("a", "b")             // → []  Continuing pairs this frame
world.collision_exits("a", "b")             // → []  Just-exited pairs this frame
world.poll_collision_events()               // → []  Raw event log
world.clear_events()                        // Flush event log
```

Each pair map: `#{ kind_a: id_a, kind_b: id_b }` — keys are the actual kind strings.

### World Bounds & Wrap

```rhai
world.set_world_bounds(min_x, min_y, max_x, max_y)  // Set toroidal wrap region
world.world_bounds()          // → #{ min_x, min_y, max_x, max_y }
world.enable_wrap(id, min_x, min_y, max_x, max_y)   // Enable per-entity wrapping
world.disable_wrap(id)        // Disable per-entity wrapping
```

### Timers (World-level)

```rhai
world.after_ms("label", delay_ms)   // Start one-shot timer
world.timer_fired("label")          // → bool  True once when timer fires (clears it)
world.cancel_timer("label")         // → bool  Cancel pending timer
```

### Emitter

```rhai
world.emit("emitter.name", owner_id, #{ ... })  // → int  Emit particle/fx entity
```

**Emitter args map keys**:

| Key           | Type   | Required | Description                              |
|---------------|--------|----------|------------------------------------------|
| `kind`        | str    | Yes      | Entity kind for spawned fx               |
| `template`    | str    | Yes      | Scene template name to clone             |
| `owner_bound` | bool   | No       | If true, despawns with owner             |
| `ttl_ms`      | int    | No       | Lifetime in milliseconds                 |
| `speed`       | float  | No       | Launch speed in world units/sec          |
| `spread`      | float  | No       | Directional offset in radians            |
| `radius`      | int    | No       | Dot radius (visual only)                 |
| `fg`          | str    | No       | Foreground colour name                   |
| `local_x`     | float  | No       | Owner-local anchor X (+right)            |
| `local_y`     | float  | No       | Owner-local anchor Y (+down)             |
| `side_offset` | float  | No       | Extra right-offset (legacy additive)     |
| `emission_local_x` | float | No   | Owner-local base emission X (+right)     |
| `emission_local_y` | float | No   | Owner-local base emission Y (+down)      |

Anchor and direction precedence:
- Anchor: args `local_x/local_y` → catalog `local_x/local_y` → catalog edge interpolation → legacy `spawn_offset/side_offset`.
- Direction: args `emission_local_x/y` → catalog `emission_local_x/y` → default owner backward axis.
- Final emission direction applies catalog `emission_angle` then args `spread` (both radians).

### Randomness

```rhai
world.rand_i(min, max)   // → int   Deterministic seeded RNG (inclusive range)
world.rand_seed(seed)    // Set deterministic RNG seed
rand()                   // → float  [0.0, 1.0) — fast thread-local Xorshift (NOT seeded)
```

### Arcade Controller (World Level)

> **These world-level methods have been removed.** Use the entity-level API instead (`e.set_turn`, `e.set_thrust`, `e.heading_vector`).

---

## `world.entity(id)` — EntityApi

Returns a per-entity API object. All methods are no-ops when the entity does not exist.

### Identity & State

```rhai
let e = world.entity(id);

e.exists()                   // → bool
e.id()                       // → int
e.kind()                     // → str
e.tags()                     // → []
e.despawn()                  // → bool
```

### Data Fields

```rhai
e.get("path")                // → Dynamic
e.get_i("path", fallback)    // → int
e.get_f("path", fallback)    // → float
e.get_s("path", fallback)    // → str
e.get_bool("path", fallback) // → bool
e.set("path", value)         // → bool
e.set_many(#{ key: val })    // → bool  Bulk set
e.data()                     // → #{}   Full data map snapshot
e.flag("name")               // → bool  Boolean flag shorthand (get)
e.set_flag("name", true)     // → bool  Boolean flag shorthand (set)
e.has("path")                // → bool
e.remove("path")             // → bool
e.get_metadata()             // → #{}   Entity metadata snapshot
e.get_components()           // → #{}   Component snapshot
```

### Transform

```rhai
e.transform()                // → #{ x, y, heading }
e.set_position(x, y)         // → bool
e.set_heading(radians)       // → bool  Sets heading, syncs controller if present
e.set_acceleration(ax, ay)   // → bool
e.collider()                 // → #{ shape, radius, layer, mask } or ()
e.lifetime_remaining()       // → int  Milliseconds until expiry (0 = no TTL)
```

### Physics (Sub-API via `.physics`)

Accessed via `e.physics.method()`:

```rhai
e.physics.velocity()                    // → [vx, vy]
e.physics.set_velocity(vx, vy)          // → bool
e.physics.add_velocity(dvx, dvy)        // → bool
e.physics.acceleration()                // → [ax, ay]
e.physics.set_acceleration(ax, ay)      // → bool
e.physics.add_acceleration(dax, day)    // → bool
e.physics.drag()                        // → float
e.physics.set_drag(drag)               // → bool
e.physics.max_speed()                   // → float
e.physics.set_max_speed(max_speed)      // → bool
e.physics.collider()                    // → #{ shape, radius, layer, mask }
e.physics.set_collider_circle(r, layer, mask)  // → bool
e.physics.set_collider_polygon(pts, layer, mask)  // → bool (pts: array of [x,y])
```

### Cooldowns & Status

```rhai
e.cooldown_start("name", ms)    // → bool  Start named cooldown
e.cooldown_ready("name")        // → bool  True when cooldown has expired
e.cooldown_remaining("name")    // → int   Remaining ms, 0 if ready

e.status_add("name", ms)        // → bool  Start named timed status
e.status_has("name")            // → bool
e.status_remaining("name")      // → int   Remaining ms
```

### Arcade Controller (Entity Level)

```rhai
e.attach_controller(#{ ... })       // Attach ArcadeController at runtime
e.set_turn(dir)                     // -1 (left), 0 (none), 1 (right)
e.set_thrust(on)                    // bool
e.heading()                         // → int  Discrete heading step
e.heading_vector()                  // → #{ x, y }  Unit direction vector
```

**`attach_controller` config keys**:

| Key             | Type  | Default | Description              |
|-----------------|-------|---------|--------------------------|
| `turn_step_ms`  | int   | 60      | ms per discrete turn step |
| `thrust_power`  | float | 150.0   | Acceleration magnitude   |
| `max_speed`     | float | 200.0   | Velocity cap             |
| `heading_bits`  | int   | 32      | Discrete heading steps   |

---

## `collision` — CollisionApi

Structured collision event queries. Uses the same underlying collision data as
`world.collision_enters/stays/exits` but with a cleaner API surface.

```rhai
collision.enters("a", "b")    // → []   Pairs that started colliding this frame
collision.stays("a", "b")     // → []   Pairs currently overlapping
collision.exits("a", "b")     // → []   Pairs that separated this frame
collision.enters_of("kind")   // → []   All enter events involving kind
collision.stays_of("kind")    // → []   All stay events involving kind
collision.all_enters()        // → []   Every enter event this frame
collision.count_enters("a","b") // → int  Count of enter pairs
collision.any_enter("a","b")  // → bool
```

Each result map: `#{ kind_a: id, kind_b: id }` with actual kind names as keys.

```rhai
for hit in collision.enters("bullet", "enemy") {
    let bid = hit["bullet"];
    let eid = hit["enemy"];
    world.despawn(bid);
    world.despawn(eid);
}
```

---

## `scene` — SceneApi

Read and write properties on scene objects (sprites, layers) by their authored id or alias.

### Read

```rhai
let obj = scene.get("object-id");   // Returns ScriptObjectApi handle
obj.get("path")                     // → Dynamic  Read property
obj.get("position.x")               // float
obj.get("text.content")             // str
obj.get("visible")                  // bool
```

### Write

```rhai
scene.set("id", "path", value)
scene.set("hud-score", "text.content", `${score}`)
scene.set("player",    "visible", false)
scene.set("ship",      "position.x", 320.0)
scene.set("polygon",   "vector.points", pts)
```

**Writable paths**:

| Path               | Type    | Notes                                             |
|--------------------|---------|---------------------------------------------------|
| `visible`          | bool    | Show/hide the object                              |
| `position.x`       | float   | Horizontal offset                                 |
| `position.y`       | float   | Vertical offset                                   |
| `transform.heading`| float   | Rotation in radians (vector sprites)              |
| `text.content`     | str     | Text sprite body                                  |
| `text.font`        | str     | Font name                                         |
| `style.fg`         | str     | Foreground colour name                            |
| `style.bg`         | str     | Background colour name                            |
| `vector.points`    | `[]`    | Replace polygon points (array of `[x,y]`)        |

### Scene Graph Mutations

```rhai
scene.spawn("template-id", "new-target-id")  // → bool  Clone a template object
scene.despawn("target-id")                    // → bool  Remove a runtime clone
scene.set_visible("id", false)                // Shorthand visibility toggle
scene.set_vector("id", points, fg, bg)        // Set vector polygon + colours
scene.batch("id", #{ path: val, ... })        // Bulk property update
```

---

## `input` — InputApi

### Raw Key Queries

Key codes are strings: `"a"`..`"z"`, `"0"`..`"9"`, `"Enter"`, `"Backspace"`,
`"Tab"`, `"Up"`, `"Down"`, `"Left"`, `"Right"`, `"Esc"`, `"Space"`, `"F1"`..`"F12"`,
`"Shift"`, `"Ctrl"`, `"Alt"`.

```rhai
input.down("Right")            // → bool  Key held this frame
input.just_pressed("Space")    // → bool  Key pressed this frame only
input.any_down()               // → bool  Any key held
input.down_count()             // → int   Number of keys held
```

### Action System

Actions are named logical inputs bound to one or more physical keys.

```rhai
input.action_down("fire")           // → bool  Any bound key held
input.action_just_pressed("jump")   // → bool  Any bound key just pressed
input.bind_action("fire", ["Space", "z"])   // Bind keys to action this session
input.load_profile("default")       // Load action bindings from catalog input profile
```

> Input profiles are defined in `catalogs/input-profiles.yaml` and loaded by name.

---

## `audio` — AudioApi

```rhai
audio.event("cue-name")              // Play SFX cue (volume from catalog)
audio.event("cue-name", 0.8)         // Play with gain override [0.0..1.0]
audio.cue("cue-name")                // Alias for audio.event
audio.cue("cue-name", 1.0)           // With volume override
audio.play_song("song-id")           // Start background music track
audio.stop_song()                    // Stop background music
```

> Audio cue names correspond to keys in `audio/sfx.yaml` → `cues:` section.

---

## `effects` — EffectsApi

Runtime post-FX triggers. Effects are processed by the compositor after rendering.

```rhai
effects.shake(duration_ms, amp_x, amp_y, frequency)
// duration_ms: int   — how long the shake lasts
// amp_x, amp_y: float — peak amplitude in each axis
// frequency: float  — oscillations per second

effects.trigger("effect-name", duration_ms, #{})      // Trigger a named post-FX once
effects.trigger_loop("effect-name", duration_ms, #{}) // Trigger a looping post-FX
```

> `"effect-name"` must match a post-FX effect defined in the scene YAML `postfx:` list.

---

## `time` — TimeApi

```rhai
time.scene_elapsed_ms    // int  — ms since scene became active
time.stage_elapsed_ms    // int  — ms since current stage started
time.stage               // str  — "on_enter" | "on_idle" | "on_leave" | "done"

time.get("scene_elapsed_ms")          // Dynamic equivalent
time.get_i("stage_elapsed_ms", 0)     // int with fallback
time.delta_ms(max_ms, "game.last_tick") // → int  Clamped delta since last call (uses game state)
```

---

## `game` — GameApi

Cross-scene persistent state (lives for the session, cleared on restart).

```rhai
game.get("path")               // → Dynamic
game.get_i("path", fallback)   // → int
game.get_f("path", fallback)   // → float
game.get_s("path", fallback)   // → str
game.get_b("path", fallback)   // → bool
game.set("path", value)        // → bool
game.has("path")               // → bool
game.remove("path")            // → bool
game.push("path", value)       // Append to array field

game.jump("scene-id")          // Transition to another scene immediately
```

---

## `level` — LevelApi

Manage a list of levels defined in the mod catalog.

```rhai
level.ids()             // → []   All level ids
level.current()         // → str  Active level id
level.get("path")       // Read from current level config
level.set("path", val)  // Write to current level config
level.has("path")       // → bool
level.remove("path")    // → bool
level.push("path", val) // Append to array field
level.select("lvl-id")  // Switch active level
```

---

## `persist` — PersistApi

Disk-backed key-value store. Survives session restarts.

```rhai
persist.get("path")            // → Dynamic
persist.set("path", value)     // → bool
persist.has("path")            // → bool
persist.remove("path")         // → bool
persist.push("path", value)    // Append to array field
persist.reload()               // Force reload from disk
```

---

## `debug` — DebugApi

Emits messages to the debug overlay log (visible with `~` when `--debug-feature` is active).

```rhai
debug.info("message")
debug.warn("message")
debug.error("message")
```

---

## `ui` — UiApi

Query TUI input widget state (for menu/form scenes with `ui:` blocks).

```rhai
ui.focused_target()     // → str  Id of the currently focused widget
ui.theme()              // → str  Active UI theme name
ui.has_submit()         // → bool  True if a widget submitted this frame
ui.submit_target()      // → str  Id of the submitting widget
ui.submit_text()        // → str  Text submitted
ui.has_change()         // → bool  True if a widget value changed this frame
ui.change_target()      // → str  Id of the changed widget
ui.change_text()        // → str  New text value
ui.flash_message("msg", ttl_ms)  // Show a flash message overlay
```

---

## `terminal` — TerminalApi

For scenes with a `terminal-shell` layer — push/clear terminal output lines.

```rhai
terminal.push("Hello, world!")   // Append a line
terminal.clear()                 // Clear all output
```

---

## Free Functions

These are available globally (not on a namespace object).

### Math

Standard Rhai built-ins (all available): `abs`, `ceil`, `floor`, `round`, `sqrt`,
`sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `exp`, `ln`, `log`, `pow`, `min`, `max`,
`clamp`, `PI`, `E`.

Engine-provided extras:

```rhai
sin32(idx)               // → int  Fixed-point sine: idx in [0..255], result in [-100..100]
to_i(value)              // → int  Safe float→int cast (no panic on NaN/inf)
to_float(value)          // → float  int→float
```

### Geometry

Integer-domain polygon helpers (all inputs/outputs in `i32`):

```rhai
regular_polygon(sides, radius)       // → []  Array of [x,y] points forming a regular polygon
jitter_points(points, amount, seed)  // → []  Randomly displace points (deterministic by seed)
rotate_points(points, step)          // → []  Rotate by discrete step (0..255 maps to 0..2π)
dent_polygon(points, ix, iy, str)   // → []  Push closest vertex toward centroid by str% (0–100)
subtract_polygon(poly_a, poly_b)    // → [[]]  Boolean difference: subtract B from A, returns array of polygons
polygon_area(points)                 // → int  Absolute area of polygon (Shoelace formula)
```

All return arrays of `[x, y]` integer pairs. `subtract_polygon` returns an array of
polygons (each a `[[x,y], ...]` array) — one element for a notch, two or more when
the cut splits the shape, empty when B completely covers A.

**Deformation usage:**
```rhai
// Create crater at impact point and subtract from shape
let crater = regular_polygon(6, 8);
let crater_at = [];
for p in crater { crater_at.push([p[0] + ix, p[1] + iy]); }
let fragments = subtract_polygon(shape, crater_at);
// fragments.len() == 1 → deformed, 2+ → split, 0 → destroyed
```

### Strings & Misc

```rhai
rand()                   // → float  [0.0, 1.0) — thread-local non-seeded RNG
is_blank(str)            // → bool   True if string is empty or whitespace-only
type_of(value)           // → str    Rhai type name (built-in)
```

---

## Type Conventions

| Rhai type   | Notes                                                            |
|-------------|------------------------------------------------------------------|
| `int`       | 64-bit signed integer (`i64`)                                    |
| `float`     | 64-bit float (`f64`)                                             |
| `bool`      | Boolean                                                          |
| `str`       | Immutable string                                                 |
| `()`        | Unit / null — use `type_of(x) == "()"` to check                 |
| `#{ }`      | Object map (key → Dynamic)                                       |
| `[ ]`       | Array                                                            |

**Null checks:**
```rhai
if type_of(value) == "()" { /* value is null */ }
let x = value ?? default_value;   // Null-coalescing
```

---

## Limits & Behaviour

- Scripts run synchronously on the game thread — avoid unbounded loops.
- `local` state is per-behavior-instance (not shared across multiple behaviors on the same scene).
- `game.*` is shared across all scenes and scripts in the same session.
- `persist.*` is shared and disk-backed — write sparingly.
- `rand()` is NOT deterministic — use `world.rand_i()` or `world.rand_seed()` for repeatable results.
- Rhai integer overflow is silent (wraps). Float division by zero returns `inf`.
- Compile errors and runtime Rhai panics emit a `ScriptError` command which is visible in the debug overlay.
