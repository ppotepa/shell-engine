# Scripting API Reference

This document describes the complete Rhai scripting API available to scene behaviors.
Scripts run every frame during `on_idle` (or once during `on_enter`/`on_leave`).

All types and function names listed here are exact Rhai identifiers.

---

## Script Structure

Treat `main.rhai` as a thin entrypoint. Put reusable domain logic under
`mods/<mod>/scripts/` and import modules from there. Import ids are resolved
relative to that directory, so `import "vehicle/state" as state;` loads
`mods/<mod>/scripts/vehicle/state.rhai`.

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

Authoring rules:

- Prefer one owned state object, usually `local.state`.
- Keep raw `local["..."]` access inside dedicated `state*.rhai` /
  `handoff*.rhai` modules instead of scattering keys through entrypoints.
- Avoid `if type_of(local) == "()" { ... }` in entrypoints; hide bootstrap in a
  helper module.
- Use backtick templates for multiline strings: `` `Hello ${name}` `` — never
  `"...\n..."`.

---

## Scope Variables

Every frame, the engine injects a canonical `runtime` root plus frame/timing
values and convenience aliases. Some domains are still reached through those
aliases in current builds while the runtime root surface is being aligned.

| Variable           | Type              | Description                                        |
|--------------------|-------------------|----------------------------------------------------|
| `runtime`          | `RuntimeApi`      | Canonical root namespace for scene/world/services/stores |
| `scene`            | `SceneApi`        | Concise scene-root shorthand; prefer the runtime-root mental model |
| `world`            | `WorldApi`        | Gameplay shorthand; canonical ownership lives under `runtime.world` |
| `game`             | `GameApi`         | Session store shorthand; conceptually under `runtime.stores` |
| `level`            | `LevelApi`        | Level store shorthand; conceptually under `runtime.stores` |
| `persist`          | `PersistApi`      | Persistent store shorthand; conceptually under `runtime.stores` |
| `input`            | `InputApi`        | Input/action queries; service-domain alias |
| `audio`            | `AudioApi`        | Sound cue playback and music; service-domain alias |
| `effects`          | `EffectsApi`      | Screen shake and post-FX trigger; service-domain alias |
| `collision`        | `CollisionApi`    | Structured collision event queries; service-domain alias |
| `ui`               | `UiApi`           | TUI input widget queries; service-domain alias |
| `terminal`         | `TerminalApi`     | Terminal shell output (text push/clear); service-domain alias |
| `palette`          | `PaletteApi`      | Active colour palette and particle ramps; service-domain alias |
| `diag`             | `DebugApi`        | Debug/log diagnostics output; service-domain alias |
| `time`             | `TimeApi`         | Elapsed time, stage name                           |
| `local`            | `Dynamic`         | Per-script frame-to-frame state slot; prefer a single owned `local.state` object |
| `frame_ms`         | `int`             | Actual elapsed time for this frame (milliseconds)  |
| `scene_elapsed_ms` | `int`             | Total elapsed ms since scene start                 |
| `stage_elapsed_ms` | `int`             | Total elapsed ms since current stage start         |

This reference intentionally omits older compatibility maps such as `objects`,
`state`, `menu`, and `key` from the primary authoring surface.

---

## Handle vs Snapshot Model

The scripting surface now separates live handles from snapshot reads.

- `runtime.scene.objects.find(target)` returns a live scene-object handle
- `scene.object(target)` returns the same live scene-object handle through the
  root-scene shorthand
- `scene.inspect(target)` and `scene.region(target)` return snapshot maps
- `world.objects.find(target)` returns a live gameplay lookup handle
- `world.entity(id)` returns the richer gameplay entity handle for transform,
  physics, controller, cooldown, status, and other component helpers

Use live handles when same-frame writes should remain visible to later reads in
the same script. Use snapshot helpers when the script needs a stable read-only
view of the last published runtime state. `scene.object(target)` is the concise
root-scene shorthand for the same live handle type.

This reference intentionally omits older path-based scene compatibility helpers
from the primary flow. If they still appear in older mods, treat them as
migration-only syntax rather than as the authoring target.

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
| `tags`             | `[]`    | Tags to add atomically after spawn (avoids separate `world.tag_add` calls) |

### Query

```rhai
world.exists(id)              // → bool
world.count()                 // → int  Total entity count
world.count_kind(kind)        // → int  Count by kind string (O(1))
world.count_tag(tag)          // → int  Count by tag (O(1))
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

### Spatial Queries

```rhai
world.query_circle(x, y, radius)              // → []   All entity ids within circular radius
world.query_rect(x, y, w, h)                  // → []   All entity ids in axis-aligned box
world.query_nearest(x, y, max_dist)           // → int  Closest entity id within max_dist, or 0
world.query_nearest_kind(kind, x, y, max_dist) // → int  Closest entity of kind within max_dist, or 0
```

**Example: Find threats near the ship**

```rhai
let ship_id = world.first_kind("ship");
if ship_id > 0 {
    let ship_t = world.transform(ship_id);
    if ship_t != () {
        // All entities within 100px
        let nearby = world.query_circle(ship_t.x, ship_t.y, 100.0);
        
        // Filter to just asteroids
        let threats = nearby.filter(|id| world.kind(id) == "asteroid");
        
        // Or find the closest asteroid directly
        let closest = world.query_nearest_kind("asteroid", ship_t.x, ship_t.y, 200.0);
    }
}
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

### World Object Lookup

`world.objects` is the lookup-oriented gameplay object registry. It resolves
either a numeric entity id or a bound visual/runtime target back to the live
gameplay object.

```rhai
let ship = world.objects.find("ship-shadow"); // bound visual id
let also_ship = world.objects.find(1);        // numeric entity id
ship.exists()                 // → bool
ship.id()                     // → int
ship.kind()                   // → str
ship.tags()                   // → []
ship.inspect()                // → #{} snapshot-ish identity/data map
ship.get("hp")                // → Dynamic
ship.set("hp", 42)            // → bool
```

Use `world.objects` to discover or relink gameplay objects from visuals/tags.
Use `world.entity(id)` when the script needs the richer gameplay entity API.

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
world.world_width()           // → float  Width scalar (max_x - min_x); cheaper than world_bounds()
world.world_height()          // → float  Height scalar (max_y - min_y); cheaper than world_bounds()
world.enable_wrap(id, min_x, min_y, max_x, max_y)   // Enable per-entity wrapping
world.disable_wrap(id)        // Disable per-entity wrapping
```

### Camera / Viewport

```rhai
world.set_camera(x, y)        // Shift viewport so world-pos (x, y) maps to screen top-left.
                                // Call each frame: world.set_camera(ship_x - 320.0, ship_y - 180.0)
                                // Screen-space layers are NOT affected — they stay fixed.
                                // Camera resets to (0,0) on scene transition.
world.set_camera_3d_look_at(eye_x, eye_y, eye_z, target_x, target_y, target_z)
                                // Update the shared scene 3D camera. OBJ / scene3_d sprites use it
                                // only when authored with camera-source: scene.
world.set_camera_3d_up(up_x, up_y, up_z)
                                // Override the shared scene 3D up vector for banking / cockpit roll.
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
| `color_ramp`  | `[]`   | No       | Per-particle lifetime colour ramp override |
| `radius_max`  | int    | No       | Lifetime ramp start radius (fresh particle) |
| `radius_min`  | int    | No       | Lifetime ramp end radius (old particle)  |

Anchor and direction precedence:
- Anchor: args `local_x/local_y` → catalog `local_x/local_y` → catalog edge interpolation → legacy `spawn_offset/side_offset`.
- Direction: args `emission_local_x/y` → catalog `emission_local_x/y` → default owner backward axis.
- Final emission direction applies catalog `emission_angle` then args `spread` (both radians).

Emitter catalogs can also contribute runtime-only particle behavior:

- `thread_mode`: `light`, `physics`, or `gravity`
- `collision` / `collision_mask`
- `gravity_scale`
- `gravity_mode`: `flat` or `orbital`
- `gravity_center_x` / `gravity_center_y` / `gravity_constant`
- `palette_ramp`, `color_ramp`, `radius_max`, `radius_min`

### Randomness

```rhai
world.rand_i(min, max)   // → int   Deterministic seeded RNG (inclusive range)
world.rand_seed(seed)    // Set deterministic RNG seed
rand()                   // → float  [0.0, 1.0) — fast thread-local Xorshift (NOT seeded)
```

### Arcade Controller (World Level)

> **These world-level methods have been removed.** Use the entity-level API instead (`e.set_turn`, `e.set_thrust`, `e.heading_vector`).

### Angular Body

Smooth rotation with per-entity angular velocity and auto-brake.

```rhai
world.angular_body_attach(id, #{
    accel: 5.5,        // angular acceleration (rad/s²)
    max: 7.0,          // max angular velocity (rad/s)
    deadband: 0.10,    // auto-brake deadband (rad/s)
    auto_brake: true,  // brake toward zero when input is 0
    angular_vel: 0.0   // initial angular velocity (rad/s)
})                                  // → bool

world.set_angular_input(id, turn)   // → bool  Set turn input −1.0…+1.0 for this frame
world.angular_vel(id)               // → float  Current angular velocity (rad/s)
```

The `angular_body_system` runs before physics integration each tick — no manual update needed.

### Linear Brake

Smooth velocity damping with per-entity deceleration and auto-brake.

```rhai
world.linear_brake_attach(id, #{
    decel: 45.0,       // deceleration (world units/s²)
    deadband: 2.0,     // stop completely below this speed
    auto_brake: true   // brake automatically when not thrusting
})                                          // → bool

world.set_linear_brake_active(id, true)     // → bool  Suppress auto-brake this frame
                                            //         (call each frame while thrusting)
```

`linear_brake_system` runs before physics; not calling `set_linear_brake_active` on a frame allows braking to apply.

### ThrusterRamp

Engine-managed ramp timing component. Tracks how long thrust/brake inputs have been active and outputs normalised 0–1 factors that scripts can read to drive VFX emitters. All state management moves out of Rhai into Rust — scripts just read outputs.

Requires the entity to also have `ArcadeController`, `AngularBody`, `LinearBrake`, and `PhysicsBody2D`.

```rhai
// Attach at spawn (all fields optional, shown with defaults):
world.thruster_ramp_attach(id, #{
    thrust_delay_ms:       8.0,    // ms before thrust emission starts
    thrust_ramp_ms:        12.0,   // ms to reach full thrust intensity
    no_input_threshold_ms: 30.0,   // ms idle before linear auto-brake begins
    rot_factor_max_vel:    7.0,    // rad/s that maps to rot_factor=1.0
    burst_speed_threshold: 15.0,   // px/s below which settling bursts trigger
    burst_wave_interval_ms: 150.0, // ms between burst waves
    burst_wave_count:      3,      // total burst wave count
    rot_deadband:          0.10,   // rad/s below which entity is "stopped rotating"
    move_deadband:         2.5,    // px/s below which entity is "stopped moving"
})                                 // → bool

// Read per-frame outputs (call once, read all factors):
let ramp = world.thruster_ramp(id);  // → #{...} or #{} if not attached
ramp["thrust_factor"]       // float 0–1: thrust intensity ramp
ramp["rot_factor"]          // float 0–1: rotation intensity (from angular_vel)
ramp["brake_factor"]        // float 0–1: auto-brake intensity ramp
ramp["brake_phase"]         // string: "idle"|"rotation"|"linear"|"stopped"|"thrusting"
ramp["final_burst_fired"]   // bool: true for exactly one frame when burst fires
ramp["final_burst_wave"]    // int: which wave (0..burst_wave_count)
ramp["thrust_ignition_ms"]  // float: raw ignition accumulator (for heat curves etc.)

world.thruster_ramp_detach(id)  // → bool  Remove component
```

`thruster_ramp_system` runs after `angular_body_system` + `linear_brake_system` and before the behavior (script) system, so outputs always reflect the current frame's physics state.

### Heading-Relative Helpers

```rhai
// Decompose velocity into heading-relative components
let hd = world.heading_drift(id);   // → #{fwd, right, drift, speed}
// fwd:   forward velocity (+ = along heading, − = backward)
// right: lateral velocity (+ = drifting clockwise, − = counter-clockwise)
// drift: |right| / speed, normalised 0–1 (0 = perfectly aligned)
// speed: total speed magnitude

// Spawn a prefab along owner heading direction (removes sin/cos from scripts)
let id = world.spawn_from_heading(owner_id, "prefab-name", #{
    speed: 280.0,             // projectile speed (world units/s)
    offset: 17.0,             // forward offset from owner origin
    inherit_velocity: true,   // add owner velocity to projectile
    ttl_ms: 1400              // forwarded to spawn_prefab
})                            // → int  Spawned entity id (0 on failure)
```

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

### Physics Helpers

```rhai
// Instant velocity changes (impulses)
e.apply_impulse(vx, vy)            // → bool  Add velocity instantly (explosions, knockback)

// Velocity queries
e.velocity_magnitude()             // → float  Speed scalar (√(vx² + vy²))
e.velocity_angle()                 // → float  Direction in radians (atan2(vy, vx))

// Polar velocity control
e.set_velocity_polar(speed, angle) // → bool  Set velocity from speed + angle (radians)
```

**Example: Asteroid debris scatter**

```rhai
fn split_asteroid(parent_id, world) {
    let parent_phys = world.physics(parent_id);
    if parent_phys == () { return; }
    
    for i in 0..2 {
        let debris_id = world.spawn_prefab("asteroid-small", #{
            x: 100.0, y: 100.0,
            vx: parent_phys.vx * 0.5,  // Inherit 50% parent velocity
            vy: parent_phys.vy * 0.5
        });
        
        if debris_id > 0 {
            // Apply random scatter impulse
            let scatter = (rand() - 0.5) * 80.0;
            world.entity(debris_id).apply_impulse(scatter, scatter);
        }
    }
}
```

**Example: Speed limit with direction preservation**

```rhai
let speed = world.entity(ship_id).velocity_magnitude();
if speed > MAX_SPEED {
    let angle = world.entity(ship_id).velocity_angle();
    world.entity(ship_id).set_velocity_polar(MAX_SPEED, angle);
}
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

Resolve live scene handles through the runtime registry, then use snapshots or
typed mutations only where they fit better than direct handle access.

`runtime.scene.objects` is the primary discovery API for live scene handles.
`scene.object(...)` is the concise root-scene live-handle shorthand.
`scene.inspect(...)` and `scene.region(...)` remain the snapshot reads.

### Runtime Scene Handles

```rhai
let hud = runtime.scene.objects.find("hud-score");
hud.get("text.content")             // → Dynamic
hud.set("text.content", `${score}`) // → bool

let subtitle = scene.object("hud-subtitle");
subtitle.set("text.content", "Ready")

for object in runtime.scene.objects.all() {
    if object.get("visible") {
        // ...
    }
}
```

### Read

```rhai
let obj = runtime.scene.objects.find("object-id");
let snap = scene.inspect("object-id") // Snapshot map for the resolved object id
let box  = scene.region("object-id")  // Runtime layout box map for the resolved object id
obj.get("path")                     // → Dynamic  Read property
obj.get("position.x")               // float
obj.get("text.content")             // str
obj.get("visible")                  // bool
snap.get("capabilities.text.content") // bool
box.get("width")                    // int
```

`scene.inspect(...)` remains a snapshot surface. Pending live-handle writes made
through `runtime.scene.objects.find(...).set(...)` do not rewrite the snapshot
returned by `inspect(...)` during the same frame.

### Write

```rhai
runtime.scene.objects.find("hud-score").set("text.content", `${score}`)
scene.object("hud-subtitle").set("text.content", "Ready")
runtime.scene.objects.find("player").set("visible", false)
runtime.scene.objects.find("ship").set("position.x", 320.0)
runtime.scene.objects.find("polygon").set("vector.points", pts)
runtime.scene.objects.find("main-planet").set("planet.spin_deg", 18.0)
runtime.scene.objects.find("main-planet").set("planet.sun_dir.x", 0.72)

// Typed mutation request (preferred for camera/render mutations that are not
// naturally expressed as per-object handle writes):
scene.mutate(#{
  type: "set_camera3d",
  kind: "look_at",
  eye: [0.0, 0.0, 6.0],
  look_at: [0.0, 0.0, 0.0]
})
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
| `obj.world.x`      | float   | OBJ world-space X translation (pre-projection)   |
| `obj.world.y`      | float   | OBJ world-space Y translation (pre-projection)   |
| `obj.world.z`      | float   | OBJ world-space Z translation (pre-projection)   |
| `obj.ambient`      | float   | Ambient light level (0.0–0.5)                    |
| `obj.light.x`      | float   | Directional light X (−1.0–1.0)                   |
| `obj.light.y`      | float   | Directional light Y (−1.0–1.0)                   |
| `obj.light.z`      | float   | Directional light Z (−1.0–1.0)                   |
| `obj.rotation-speed` | float | Rotation speed in deg/sec                       |
| `obj.atmo.color`   | str     | Atmosphere rim color name (`"none"` to disable)  |
| `obj.atmo.strength` | float  | Atmosphere rim blend (0.0–1.0)                   |
| `obj.atmo.rim_power` | float | Rim falloff exponent (0.1–16.0)                  |
| `obj.atmo.haze_strength` | float | Haze blend (0.0–1.0)                        |
| `obj.atmo.haze_power` | float | Haze falloff exponent (0.1–8.0)               |
| `world.seed`       | int     | Planet seed (0–9999) — triggers regeneration     |
| `world.ocean_fraction` | float | Ocean coverage (0.01–0.99)                   |
| `world.continent_scale` | float | Continent size (0.5–10)                     |
| `world.continent_warp` | float | Coastline chaos (0–2)                        |
| `world.continent_octaves` | int | Continent detail (1–8)                      |
| `world.mountain_scale` | float | Mountain spacing (1–15)                      |
| `world.mountain_strength` | float | Mountain height (0–1)                     |
| `world.mountain_ridge_octaves` | int | Ridge detail (1–8)                    |
| `world.moisture_scale` | float | Moisture pattern size (0.5–8)                |
| `world.ice_cap_strength` | float | Polar ice intensity (0–3)                  |
| `world.lapse_rate` | float   | Altitude cooling (0–1.5)                         |
| `world.rain_shadow` | float  | Rain shadow strength (0–1)                       |
| `world.displacement_scale` | float | Surface displacement (0–0.6)             |
| `world.subdivisions` | int   | Mesh resolution (32/64/128/256/512)              |
| `world.coloring`   | str     | `"biome"` / `"altitude"` / `"moisture"` / `"none"` |
| `world.base`       | str     | Sphere topology: `"cube"` / `"uv"` / `"tetra"` / `"octa"` / `"icosa"` |
| `planet.spin_deg`  | float   | Planet surface spin angle in degrees             |
| `planet.cloud_spin_deg` | float | Planet primary cloud deck spin angle in degrees |
| `planet.cloud2_spin_deg` | float | Planet secondary cloud breakup spin angle in degrees |
| `planet.observer_altitude_km` | float | Observer altitude hint used for planet atmosphere presentation |
| `planet.sun_dir.x` | float   | Planet sun direction X override                  |
| `planet.sun_dir.y` | float   | Planet sun direction Y override                  |
| `planet.sun_dir.z` | float   | Planet sun direction Z override                  |

### Scene Graph Mutations

```rhai
scene.instantiate("template-id", "new-target-id")  // → bool  Clone a template object
scene.despawn("target-id")                    // → bool  Remove a runtime clone
scene.mutate(#{ ... })                        // → bool  Typed scene/runtime mutation request
```

Older helper shorthands may still exist in transitional code, but they are not
part of the recommended authoring surface.

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

## `diag` — DebugApi

Emits messages to the debug overlay log (open the console with `~` / `` ` `` when
`--debug-feature` is active; `Tab` cycles `Stats -> Logs -> Layout`).

```rhai
diag.info("message")
diag.warn("message")
diag.error("message")
diag.layout_info("hud-score", "layout refreshed")
diag.layout_warn("hud-score", "text clipped")
diag.layout_error("hud-score", "missing font")
```

`diag.layout_*` entries are surfaced in the `Layout` debug overlay view alongside
runtime text layout measurements, cheap overflow/clamp hints, and stale/clean
layout status.

---

## `gui` — GuiApi

Query and control GUI widget state (for scenes with `gui:` blocks).

```rhai
gui.slider_value("id")        // → f64   Current slider value (0.0–max)
gui.toggle_on("id")           // → bool  Whether toggle is on
gui.button_clicked("id")      // → bool  True the frame a button was clicked
gui.has_change()              // → bool  True if any widget value changed this frame
gui.changed_widget()          // → str   Id of the last changed widget (or "")
gui.widget_value("id")        // → f64   Alias for slider_value
gui.widget_hovered("id")      // → bool  True if mouse is over the widget
gui.widget_pressed("id")      // → bool  True if widget is currently pressed
gui.set_widget_value("id", v) // → bool  Programmatically set widget value
gui.set_panel_visible("id", b)// → bool  Show/hide a panel widget
gui.mouse_x                   // → int   Mouse x (output-space pixels)
gui.mouse_y                   // → int   Mouse y (output-space pixels)
gui.mouse_x_f                 // → float Mouse x (f32 precision)
gui.mouse_y_f                 // → float Mouse y (f32 precision)
gui.mouse_left_down           // → bool  True while LMB is held outside any widget
```

Slider handle positioning is automatic at the engine level via `GuiControl::visual_sync()`.
Scripts only need to **read** slider values — no manual `runtime.scene.objects.find("handle").set("position.x", ...)`
required.

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

## `palette` — PaletteApi

Active colour palette for the mod. Palette files live in `palettes/*.yml` and define named colors and particle ramps.

```rhai
palette.get("key")           // → str  Hex color for named key, or "" if missing
palette.color_at(idx)        // → str  Hex color at position idx (YAML declaration order), or "" if out of range
palette.key_at(idx)          // → str  Key name at position idx, or "" if out of range
palette.colors_len()         // → int  Number of named colors in the active palette
palette.color_keys()         // → []   Ordered list of color key names
palette.color_values()       // → []   Ordered list of color hex values
palette.particles("ramp")    // → []   Ordered color array for particle ramp (e.g. "thruster")
palette.version()            // → int  Monotonic counter, incremented on every palette change.
                             //        Cache this in local.pal_ver; re-read colors only when it differs.
palette.name()               // → str  Display name of active palette
palette.id()                 // → str  ID of active palette
palette.set_active("id")     // → bool  Persist active palette choice
palette.cycle()              // → str  Cycle to next palette, return new id
palette.list()               // → []   All available palette ids (in load order)
```

**Palette YAML shape** (`palettes/<id>.yml`):
```yaml
id: teal
name: Tropical Teal
colors:
  bg: "#060620"
  ship: "#5bc0be"
  # ...any mod-defined keys
particles:
  thruster:   ["#FFEECC", "#FFCC88", "#FF9944", "#CC6622", "#662211", "#221108"]
  hot_engine: ["#CCFFFF", "#66FFFF", "#00FFCC", "#00CC99", "#008866", "#004433"]
  brake:      ["#DDDDDD", "#BBBBBB", "#888888", "#555555", "#333333", "#111111"]
  heat_trail: ["#AAFFFF", "#88DDDD", "#66BBBB", "#448888", "#225555", "#112233"]
```

**Emitter catalog integration** — set `palette_ramp: "ramp_name"` on an emitter config to auto-resolve the active palette's ramp at emit time:
```yaml
# mods/my-mod/catalogs/emitters.yaml
my_emitter:
  palette_ramp: "thruster"   # resolved from active palette.particles["thruster"]
  color_ramp: [...]          # static fallback if palette has no matching entry
```
Resolution order: **script `color_ramp` arg > active palette `palette_ramp` entry > static `color_ramp`**.

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

### World Generation

```rhai
planet_last_stats()      // → #{}   Biome coverage from last world:// generation
```

Returns a map with fractional coverage values (0.0–1.0):

| Key | Description |
|-----|-------------|
| `"ocean"` | Deep + shallow ocean combined |
| `"shallow"` | Shallow ocean / coastal only |
| `"desert"` | Desert biome |
| `"grassland"` | Grassland / savanna |
| `"forest"` | Temperate + tropical forest combined |
| `"cold"` | Tundra + snow biomes combined |
| `"mountain"` | Mountain / high altitude |

Returns an empty map if no `world://` mesh has been generated yet.

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
