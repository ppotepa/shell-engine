# Shell Quest — Scripting Contract & API Reference

This document is the **canonical reference** for the Rhai scripting system.
Read it top-to-bottom before writing any scripts.

---

## Table of Contents

1. [Architecture Overview](#1-architecture-overview)
2. [Three Data Planes](#2-three-data-planes)
3. [API Reference](#3-api-reference)
4. [Frame System Order](#4-frame-system-order)
5. [Entity Component Model](#5-entity-component-model)
6. [Visual Binding & Sync](#6-visual-binding--sync)
7. [Implementation Status](#7-implementation-status)
8. [Target Script Shape](#8-target-script-shape)

---

## 1. Architecture Overview

Shell Quest is a Rust terminal/SDL2 game engine. Content is authored in YAML (scenes, objects,
effects) and scripted in Rhai. The engine follows a **fixed frame loop** with 17 ordered systems.

### Key files

| File | Role |
|------|------|
| `engine-behavior/src/lib.rs` | Rhai engine init, ALL function registration (~6000 LOC) |
| `engine-game/src/gameplay.rs` | GameplayWorld: entity store, components, JSON path access |
| `engine-game/src/components.rs` | Typed components: Transform2D, PhysicsBody2D, Collider2D, Lifetime, VisualBinding |
| `engine-game/src/collision.rs` | Collision detection: broadphase, narrowphase, layer/mask |
| `engine/src/systems/visual_sync.rs` | Auto-sync Transform2D → scene position |
| `engine/src/systems/visual_binding.rs` | Visual cleanup buffer for despawned entities |
| `engine/src/systems/gameplay.rs` | Physics step + lifetime expiry + auto-visual-despawn |
| `engine/src/game_loop.rs` | Frame system order |

### Mod structure

```
mods/asteroids/
├── mod.yaml                    # Mod metadata
├── scenes/game/scene.yml       # Scene with layers, sprites, object refs, behaviors
├── objects/                    # Visual templates (YAML)
│   ├── asteroid-vector.yml     # Vector sprite: points, fg, visible
│   ├── bullet-vector.yml
│   ├── ship-vector.yml
│   └── smoke-vector.yml
├── behaviors/                  # Rhai scripts
│   ├── asteroids-game-loop.rhai   (887 LOC — logic, physics, spawning, collisions)
│   └── asteroids-render-sync.rhai (204 LOC — copies entity state → scene visuals)
└── scripts/                    # Shared Rhai modules (NEW, A4)
    └── asteroids-shared.rhai
```

---

## 2. Three Data Planes

The engine enforces separation between three data planes. Scripts bridge them.

```
┌─────────────────┐     ┌──────────────────────┐     ┌──────────────────┐
│  AUTHORED SCENE  │     │  GAMEPLAY SIMULATION  │     │   PRESENTATION   │
│  (YAML → Scene)  │     │  (GameplayWorld)      │     │  (Compositor)    │
│                  │     │                       │     │                  │
│  sprites         │◄────│  entities             │────►│  rendered output │
│  layers          │     │  transforms           │     │  terminal/SDL2   │
│  effects         │     │  physics              │     │                  │
│  object templates│     │  colliders            │     │                  │
└─────────────────┘     └──────────────────────┘     └──────────────────┘
     scene.*                   world.*                   (automatic)
```

**Ownership rules:**
- `scene.*` API reads/writes the authored scene (sprite properties, visibility, text, position)
- `world.*` API reads/writes gameplay simulation (entities, transforms, physics, colliders)
- Presentation is automatic (compositor reads scene state)
- **Visual sync** (A3) auto-copies Transform2D → scene position.x/y
- **Visual despawn** (A1) auto-cleans scene objects when entities are despawned

---

## 3. API Reference

### 3.1 Scope Variables (available in every script)

| Variable | Type | Description |
|----------|------|-------------|
| `scene` | SceneApi | Scene manipulation |
| `world` | GameplayApi | Gameplay entity world |
| `input` | InputApi | Keyboard state |
| `audio` | AudioApi | Audio cues, events, songs |
| `game` | GameApi | Game-level persistent state |
| `level` | LevelApi | Level selection & state |
| `persist` | PersistenceApi | Save/load persistent data |
| `terminal` | TerminalApi | Terminal text output |
| `diag` | DebugApi | Debug logging (info/warn/error) |
| `time` | Map | `{scene_elapsed_ms, stage_elapsed_ms, delta_ms}` |
| `menu` | Map | `{selected_index, count, items}` |
| `key` | Map | `{code, ctrl, alt, shift, pressed}` |
| `collisions` | Array | `[{a: entity_id, b: entity_id}, ...]` |
| `ipc` | Map | Sidecar IO: `{output_lines, clear_count, screen_full_lines, custom_events}` |
| `state` | Dynamic | Per-behavior persistent state |

### 3.2 Scene API (`scene.*`)

| Function | Description |
|---|---|
| `scene.get(id)` | Returns ScriptObjectApi snapshot |
| `scene.set(id, path, value)` | Set a scene property |
| `scene.set_vector(id, points, fg, bg)` | Set all 3 vector props at once |
| `scene.set_visible(id, bool)` | Sugar for `set(id,"vector.visible",bool)` |
| `scene.batch(id, map)` | Set multiple properties: `#{fg:.., bg:.., points:..}` |
| `scene.spawn_object(template, id)` | Spawn scene object from template |
| `scene.despawn_object(id)` | Remove scene object |

### 3.3 Gameplay World API (`world.*`)

#### Entity lifecycle
| Function | Returns | Description |
|---|---|---|
| `world.spawn(kind, data)` | INT id | Spawn entity with JSON data |
| `world.spawn_object(kind, data)` | INT id | Spawn with scene visual |
| `world.spawn_visual(kind, template, data)` | INT id | Spawn + scene object + bind |
| `world.exists(id)` | bool | |
| `world.ids()` | array | All live entity IDs |
| `world.entity(id)` | EntityRef | Get typed entity ref |

#### Queries
| Function | Returns | Description |
|---|---|---|
| `world.query_kind(kind)` | array | All live IDs of kind |
| `world.query_tag(tag)` | array | All live IDs with tag |
| `world.count_kind(kind)` | int | |
| `world.first_kind(kind)` | int | First live ID or 0 |
| `world.any_alive(kind)` | bool | Sugar: count_kind > 0 |
| `world.distance(a, b)` | float | Distance between two entity transforms |

#### Bounds & wrap
| Function | Description |
|---|---|
| `world.set_world_bounds(min_x, max_x, min_y, max_y)` | Store global wrap bounds |
| `world.world_bounds()` | Returns `#{min_x, max_x, min_y, max_y}` map |
| `world.enable_wrap(id, min_x, max_x, min_y, max_y)` | Per-entity explicit bounds |
| `world.enable_wrap_bounds(id)` | Use stored global bounds |
| `world.disable_wrap(id)` | Remove wrap |

#### RNG
| Function | Description |
|---|---|
| `world.rand_i(min, max)` | Random int in [min, max], engine-managed seed |
| `world.rand_seed(seed)` | Reset RNG seed |

#### Tags
| Function | Description |
|---|---|
| `world.tag_add(id, tag)` | Add runtime tag to entity |
| `world.tag_remove(id, tag)` | Remove tag |
| `world.tag_has(id, tag)` | Check tag |

#### Components
| Function | Description |
|---|---|
| `world.set_transform(id, x, y, heading)` | |
| `world.set_physics(id, vx, vy, ax, ay, drag, max_speed)` | |
| `world.set_collider_circle(id, radius, layer, mask)` | |
| `world.attach_ship_controller(id, config)` | config: `#{turn_step_ms, thrust_power, max_speed, heading_bits}` |
| `world.set_visual(id, scene_id)` | Bind entity to scene object |
| `world.bind_visual(id, scene_id)` | Add additional visual binding |
| `world.set_lifetime(id, ms)` | Auto-expire entity |

#### Collision events
| Function | Returns | Description |
|---|---|---|
| `world.collision_enters(kind_a, kind_b)` | array of maps | New contacts this frame |
| `world.collision_stays(kind_a, kind_b)` | array | Ongoing contacts |
| `world.collision_exits(kind_a, kind_b)` | array | Ended contacts |

Each hit map: `#{ "kind_a_name": id_a, "kind_b_name": id_b }`

#### Children
| Function | Description |
|---|---|
| `world.spawn_child(parent, kind, template, data)` | Spawn entity attached to parent |
| `world.despawn_children(parent_id)` | Despawn all children |

### 3.4 Entity Ref API (`world.entity(id).*`)

#### Identity & lifecycle
| Method | Returns | Description |
|---|---|---|
| `e.id()` | int | Numeric entity id |
| `e.exists()` | bool | |
| `e.kind()` | string | |
| `e.tags()` | array | Spawned tags |
| `e.despawn()` | bool | Despawn + auto-despawn bound visuals |

#### Transform & physics
| Method | Description |
|---|---|
| `e.transform()` | Returns `#{x, y, heading}` |
| `e.set_position(x, y)` | |
| `e.set_heading(h)` | |
| `e.physics()` | Returns `#{vx, vy, ax, ay, drag, max_speed}` |
| `e.set_velocity(vx, vy)` | |
| `e.set_acceleration(ax, ay)` | |
| `e.collider()` | Returns collider map |
| `e.lifetime_remaining()` | ms remaining |

#### Ship controller (entities with TopDownShipController)
| Method | Description |
|---|---|
| `e.attach_ship_controller(config)` | config: `#{turn_step_ms, thrust_power, max_speed, heading_bits}` |
| `e.set_turn(dir)` | dir: -1/0/1 |
| `e.set_thrust(on)` | bool |
| `e.heading()` | int (0..heading_bits) |
| `e.heading_vector()` | `#{x, y}` unit vector |

#### JSON data
| Method | Description |
|---|---|
| `e.get(path)` | Dynamic — JSON pointer e.g. `"/size"` |
| `e.get_i(path, fallback)` | int |
| `e.get_f(path, fallback)` | float |
| `e.get_s(path, fallback)` | string |
| `e.get_b(path, fallback)` | bool |
| `e.set(path, value)` | |
| `e.set_many(map)` | Bulk write |
| `e.data()` | Full data map |
| `e.flag(name)` | Sugar: `get_b("/name", false)` |
| `e.set_flag(name, bool)` | Sugar: `set("/name", val)` |

#### Timers
| Method | Description |
|---|---|
| `e.cooldown_start(name, ms)` | Start named cooldown |
| `e.cooldown_ready(name)` | bool — elapsed |
| `e.cooldown_remaining(name)` | ms left |
| `e.status_add(name, ms)` | Add named status effect |
| `e.status_has(name)` | bool |
| `e.status_remaining(name)` | ms left |

### 3.5 Other APIs

**Input**:
- `input.down(code)` → bool, `input.any_down()` → bool, `input.down_count()` → int
- `input.bind_action(name, keys)` — register named action binding (e.g. `input.bind_action("turn_left", [KEY_LEFT, "a", "A"])`)
- `input.action_down(name)` → bool — query named action

**Audio**: `audio.cue(name)`, `audio.cue(name, vol)`, `audio.event(name)`, `audio.event(name, gain)`, `audio.play_song(id)`, `audio.stop_song()`

**Game state** (`game.*`):
- `game.get(path)` / `game.set(path, val)` / `game.has(path)` / `game.remove(path)` / `game.push(path, val)`
- `game.get_i(path, fallback)` / `game.get_s(path, fallback)` / `game.get_b(path, fallback)` / `game.get_f(path, fallback)` — typed getters with fallback
- `game.jump(scene_id)` — scene transition

**Level/Persist**: Level shares `get/set/has/remove/push`, adds `select(id)`, `current()`, `ids()`. Persist adds `reload()`.

**Terminal**: `terminal.push(line)`, `terminal.clear()`

**Debug**: `diag.info(msg)`, `diag.warn(msg)`, `diag.error(msg)`

### 3.6 Standalone Functions

| Function | Description |
|---|---|
| `sin32(heading)` | Sine on 0-31 heading scale (returns int*1024) |
| `unit_vec32(heading)` | Returns `#{x, y}` normalised unit vector for heading 0-31 |
| `rotate_points(pts, heading)` | Rotate point array by heading |
| `asteroid_points(shape, size)` | Get asteroid polygon points |
| `asteroid_radius(size)` | Collision radius for size |
| `asteroid_score(size)` | Score value for size |
| `clamp_i(v, min, max)` | Int clamp |
| `clamp_f(v, min, max)` | Float clamp |
| `abs_i(v)` / `sign_i(v, fallback)` | |
| `to_i(v)` / `to_float(v)` | Type conversions |

**Collision helpers**: `poly_hit(polyA, ax, ay, polyB, bx, by)`, `point_in_poly(px, py, poly, ox, oy)`, `segment_poly_hit(x0, y0, x1, y1, poly, ox, oy)`

### 3.7 Rhai Module System

Scripts can import shared modules from `{mod}/scripts/`:
```rhai
import "asteroids-shared" as shared;
shared::wrap_heading32(heading);
```

### 3.8 Constants

#### Key codes
`KEY_LEFT`, `KEY_RIGHT`, `KEY_UP`, `KEY_DOWN`, `KEY_SPACE`, `KEY_ESC`, `KEY_ENTER`, `KEY_BACKSPACE`, `KEY_TAB`, `KEY_F1`–`KEY_F12`

Use with `input.bind_action()`:
```rhai
input.bind_action("turn_left", [KEY_LEFT, "a", "A"]);
input.bind_action("quit",      [KEY_ESC]);
```

#### Collision layer masks
`LAYER_ALL` = 0xFFFF, `LAYER_NONE` = 0

---

## 4. Frame System Order

```
 1. INPUT            — poll keyboard, clear previous frame keys
 2. LIFECYCLE        — process scene transitions, quit events
 3. ANIMATION        — animator_system (stage stepping)
 4. POST-ANIM LIFE   — scene transitions from animation
 5. GAMEPLAY         — physics_step(dt) + lifetime_expiry + auto_visual_despawn
 6. COLLISION        — broadphase + narrowphase → collision buffer
 7. HOT RELOAD       — debug scene reload
 8. ENGINE IO        — sidecar communication
 9. BEHAVIOR         — ██ RHAI SCRIPTS RUN HERE ██
10. VISUAL SYNC      — Transform2D → scene position.x/y auto-copy
11. AUDIO SEQUENCER  — audio cue scheduling
12. AUDIO            — audio playback
13. COMPOSITOR       — layer compositing + sprite rendering
14. POST FX          — effects pipeline
15. RENDERER         — terminal/SDL2 output
16. CLEANUP          — clear events + cleanup_visuals (despawn queued visuals)
17. SLEEP            — frame pacing
```

**Key insight**: Scripts (step 9) see collision results from step 6 and physics state from step 5.
Visual sync (step 10) copies transforms to scene AFTER scripts run. Compositor (step 13) reads
the final scene state.

---

## 5. Entity Component Model

### Engine-managed typed components

These are processed by engine systems each frame:

| Component | Fields | Description |
|---|---|---|
| `Transform2D` | x, y, heading | Position/heading; auto-synced to scene |
| `PhysicsBody2D` | vx, vy, ax, ay, drag, max_speed | Velocity/acceleration/drag integration |
| `Collider2D` | radius, layer, mask | Circle collision detection |
| `TopDownShipController` | turn_step_ms, thrust_power, max_speed, heading_bits | Arcade heading + thrust |
| `Lifetime` | ttl_ms | Auto-expire entity after ms |
| `VisualBinding` | visual_id, additional_visuals | Auto-despawn scene objects on entity despawn |
| `WrapBounds` | min_x, max_x, min_y, max_y | Toroidal space wrap |

### Game-defined data components

Any other "component" is just JSON data on the entity, defined in YAML:

```yaml
with:
  data: { health: { hp: 100, max_hp: 100 }, ammo: { rounds: 8 } }
```

Accessed in scripts via `entity.get_i("/health/hp", 0)` / `entity.flag("invulnerable")`.
No engine rebuild needed to add new component types.

### Collision system

- Broadphase: BruteForce (all pairs)
- Narrowphase: Circle-circle only
- Layer/mask filtering: `(a.mask & b.layer) != 0 && (b.mask & a.layer) != 0`
- Wrap: Toroidal space support
- Output: `collision_enters/stays/exits(kind_a, kind_b)` — kind-filtered, named maps

---

## 6. Visual Binding & Sync

### Dual-entity pattern

```
YAML template (visual only)          Gameplay entity (logic only)
─────────────────────────           ─────────────────────────────
asteroid-vector.yml                  world.spawn_visual("asteroid", "asteroid-vector", #{...})
 └ type: vector                       └ x, y, size, shape
 └ points: [[0,-10],[8,-6],...]       └ flash_ms, split_pending
 └ fg: "#cccccc"                      └ collider, transform
 └ visible: false

         ↕ LINKED BY VisualBinding ↕
         ↕ POSITION SYNCED BY visual_sync_system (auto) ↕
```

### What the engine handles automatically

- **Auto-despawn**: `entity.despawn()` / `world.despawn_object(id)` removes ALL bound visuals
- **Multi-visual**: `world.bind_visual(id, visual_id)` supports additional bindings
- **Atomic spawn**: `world.spawn_visual(kind, template, data)` creates entity + visual + binding in one call
- **Position sync**: `visual_sync_system` auto-copies `Transform2D.x/y` → `scene position.x/y` every frame (after scripts run, step 10)

### What still requires script

Position sync is automatic. These still need script:

| Property | Why script needed |
|---|---|
| `visible` | Depends on game logic (active, respawn, invulnerability flash) |
| `vector.points` | Depends on heading rotation, asteroid shape, smoke size |
| `vector.fg` / `vector.bg` | Depends on flash state, TTL-based fade |
| Crack visual positions | Relative to parent asteroid |

Use `scene.set_vector(id, points, fg, bg)` or `scene.batch(id, map)` for efficient multi-property updates.

---

## 7. Implementation Status

All planned enhancements are complete. The asteroids mod (~1,100 LOC across three scripts) demonstrates the full scripting API.

### Completed
- ✅ E1: `entity.set_many`, `entity.data`, `entity.get_f`, `entity.get_s`
- ✅ E2: `world.collision_enters/stays/exits` — kind-filtered, named-field maps
- ✅ E3: Toroidal wrap via `world.enable_wrap_bounds` + physics-step integration
- ✅ E4: Engine physics (`TopDownShipController`, `PhysicsBody2D`) — scripts no longer integrate velocity manually
- ✅ E5: `game.*` / `level.*` / `input.*` APIs — session entity pattern eliminated
- ✅ E6: Shared module system (`import "mod-name" as m`)
- ✅ E7: Collision filtering (`world.collision_enters/stays/exits`)
- ✅ Sprint 2: `spawn_child`, collision events, action bindings
- ✅ API polish: `entity.id`, `entity.flag`, `entity.set_flag`, `scene.set_vector`, `scene.set_visible`, `scene.batch`, `world.rand_i`, `world.rand_seed`, `world.set_world_bounds`, `world.enable_wrap_bounds`, `KEY_*` and `LAYER_*` constants, `unit_vec32`, `game.get_i/s/b/f`, `world.any_alive`, `world.distance`

### Architecture
- Engine layer = generic typed systems. Scripts never re-implement physics/collision/timing.
- Script layer = game rules, spawn logic, event reactions only.
- Health, ammo, and other game data → entity JSON data bag, not engine structs.

---

## 8. Target Script Shape

### asteroids-game-loop.rhai (~200 LOC target)

```
import "asteroids-shared" as h;

// ── Constants ──────────────────────────────── (~10 LOC)
const TURN_STEP_MS = 40;
const SHOT_COOLDOWN = 200;
// ...

// ── Init (runs once) ──────────────────────── (~20 LOC)
if world.count_kind("ship") == 0 {
    world.spawn_visual("ship", "ship-vector", #{x: 320, y: 240, collider_radius: 10.0});
    game.set("/ast/score", 0);
    game.set("/ast/lives", 3);
    game.set("/ast/wave", 1);
    spawn_wave(world, 1);
}

// ── Input → Ship control ──────────────────── (~30 LOC)
let ship = world.entity(world.first_kind("ship"));
if input.down("a") { /* rotate left */ }
if input.down("d") { /* rotate right */ }
if input.down("w") { ship.set_acceleration(ax, ay); /* spawn smoke */ }
if input.down("space") { /* spawn bullet via spawn_visual */ }

// ── Asteroid update (rotation, flash, split) ─ (~40 LOC)
for ast_id in world.query_kind("asteroid") {
    let ast = world.entity(ast_id);
    let d = ast.data();
    // rotation animation, flash countdown, split spawning
    ast.set_many(#{rot_phase: new_phase, flash_ms: new_flash, ...});
}

// ── Collisions ────────────────────────────── (~30 LOC)
for hit in world.collisions_between("bullet", "asteroid") {
    world.entity(hit["bullet"]).despawn();
    let ast = world.entity(hit["asteroid"]);
    ast.set_many(#{flash_ms: 200, split_pending: true});
    game.set("/ast/score", game.get("/ast/score") + h::asteroid_score(ast.get_i("/size", 0)));
}
for hit in world.collisions_between("ship", "asteroid") {
    // ship death + invulnerability
}

// ── Wave spawn ────────────────────────────── (~20 LOC)
if world.count_kind("asteroid") == 0 {
    let wave = game.get("/ast/wave") + 1;
    game.set("/ast/wave", wave);
    spawn_wave(world, wave);
}

// ── HUD ───────────────────────────────────── (~5 LOC)
scene.set("hud-score", "text.content", `SCORE: ${game.get("/ast/score")}`);

// ── Helper functions ──────────────────────── (~45 LOC)
fn spawn_wave(world, wave) { /* spawn asteroids at borders */ }
```

### asteroids-render-sync.rhai (~50 LOC target)

```
import "asteroids-shared" as h;

// Position sync: AUTOMATIC (A3 visual_sync_system)
// Only update visual PROPERTIES that depend on game state:

// Ship visibility (invuln flash)
let ship_id = world.first_kind("ship");
if ship_id > 0 {
    let invuln = game.get("/ast/ship_invuln_ms");
    let visible = if invuln > 0 { (time.scene_elapsed_ms / 120) % 2 == 0 } else { true };
    scene.set("ship", "visible", visible);
    scene.set("ship", "vector.points", ship_points(game.get("/ast/ship_h")));
}

// Asteroid visual state (rotation, flash color, crack animations)
for ast_id in world.query_kind("asteroid") {
    let d = world.entity(ast_id).data();
    scene.set(d["visual_id"], "visible", d["active"]);
    scene.set(d["visual_id"], "vector.points", rotate_points(asteroid_points(d["shape"], d["size"]), d["rot_phase"]));
    scene.set(d["visual_id"], "vector.fg", h::asteroid_stroke_hex(d["flash_ms"], d["flash_total_ms"]));
    // crack animations (~10 LOC)
}

// Bullet: visibility only (position auto-synced by A3)
for b_id in world.query_kind("bullet") {
    let bref = world.entity(b_id);
    scene.set(bref.get("/visual_id"), "visible", bref.get_b("/active", false));
}

// Smoke: shape + color fade (position auto-synced by A3)
for s_id in world.query_kind("smoke") {
    let d = world.entity(s_id).data();
    scene.set(d["visual_id"], "visible", d["active"]);
    scene.set(d["visual_id"], "vector.points", h::smoke_points(d["radius"], d["ttl_ms"], d["max_ttl_ms"]));
    scene.set(d["visual_id"], "vector.fg", h::smoke_colour_hex(d["ttl_ms"], d["max_ttl_ms"]));
}
```

### asteroids-shared.rhai (~80 LOC target)

```
// Math helpers
fn crack_duration_ms() { 200 }
fn despawn_split_threshold() { 50 }
fn wrap_heading32(h) { ((h % 32) + 32) % 32 }
fn fragment_heading_offset(idx) { [8, -8, 16][idx] }

// Visual helpers
fn asteroid_stroke_hex(flash_ms, total_ms) { /* color interpolation */ }
fn smoke_colour_hex(ttl_ms, max_ttl_ms) { /* fade to dark */ }
fn smoke_points(radius, ttl_ms, max_ttl_ms) { /* shrinking diamond */ }
fn flash_fill_hex(flash_ms, total_ms) { /* white flash fade */ }
fn crack_visual_id(ast_data, idx) { ast_data[`crack_visual_${idx}`] }
fn hide_crack_visuals(scene, ast_data) { /* set visible=false on 3 cracks */ }
```

### LOC Summary

| File | Current | Target | Reduction |
|------|---------|--------|-----------|
| asteroids-game-loop.rhai | 887 | ~200 | -77% |
| asteroids-render-sync.rhai | 204 | ~50 | -76% |
| asteroids-shared.rhai | 0 | ~80 | (new) |
| **Total** | **1091** | **~330** | **-70%** |

---

## Appendix: BehaviorCommand Variants

```
PlayAudioCue { cue, volume? }
PlayAudioEvent { event, gain? }
PlaySong { song_id }
StopSong
SetVisibility { target, visible }
SetOffset { target, dx, dy }
SetText { target, text }
SetProps { target, visible?, dx?, dy?, text? }
SetProperty { target, path, value }
SceneSpawn { template, target }
SceneDespawn { target }
TerminalPushOutput { line }
TerminalClearOutput
SceneTransition { to_scene_id }
DebugLog { scene_id, source?, severity, message }
ScriptError { scene_id, source?, message }
```