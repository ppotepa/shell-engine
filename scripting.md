# Rhai Scripting Reference

This document is the practical reference for the Rhai API exposed by
`engine-behavior`. It focuses on the script surface mod authors actually use,
the argument names that matter in real gameplay code, and the gotchas that are
easy to miss when wiring YAML, runtime entities, and behaviors together.

For the implementation source of truth, see:

- `engine-behavior/src/lib.rs`
- `engine-behavior/src/scripting/gameplay.rs`
- `engine-behavior/src/scripting/gameplay_impl.rs`
- `engine-behavior/src/scripting/game.rs`
- `engine-behavior/src/scripting/scene.rs`
- `engine-behavior/src/scripting/audio.rs`

## Runtime Model

Each Rhai behavior runs once per frame with a fixed set of injected variables.
The most important ones are:

| Variable | Type | Purpose |
|---|---|---|
| `time` | `TimeApi` | Stage and timing helpers |
| `scene` | `SceneApi` | Scene object/property mutation |
| `game` | `GameApi` | Persistent per-run game state |
| `level` | `LevelApi` | Active level payload access |
| `world` | `GameplayApi` | Runtime gameplay entities and collisions |
| `input` | `InputApi` | Action bindings and button state |
| `audio` | `AudioApi` | Song and semantic audio events |
| `fx` | `FxApi` | Built-in emitter/effect entrypoints |
| `persist` | `PersistenceApi` | Save/highscore style persistent data |
| `diag` | `DebugApi` | Script-side diagnostics |
| `key` | map | Raw key event for this frame |
| `menu` | map | Scene menu metadata |
| `local` | map | Behavior-local persistent state |

Also available for compatibility:

- `scene_elapsed_ms`
- `stage_elapsed_ms`
- `selected_index`
- `menu_count`
- `regions`
- `objects` (kept for compatibility; prefer `scene.get(...)`)
- `state`
- `collisions`
- `ipc`

## Behavior State Rules

### `local[]` is behavior-local

`local[]` belongs to one behavior instance. If a scene has both
`game-loop.rhai` and `render-sync.rhai`, they do **not** share the same
`local[]` data.

Use `local[]` for script-private frame-to-frame state:

```rhai
if !local.contains("thrust_start") {
    local["thrust_start"] = -1;
}
```

Use `game.set/get` for cross-script coordination:

```rhai
// gameplay script
game.set("/my-mod/player_id", ship_id);

// render script
let ship_id = game.get_i("/my-mod/player_id", 0);
```

This is the preferred way to bridge gameplay and visual sync logic.

## Stage-Based Execution

Scripts are expected to branch by stage:

```rhai
if time.stage == "on_enter" {
    // initialize scene-local state
}

if time.stage == "on_idle" {
    // per-frame logic
}

if time.stage == "on_leave" {
    // cleanup or stop audio
}
```

Available stage values:

- `"on_enter"`
- `"on_idle"`
- `"on_leave"`
- `"done"`

## Time API

### Getters

Use these as properties:

```rhai
let stage = time.stage;
let scene_ms = time.scene_elapsed_ms;
let stage_ms = time.stage_elapsed_ms;
```

### Delta helper

There is **no** `time.dt` property.

Use:

```rhai
let dt = time.delta_ms(220, "/my-mod/last_ms");
```

`delta_ms(max_ms, state_path)`:

- reads the current scene time,
- subtracts the previous value stored at `state_path` in game state,
- clamps the result to `0..max_ms`,
- writes the new timestamp back to the same path.

This is the preferred pattern for script-side frame deltas.

## Game, Level, and Persistence APIs

### `game`

Per-run mutable state shared across scenes and behavior files.

Common helpers:

```rhai
game.set("/ast/score", 0);
let score = game.get_i("/ast/score", 0);
let mode = game.get_s("/ast/mode", "normal");
let alive = game.get_b("/ast/alive", false);
let speed = game.get_f("/ast/speed", 0.0);
game.push("/ast/log", "spawned");
game.jump("my-scene-id");
```

### `level`

Read the currently selected level payload:

```rhai
let thrust = if level.has("/player/thrust_power") {
    to_float(level.get("/player/thrust_power"))
} else {
    120.0
};
```

Useful helpers:

- `level.get(path)`
- `level.set(path, value)`
- `level.has(path)`
- `level.remove(path)`
- `level.push(path, value)`
- `level.select(level_id)`
- `level.current()`

### `persist`

Persistent storage for save data, highscores, or unlocks:

```rhai
persist.push("/asteroids/highscores", #{ score: 1200, wave: 5 });
let has_data = persist.has("/asteroids/highscores");
```

## World API

`world` is the main gameplay surface for runtime entities.

### Spawning

#### `spawn_prefab`

```rhai
let ship_id = world.spawn_prefab("ship", #{
    x: 0,
    y: 0,
    heading: 0,
    cfg: #{
        thrust_power: 120.0,
        max_speed: 270.0,
        turn_step_ms: 25
    },
    invulnerable_ms: 3000
});
```

Important notes:

- `name` is looked up in `catalogs/prefabs.yaml`.
- `args["cfg"]` is merged into the prefab controller config.
- This is the intended path for runtime tuning of
  `TopDownShipController`-backed prefabs.
- `invulnerable_ms` is a special-case convenience override used by the
  gameplay helpers.

#### `spawn_group`

```rhai
world.spawn_group("asteroids.initial", "asteroid");
```

This reads from `catalogs/spawners.yaml -> groups`.

#### `spawn_wave`

```rhai
world.spawn_wave("asteroids.dynamic", #{
    spawn_count: 6,
    ship_x: 0,
    ship_y: 0,
    min_x: -320.0,
    max_x: 320.0,
    min_y: -240.0,
    max_y: 240.0
});
```

The important runtime argument names are:

- `spawn_count`
- `ship_x`
- `ship_y`
- `min_x`
- `max_x`
- `min_y`
- `max_y`

Do not use older placeholder names like `count`, `parent_x`, or `ship_id`
unless the implementation is explicitly updated to support them.

### Collision queries

```rhai
for hit in world.collision_enters("bullet", "asteroid") {
    let asteroid = world.entity(hit["asteroid"]);
    let bullet = world.entity(hit["bullet"]);
    if asteroid.exists() && bullet.exists() {
        // mod-side collision policy goes here
    }
}
```

Useful calls:

- `world.collision_enters(kind_a, kind_b)`
- `world.collision_stays(kind_a, kind_b)`
- `world.collision_exits(kind_a, kind_b)`

The engine intentionally stops at generic collision/event primitives. Mod-shaped
combat flow such as weapon firing, split timing, hit reactions, crack visuals,
or ship smoke should live in shared Rhai modules for that mod.

Asteroids now does this in `mods/asteroids/scripts/asteroids-shared.rhai`,
built on top of generic primitives like:

- `world.spawn_prefab(name, args)`
- `world.entity(id)` + entity/controller/physics methods
- `world.count_kind(kind)`
- `world.rand_seed(seed)` / `world.rand_i(min, max)`
- `audio.event(name, gain)`

### Queries and lifecycle

```rhai
if !world.any_alive("asteroid") {
    // next wave
}

for id in world.query_kind("asteroid") {
    let entity = world.entity(id);
    if entity.exists() {
        // ...
    }
}
```

Common helpers:

- `world.entity(id)`
- `world.query_kind(kind)`
- `world.count_kind(kind)`
- `world.any_alive(kind)`
- `world.exists(id)`
- `world.despawn(id)`
- `world.reset_dynamic_entities()`

### Bounds and wrapping

Use the natural argument order:

```rhai
world.set_world_bounds(-320.0, -240.0, 320.0, 240.0);
```

Order:

```text
min_x, min_y, max_x, max_y
```

Related helpers:

- `world.world_bounds()`
- `world.enable_wrap(id, min_x, max_x, min_y, max_y)`
- `world.disable_wrap(id)`

### RNG and timers

```rhai
world.rand_seed(time.scene_elapsed_ms);
world.after_ms("respawn", 1000);

if world.timer_fired("respawn") {
    // ...
}
```

Useful helpers:

- `world.rand_seed(seed)`
- `world.after_ms(label, delay_ms)`
- `world.timer_fired(label)`
- `world.cancel_timer(label)`

## Entity API

`world.entity(id)` returns a gameplay entity handle.

### Common accessors

```rhai
let ship = world.entity(ship_id);
if ship.exists() {
    let pos = ship.transform();
    let heading = ship.heading();
    let is_invulnerable = ship.status_has("invulnerable");
    let flash_ms = ship.get_i("/flash_ms", 0);
}
```

Common methods:

- `exists()`
- `id()`
- `kind()`
- `tags()`
- `transform()`
- `set_position(x, y)`
- `set_heading(heading)`
- `get(path)`
- `get_i(path, fallback)`
- `get_f(path, fallback)`
- `get_s(path, fallback)`
- `get_b(path, fallback)`
- `set(path, value)`
- `set_many(map)`
- `data()`
- `despawn()`
- `lifetime_remaining()`

### Cooldowns and statuses

```rhai
ship.cooldown_start("fire", 120);
if ship.cooldown_ready("fire") {
    // can shoot
}

ship.status_add("invulnerable", 3000);
```

Helpers:

- `cooldown_start(name, ms)`
- `cooldown_ready(name)`
- `cooldown_remaining(name)`
- `status_add(name, ms)`
- `status_has(name)`
- `status_remaining(name)`

### Ship-controller helpers

```rhai
ship.set_turn(-1);               // -1, 0, 1
ship.set_thrust(true);
let heading = ship.heading();
let v = ship.physics.velocity();
```

Useful methods:

- `attach_ship_controller(config)`
- `set_turn(dir)`
- `set_thrust(on)`
- `heading()`
- `heading_vector()`
- `velocity()`

### Physics property

Entities expose physics through a property-style API:

```rhai
let p = ship.physics;
let v = p.velocity();
p.set_velocity(0.0, -120.0);
p.add_velocity(10.0, 0.0);
p.set_acceleration(0.0, 0.0);
p.set_drag(0.0);
p.set_max_speed(270.0);
```

## Scene API

Use `scene` for authored object and sprite mutation.

```rhai
scene.set("hud-score", "text.content", "SCORE: 10");
scene.set_visible("ship", true);
scene.set_vector("asteroid-12", points, "#aaaaaa", "#333333");
scene.batch("hud-score", #{
    "text.content": "SCORE: 20",
    "props.visible": true
});
```

Common helpers:

- `scene.get(target)`
- `scene.set(target, path, value)`
- `scene.set_visible(target, visible)`
- `scene.set_vector(target, points, fg, bg)`
- `scene.batch(target, props)`
- `scene.spawn(template, target)`
- `scene.despawn(target)`

Remember that scene spawn/despawn operations are queued through behavior
commands. They are not an immediate synchronous mutation surface.

## Input API

Typical flow:

```rhai
input.load_profile("asteroids.default");

if input.action_down("thrust") {
    // thrust
}
```

Common helpers:

- `input.load_profile(profile_id)`
- `input.action_down(action_name)`
- `input.bind_action(action_name, key_name)`

The raw `key` map is still useful for single-frame keys such as `Esc` or `R`.

## Audio API

### Audio

```rhai
audio.play_song("my-song");
audio.stop_song();
audio.event("gameplay.ship.shoot");
audio.cue("menu.move");
```

There is no built-in generic `fx` object anymore. Keep emitter policy and other
mod-specific effects in shared Rhai helpers built on generic spawn/physics/audio
APIs.

## Diagnostics

Use `diag` to surface script-side messages:

```rhai
diag.info("spawned ship");
diag.warn("wave config missing, using fallback");
diag.error("player ship missing");
```

Rhai compile/runtime failures are also surfaced into the debug log buffer by the
engine through `BehaviorCommand::ScriptError`.

## Common Patterns

### Basic scene bootstrap

```rhai
if time.stage == "on_enter" {
    world.reset_dynamic_entities();
    world.set_world_bounds(-320.0, -240.0, 320.0, 240.0);
    input.load_profile("asteroids.default");
    game.set("/ast/score", 0);
}
```

### Cross-script id handoff

```rhai
let ship_id = world.spawn_prefab("ship", #{ cfg: cfg });
game.set("/ast/ship_id", ship_id);
```

### HUD diffing

```rhai
let score = game.get_i("/ast/score", 0);
let prev = if local.contains("hud_score") { local["hud_score"] } else { -1 };
if score != prev {
    scene.set("hud-score", "text.content", `SCORE: ${score}`);
    local["hud_score"] = score;
}
```

## Gotchas

### 1. No shared `local[]` across behavior files

Use `game.set/get` for shared state.

### 2. No `time.dt`

Use `time.delta_ms(max_ms, path)`.

### 3. Use `shot_cooldown_ms` at runtime

The weapon catalog uses `cooldown_ms`; the runtime override is
`shot_cooldown_ms`.

### 4. `spawn_wave` argument names matter

Use `spawn_count`, `ship_x`, `ship_y`, `min_x`, `max_x`, `min_y`, `max_y`.

### 5. Ship smoke emitters are ship-based

Use `ship_id` and optional `thrust_ms`.

### 6. World bounds order is natural

Use `min_x, min_y, max_x, max_y`.

### 7. `spawn_prefab(..., #{ cfg: ... })` is how controller tuning flows in

That is the supported way to override catalog controller config such as
`TopDownShipController` values per level or mode.

## Validation

Useful validation commands while iterating on scripts:

```bash
cargo test -p engine-behavior
cargo run -p app -- --mod-source=mods/my-mod --check-scenes
```

If you change the public scripting contract, update:

- `AUTHORING.md`
- `engine-behavior/README.md`
- `engine-behavior/README.AGENTS.md`
- this file
