# Scripting and Engine API Plan (2D, Mod-Agnostic)

Status date: 2026-03-29
Audience: next implementation agent
Scope: canonical engine-side scripting contract for 2D gameplay

## 0) Why this document exists

We aligned on one direction:
- mods are the lowest layer
- engine exposes generic capabilities
- scripts define rules, not low-level simulation plumbing

Current pain point is not missing one helper. The real issue is overlap:
- script does simulation bookkeeping
- script does gameplay state mutation
- script does gameplay -> scene render synchronization

This doc defines a non-overlapping target contract.

## 1) Hard constraints

1. Engine must not encode game-specific behavior.
2. Mod content must consume capabilities, never define engine contracts.
3. Keep `world.get/set(path)` only as fallback escape hatch.
4. Core 2D API must exclude combat-specific domains (`health`, `damage`, `weapon`) in v1 core.
5. Introduce capabilities in engine domains, not in mod scripts.

## 2) Current code anchors

Frame order and system wiring:
- `engine/src/game_loop.rs:141-159`

Gameplay simulation storage and components:
- `engine-game/src/components.rs:4-83`
- `engine-game/src/gameplay.rs:40-334`
- `engine-game/src/strategy.rs:6-52`
- `engine-game/src/collision.rs:7-99`

Collision event bridge:
- `engine/src/systems/gameplay_events.rs:1-30`

Behavior context and command model:
- `engine-behavior/src/lib.rs:36-147`

Rhai registration surface:
- `engine-behavior/src/lib.rs:620-1065`
- `engine-behavior/src/lib.rs:1644-2007`

Runtime command application:
- `engine-scene-runtime/src/behavior_runner.rs:221-498`
- runtime clone spawn/despawn: `engine-scene-runtime/src/behavior_runner.rs:480-771`

Audio sequencing:
- `engine/src/audio_sequencer.rs:16-134`
- `engine/src/systems/audio_sequencer.rs:9-33`
- `engine-audio-sequencer/src/lib.rs`

Startup validation:
- `app/src/main.rs:173-189`
- `app/src/main.rs:348-401`
- `engine-mod/src/startup/runner.rs:35-48`

Authoring contracts:
- script logic source contract: `engine-authoring/src/compile/scene.rs:135-181`
- repeat expansion: `engine-authoring/src/compile/scene.rs:822-927`
- schema logic/repeat: `schemas/scene.schema.yaml:152-170`, `480-560`

## 3) Data planes (must stay separated)

### 3.1 Terminology (strict)

- `object`: authored scene object/template instance (scene graph domain).
- `entity`: gameplay runtime entity in `GameplayWorld` (simulation domain).
- `visual binding`: mapping between entity and object for presentation sync.

Do not use `object` and `entity` interchangeably in code/docs/tasks.

### Plane A: Authored Scene State
Owned by scene authoring/runtime.
Examples:
- layout
- static objects
- UI widgets
- visual templates

Primary storage:
- scene data + `SceneRuntime`

### Plane B: Gameplay Simulation State
Owned by gameplay systems.
Examples:
- entities
- transform/physics/collider/lifetime
- timers
- world config

Primary storage:
- `GameplayWorld` + engine systems

### Plane C: Presentation State
Owned by presentation systems.
Examples:
- visibility overrides
- tint/flash
- animation playback state
- camera shake

Primary storage:
- scene runtime properties updated by bridge systems

Rule:
- never make one script own all 3 planes at once.

## 4) Non-overlap ownership matrix (single-writer contract)

| Namespace | Primary owner system | Writes | Must not write |
|---|---|---|---|
| `game` | game loop/runtime state | global runtime flags/time | entity components |
| `world` | world settings system | world bounds/gravity/wrap config | per-entity visual props |
| `entity` | entity service | create/destroy/metadata/tags | physics integration |
| `query` | query service | none (read-only) | any mutation |
| `transform` | transform service | position/rotation/scale | velocity/forces |
| `motion` | motion service | velocity/accel/drag caps | collider geometry |
| `body` | physics body service | rigidbody params/impulses | scene object properties |
| `collider` | collision service | shape/layer/mask/trigger | UI/scene text |
| `lifetime` | lifetime service | ttl state | manual scene despawn direct |
| `timer` | timer service | scheduled events/cooldowns | transform/physics |
| `camera` | camera service | camera state/follow/shake | gameplay entity health/state |
| `input` | input system | action maps/bindings | gameplay world mutation |
| `scene` | scene runtime service | authored object props | gameplay components |
| `visual` | visual binding bridge | gameplay<->scene links | physics/collision state |
| `anim` | animation service | animation playback params | entity physics |
| `audio` | audio services | cue/song/mixer state | gameplay components |
| `ui` | ui service | HUD/widget values | physics/collision |
| `fx` | vfx service | effects spawn/state | gameplay authority |
| `events` | event bus | event queues | direct component writes |
| `level` | level service | level loading/level state | per-frame physics state |
| `save` | persistence service | save values/version | scene runtime objects |
| `debug` | debug service | debug overlays/traces | gameplay authority |
| `math` | utility library | none | any mutation |
| `prefab` | prefab service | prefab instantiation | game rules |
| `spawner` | spawning service | spawn orchestration | render-only props |

Implementation rule:
- if a namespace needs to mutate data outside its ownership column, it emits an event/command and another owner applies it.

## 5) Canonical namespace map (de-duplicated)

Top-level namespaces (target):

```text
game
world
entity
query
transform
motion
body
collider
lifetime
timer
camera
input
scene
visual
audio
ui
fx
anim
level
save
events
debug
math
prefab
spawner
```

Not in core v1 (extension only):

```text
health
damage
weapon
teams
nav
```

## 6) Core API v1 contract

### 6.1 `game`

Required:
- `game.time_ms()`
- `game.delta_ms()`
- `game.fixed_delta_ms()`
- `game.tick()`
- `game.pause()`
- `game.resume()`
- `game.is_paused()`
- `game.restart()`
- `game.quit()`

Optional in v1:
- deterministic rng helpers
- lightweight global key-value state

### 6.2 `world`

Required:
- `world.width()`, `world.height()`
- `world.bounds()`
- `world.gravity()`, `world.set_gravity(x, y)`
- `world.wrap_enabled()`, `world.set_wrap_enabled(bool)`
- `world.set_wrap_bounds(left, top, right, bottom)`
- `world.clear()`, `world.reset()`

### 6.3 `entity`

Required (handle methods):
- `entity.exists()` - Check if entity still exists
- `entity.kind()` - Get entity kind
- `entity.tags()` - Get array of entity tags
- `entity.get_metadata()` - Get full entity metadata (id, kind, tags, all components)
- `entity.get_components()` - Get only component maps (transform, physics, collider, lifetime, visual_id)

Transform component (typed):
- `entity.transform()` - Get {x, y, heading}
- `entity.set_position(x, y)` - Set transform position only
- `entity.set_heading(heading)` - Set transform heading only

Physics component (typed):
- `entity.physics()` - Get {vx, vy, ax, ay, drag, max_speed}
- `entity.set_velocity(vx, vy)` - Set velocity components
- `entity.set_acceleration(ax, ay)` - Set acceleration components

Collider component (typed):
- `entity.collider()` - Get {shape, layer, mask} (shape: "circle" with radius or "polygon" with points)

Lifetime component (typed):
- `entity.lifetime_remaining()` - Get remaining time in ms

Fallback-only (escape hatch):
- `entity.get(path)`, `entity.get_i(path, fallback)`, `entity.get_bool(path, fallback)` - Read JSON path
- `entity.set(path, value)` - Mutate JSON path


### 6.4 `query`

Required (read-only):
- `query.all()`
- `query.by_kind(kind)`
- `query.by_tag(tag)`
- `query.first_by_kind(kind)`
- `query.first_by_tag(tag)`
- `query.in_radius(x, y, r)`
- `query.in_rect(x, y, w, h)`
- `query.overlap_entity(id)`
- `query.nearest(x, y, filter)`

### 6.5 `transform`

Required:
- `transform.get(id)`
- `transform.set(id, x, y)`
- `transform.set_full(id, x, y, rotation, scale_x, scale_y)`
- `transform.translate(id, dx, dy)`
- `transform.rotate(id, dr)`
- `transform.look_at(id, x, y)`
- `transform.forward(id)`

### 6.6 `motion`

Required:
- `motion.velocity(id)`, `motion.set_velocity(id, vx, vy)`
- `motion.add_velocity(id, dvx, dvy)`
- `motion.acceleration(id)`, `motion.set_acceleration(id, ax, ay)`
- `motion.drag(id)`, `motion.set_drag(id, v)`
- `motion.max_speed(id)`, `motion.set_max_speed(id, v)`
- `motion.move_forward(id, speed)`
- `motion.turn_towards(id, x, y, speed)`
- `motion.stop(id)`

### 6.7 `body`

Required:
- `body.enable/disable/is_enabled`
- `body.type/set_type` (`dynamic`/`kinematic`/`static`)
- `body.mass/set_mass`
- `body.velocity/set_velocity`
- `body.angular_velocity/set_angular_velocity`
- `body.add_force`
- `body.add_impulse`
- `body.teleport`

Optional in v1:
- restitution/friction/damping/gravity_scale
- freeze position/rotation constraints

### 6.8 `collider`

Required:
- `collider.enable/disable/is_enabled`
- `collider.clear/remove`
- `collider.set_circle`
- `collider.set_box`
- `collider.set_polygon`
- `collider.set_trigger`
- `collider.layer/set_layer`
- `collider.mask/set_mask`
- `collider.overlaps(id)`

### 6.9 `lifetime`

Required:
- `lifetime.set_ms(id, ttl_ms)`
- `lifetime.set_seconds(id, seconds)`
- `lifetime.clear(id)`
- `lifetime.remaining_ms(id)`
- `lifetime.expired(id)`

Optional in v1:
- `lifetime.pause/resume`
- expire callbacks through event bus

### 6.10 `timer`

Required:
- `timer.after(ms, event_or_callback)`
- `timer.every(ms, event_or_callback)`
- `timer.cancel(timer_id)`
- `timer.exists(timer_id)`

Per-entity cooldown helpers (recommended):
- `timer.cooldown_ready(id, name)`
- `timer.start_cooldown(id, name, ms)`
- `timer.remaining_cooldown(id, name)`

### 6.11 `camera`

Required:
- `camera.main()`
- `camera.create()/destroy()`
- `camera.position/set_position`
- `camera.zoom/set_zoom`
- `camera.follow(target_id)`
- `camera.clear_follow()`
- `camera.shake(amplitude, duration_ms)`

### 6.12 `input`

Required:
- `input.is_down(action)`
- `input.was_pressed(action)`
- `input.was_released(action)`
- `input.axis(name)`
- `input.vector(name)`

Fallback:
- raw key APIs can remain for debug/backward compatibility

### 6.13 `scene`

Required:
- `scene.find(id)`
- `scene.exists(id)`
- `scene.spawn(template_id)`
- `scene.spawn_at(template_id, x, y)`
- `scene.destroy(id)`
- `scene.get(id, path)` and `scene.set(id, path, value)` for authored props

Constraint:
- `scene` API does not own gameplay simulation data.

### 6.14 `visual`

Required:
- `visual.bind(entity_id, scene_object_id)`
- `visual.unbind(entity_id)`
- `visual.bound_object(entity_id)`
- `visual.spawn_bound(entity_id, template_id)`
- `visual.destroy_bound(entity_id)`
- `visual.enable_auto_sync(entity_id)`
- `visual.disable_auto_sync(entity_id)`

### 6.15 `events`

Required:
- `events.emit(name)`
- `events.emit_with(name, payload)`
- `events.poll(name)`
- `events.next(name)`
- `events.clear(name)`
- `events.has(name)`

Engine-generated events (v1 target):
- `entity_spawned`
- `entity_destroyed`
- `collision_enter`
- `collision_stay`
- `collision_exit`
- `lifetime_expired`

### 6.16 `math`

Required:
- vec2 helpers
- length/normalize/dot/distance
- angle/from_angle
- clamp/wrap/lerp
- deg/rad conversion

### 6.17 `prefab`

Required:
- `prefab.exists(id)`
- `prefab.spawn(id)`
- `prefab.spawn_at(id, x, y)`
- `prefab.instantiate(id, overrides)`

### 6.18 `spawner`

Required:
- `spawner.spawn(kind, config)`
- `spawner.spawn_prefab(prefab_id)`
- `spawner.spawn_prefab_at(prefab_id, x, y)`
- `spawner.spawn_many(prefab_id, count, area)`

### 6.19 `audio`

Required:
- semantic event play
- cue play/stop
- song play/stop

Reference implementation path:
- `engine/src/audio_sequencer.rs`
- `engine/src/systems/audio_sequencer.rs`

### 6.20 `level`

Required:
- `level.current()`
- `level.load(id)`
- `level.reload()`
- `level.restart()`
- `level.state_get/path`
- `level.state_set/path`

### 6.21 `save`

Required:
- `save.get/set/has/remove`
- `save.flush`

Must include schema version support (migration roadmap section below).

### 6.22 `debug`

Required:
- logging (`log/warn/error`)
- lightweight debug draw helpers (line/circle/rect/text)

## 7) Core API v2 (after v1 stabilizes)

`anim`, `fx`, `ui` are valid core v2 targets once ownership boundaries above are stable.

`teams` and `nav` can be added as optional core modules if they remain genre-agnostic.

## 8) Explicitly out of core v1

Combat-oriented domains are excluded from engine core API v1:
- `health`
- `damage`
- `weapon`

Reason:
- they are gameplay-domain specific
- they can be built as optional extensions once core simulation and event contracts are stable

Recommended place:
- dedicated extension domain (for example `engine-gameplay-combat`)
- optional Rhai module loaded by mods that opt in

## 9) Engine-side automation required (scripts should not do this manually)

Must be automatic in engine systems:
1. fixed-step motion/body update
2. collision detection
3. collision enter/stay/exit generation
4. lifetime countdown
5. entity despawn on lifetime expiry
6. bound visual cleanup on entity despawn/expiry
7. transform -> visual auto-sync (when enabled)
8. cooldown timer ticking

Current integration touchpoints:
- frame order: `engine/src/game_loop.rs:141-159`
- gameplay/lifetime: `engine/src/systems/gameplay.rs`
- collisions: `engine/src/systems/collision.rs`
- event buffer: `engine/src/systems/gameplay_events.rs`
- visual cleanup: `engine/src/systems/visual_binding.rs`

## 10) Scripting rules for mods

1. Prefer typed namespaces (`transform`, `motion`, `collider`, etc.).
2. Use `entity.get/set(path)` only for custom payload not covered by typed APIs.
3. Do not implement fixed-step accumulator in scripts.
4. Do not run manual render-sync loops when `visual` auto-sync can handle it.
5. Keep script responsibilities to:
- game rules
- progression
- event reactions
- optional authored overrides

## 11) Implementation backlog by phase

### P1 (foundation)
- hierarchy guardrails + CI checks
- single-writer ownership rules in docs
- validation pipeline extensions for capability configs

### P2 (core API ergonomics)
- typed entity handle methods
- action map API
- timer/cooldown API
- prefab + basic spawner

### P3 (simulation semantics)
- fixed-step ownership in engine
- collision enter/stay/exit
- query API expansion
- named collision matrix

### P4 (bridge and migration)
- visual auto-sync bridge
- spawn/pool service
- asteroids migration to typed API + KPI measurement

## 12) Asteroids migration checklist (benchmark mod)

Baseline:
- `mods/asteroids/behaviors/asteroids-game-loop.rhai` ~925 lines
- `mods/asteroids/behaviors/asteroids-render-sync.rhai` ~209 lines

Targets:
1. remove script-level fixed-step accumulator
2. remove manual gameplay -> scene position sync loops where possible
3. reduce root-map `world.get/set("/")` usage
4. move generic logic to engine capabilities

KPI:
- 40-60% gameplay script line reduction
- 70%+ reduction of root map writes
- 60%+ reduction of repetitive scene property sets from gameplay scripts

## 13) Validation and verification commands

```bash
cargo check -q
cargo test -p engine
cargo test -p engine-game
cargo test -p engine-behavior
cargo run -p app -- --mod-source=mods/asteroids --check-scenes
cargo run -p app -- --mod-source=mods/shell-quest --check-scenes
```

Runtime smoke:

```bash
cargo run -p app -- --mod-source=mods/asteroids --sdl2 --audio
./menu --release --sdl2
```

## 14) First tasks for next agent

1. Add ownership matrix checks to docs and CI.
2. Implement typed `EntityHandle` API expansion.
3. Add action map schema + startup validation check.
4. Move fixed-step control from script to engine loop.
5. Implement collision enter/stay/exit event buffer.
6. Implement minimal `visual` auto-sync bridge.
7. Migrate asteroids scripts and record KPI deltas.

End of canonical scripting handover.
