# Engine Overhaul: Phase 1-3 Implementation Review & Handoff

**Date**: 2026-03-30
**Status**: Phase 1 ✓ | Phase 2.1 ⚠️ | Phase 2.2-3.2 ✗ (Wrong approach)
**Action Required**: Delete Phases 2.2, 2.3, 3.2. Keep Phase 1 + SplitOnDestroy component.

---

## Executive Summary

**What was attempted**: Phases 1-3 of engine API overhaul to move gameplay logic from Rhai scripts into typed engine components and systems.

**What went wrong**: Phases 2.2-3.2 implement **game-specific logic in the engine** (prefabs, particles, wave spawning) instead of just **exposing clean APIs for component access**. The distinction:

- ✓ **Engine layer**: Store typed components, expose clean Rhai access, run physics/collision/event systems
- ✗ **Game layer**: Define what an explosion looks like, manage waves, decide when to split asteroids
- ✗ **What happened**: Game logic (prefab factory, particle definitions, wave spawner) ended up in engine code

**Result**:
- Phase 1: Correct. Provides reusable component storage + clean access APIs.
- Phase 2.1: Component is correct, system is wrong (game logic in engine).
- Phase 2.2: Entirely wrong (prefab registry + spawning).
- Phase 2.3: Entirely wrong (particle effect factory).
- Phase 3.2: Entirely wrong (wave spawning system).

---

## Detailed Breakdown

### ✓ PHASE 1: Correct Architecture (KEEP)

#### Components Added
| Component | File | Lines | Status |
|-----------|------|-------|--------|
| `TopDownShipController` | `engine-game/src/components.rs` | 156-220 | ✓ KEEP |
| `Health` | `engine-game/src/components.rs` | 239-265 | ✓ KEEP |
| `GameplayEvent` enum | `engine-game/src/components.rs` | 317-325 | ✓ KEEP |
| `DespawnVisual` | `engine-game/src/components.rs` | 88-97 | ✓ KEEP |

**Why correct**: These are reusable typed components. Any game might need discrete heading control, health tracking, or events.

#### Systems Added
| System | File | Status |
|--------|------|--------|
| `ship_controller_system` | `engine/src/systems/ship_controller.rs` | ✓ KEEP |
| `collision_system` emits `CollisionEnter` events | `engine-game/src/collision.rs:56-57` | ✓ KEEP |

**Why correct**:
- `ship_controller_system` handles frame-rate-independent heading rotation and thrust physics — engine-level math that should run automatically.
- Collision system emitting events is standard event bus pattern — engine emits, scripts respond.

#### GameplayWorld API Methods (Phase 1)
| Method | File | Lines | Status |
|--------|------|-------|--------|
| `attach_controller(id, controller)` | `gameplay.rs` | 534-541 | ✓ KEEP |
| `controller(id)` | `gameplay.rs` | 542-547 | ✓ KEEP |
| `with_controller(id, f)` | `gameplay.rs` | 548-561 | ✓ KEEP |
| `ids_with_controller()` | `gameplay.rs` | 562-565 | ✓ KEEP |
| `set_health(id, hp, max_hp)` | `gameplay.rs` | 618-625 | ✓ KEEP |
| `health(id)` | `gameplay.rs` | 626-630 | ✓ KEEP |
| `apply_damage(target, source, amount)` | `gameplay.rs` | 632-658 | ✓ KEEP |
| `is_dead(id)` | `gameplay.rs` | 659-664 | ✓ KEEP |
| `emit_event(event)` | `gameplay.rs` | 575-583 | ✓ KEEP |
| `poll_events(event_type)` | `gameplay.rs` | 584-610 | ✓ KEEP |
| `clear_events()` | `gameplay.rs` | 611-616 | ✓ KEEP |

**Why correct**: All expose typed component data with clean, single-responsibility APIs.

#### Rhai API (Phase 1)
| Function | File | Status |
|----------|------|--------|
| `attach_ship_controller(world, id, config)` | `engine-behavior/src/lib.rs:1157` | ✓ KEEP |
| `ship_set_turn(world, id, dir)` | `engine-behavior/src/lib.rs:1162` | ✓ KEEP |
| `ship_set_thrust(world, id, on)` | `engine-behavior/src/lib.rs:1167` | ✓ KEEP |
| `ship_heading(world, id)` | `engine-behavior/src/lib.rs:1172` | ✓ KEEP |
| `ship_heading_vector(world, id)` | `engine-behavior/src/lib.rs:1177` | ✓ KEEP |
| `ship_velocity(world, id)` | `engine-behavior/src/lib.rs:1182` | ✓ KEEP |
| `health_set(world, id, hp, max_hp)` | `engine-behavior/src/lib.rs:1201` | ✓ KEEP |
| `health_get(world, id)` | `engine-behavior/src/lib.rs:1206` | ✓ KEEP |
| `health_max(world, id)` | `engine-behavior/src/lib.rs:1211` | ✓ KEEP |
| `health_dead(world, id)` | `engine-behavior/src/lib.rs:1216` | ✓ KEEP |
| `damage_apply(world, target, source, amount)` | `engine-behavior/src/lib.rs:1221` | ✓ KEEP |
| `poll_collisions(world)` | `engine-behavior/src/lib.rs:1189` | ✓ KEEP |
| `clear_events(world)` | `engine-behavior/src/lib.rs:1194` | ✓ KEEP |

**Why correct**: Clean, typed access to component data. Scripts call these instead of manually manipulating JSON.

---

### ⚠️ PHASE 2.1: Component Correct, System Wrong (PARTIALLY KEEP)

#### Component: SplitOnDestroy
| Item | File | Lines | Status |
|------|------|-------|--------|
| `SplitOnDestroy` struct | `engine-game/src/components.rs` | 272-310 | ✓ KEEP |
| `set_split_on_destroy(id, config)` | `gameplay.rs` | 671-678 | ✓ KEEP |
| `split_on_destroy(id)` | `gameplay.rs` | 679-684 | ✓ KEEP |
| `with_split_on_destroy(id, f)` | `gameplay.rs` | 685-698 | ✓ KEEP |
| `ids_with_split_on_destroy()` | `gameplay.rs` | 699-707 | ✓ KEEP |

**Why correct**: `SplitOnDestroy` is just typed data (child_count, size_delta, delay_ms). Storing and accessing it is fine.

**Rhai API:**
| Function | File | Status |
|----------|------|--------|
| `destructible_configure(world, id, config)` | `engine-behavior/src/lib.rs:1227` | ✓ KEEP |

**Why correct**: Lets script set split config on an entity.

#### System: destructible_system
| Item | File | Status |
|--------|------|--------|
| `destructible_system()` | `engine/src/systems/destructible.rs` | ✗ DELETE |
| System registration | `engine/src/systems/mod.rs:8` | ✗ DELETE |
| Game loop call | `engine/src/game_loop.rs:153-155` | ✗ DELETE |

**Why wrong**: The system does:
```rust
// 1. Poll health_zero events
let health_zero_events = world.poll_events("health_zero");

// 2. Mark as triggered
for (entity, _killer) in health_zero_events {
    world.with_split_on_destroy(entity, |split| {
        if !split.triggered {
            split.triggered = true;
            split.elapsed_ms = 0;
        }
    });
}

// 3. Advance timers
for id in world.ids_with_split_on_destroy() {
    world.with_split_on_destroy(id, |split| {
        if split.triggered {
            split.elapsed_ms += dt_ms as u32;
        }
    });
}
```

This is **game logic** (orchestrating when splits happen) disguised as a system. The script already has:
- `health_dead(world, id)` — check if entity is dead
- `split_on_destroy(world, id)` — get the split config
- Ability to call `spawn_prefab()` or similar to create children

The script should:
```rhai
// In game loop
if health_dead(world, asteroid_id) {
    let split_cfg = split_on_destroy(world, asteroid_id);
    if split_cfg.is_some() {
        // Spawn children immediately or after delay
        spawn_children(world, asteroid_id, split_cfg);
    }
}
```

---

### ✗ PHASE 2.2: Entirely Wrong (DELETE)

#### files to delete
- `engine-game/src/prefabs.rs` (entire file)

#### GameplayWorld methods to delete (gameplay.rs)
| Method | Lines | Why |
|--------|-------|-----|
| `register_prefab(prefab_id, spec)` | 760-768 | Game logic: prefab registry |
| `get_prefab(prefab_id)` | 769-776 | Game logic: prefab lookup |
| `has_prefab(prefab_id)` | 777-788 | Game logic: prefab checking |
| `spawn_from_prefab(prefab_id, params)` | 790-856 | Game logic: prefab instantiation |

#### Rhai API to delete (engine-behavior/src/lib.rs)
| Function | Lines | Why |
|----------|-------|-----|
| `spawn_prefab(world, prefab_id, params)` | 1233-1238 | Game logic |
| `register_prefab(world, prefab_id, spec)` | 1239-1244 | Game logic |
| `ScriptGameplayApi::spawn_prefab()` | 2830-... | Game logic |
| `ScriptGameplayApi::register_prefab()` | 2876-... | Game logic |

#### Why wrong
1. **Duplicates existing engine API**: `spawn()` + component setters already exist. No need for "prefab registry".

2. **Over-engineered**: `PrefabSpec` has an `Option<>` for every component type:
   ```rust
   pub transform: Option<Transform2D>,
   pub physics: Option<PhysicsBody2D>,
   pub collider: Option<Collider2D>,
   pub lifetime: Option<Lifetime>,
   pub visual: Option<VisualBinding>,
   pub health: Option<Health>,
   pub ship_controller: Option<TopDownShipController>,
   pub split_on_destroy: Option<SplitOnDestroy>,
   pub timers: Option<EntityTimers>,
   pub payload: Option<JsonValue>,
   ```
   Every time you add a component, you must update PrefabSpec, its impl, its builder methods, and `spawn_from_prefab()`. This is unmaintainable.

3. **Rhai implementation is incomplete**: `register_prefab()` only extracts `kind` from the map:
   ```rust
   if let Some(kind) = spec.get("kind") {
       if let Some(k) = kind.clone().try_cast::<String>() {
           prefab.kind = k;
       }
   }
   ```
   So `register_prefab()` creates an empty prefab with just a kind. Not useful.

4. **Belongs in game code**: Define "asteroid_large" vs "asteroid_small" in the asteroids Rhai script, not in the engine.

#### Correct approach
Keep just `spawn()` + individual component setters. Scripts can write helper functions:

```rhai
// In asteroids script
fn spawn_asteroid_large(world, x, y) {
    let id = world.spawn("asteroid", #{});
    world.set_transform(id, x, y, 0.0);
    world.set_physics(id, 0.0, 0.0, 0.0, 0.0, 0.05, 0.0);
    world.set_collider(id, 20.0);
    world.bind_visual(id, "asteroid_large");
    health_set(world, id, 50, 50);
    id
}

fn spawn_asteroid_medium(world, x, y) {
    let id = world.spawn("asteroid", #{});
    world.set_transform(id, x, y, 0.0);
    world.set_physics(id, 0.0, 0.0, 0.0, 0.0, 0.05, 0.0);
    world.set_collider(id, 15.0);
    world.bind_visual(id, "asteroid_medium");
    health_set(world, id, 25, 25);
    id
}
```

No engine prefab system needed.

---

### ✗ PHASE 2.3: Entirely Wrong (DELETE)

#### Files to delete
- `engine-game/src/particles.rs` (entire file)

#### GameplayWorld methods to delete
| Method | Lines | Why |
|--------|-------|-----|
| `spawn_effect(effect_type, x, y, velocity_scale)` | 864-880 | Game content |

#### Rhai API to delete
| Function | Lines | Why |
|----------|-------|-----|
| `spawn_effect(world, effect_type, x, y, velocity_scale)` | 1245-1250 | Game content |
| `ScriptGameplayApi::spawn_effect()` | 2896-... | Game content |

#### Why wrong
Hardcoded game content in engine:

```rust
pub fn explosion() -> PrefabSpec {
    PrefabSpec::new("particle")
        .with_visual(VisualBinding { visual_id: Some("particle_explosion".to_string()), ... })
        .with_lifetime(Lifetime { ttl_ms: 200, ... })
        .with_physics(PhysicsBody2D { vx: 0.0, vy: 0.0, ax: 0.0, ay: 0.0, drag: 0.1, max_speed: 0.0 })
}

pub fn smoke() -> PrefabSpec {
    PrefabSpec::new("particle")
        .with_visual(VisualBinding { visual_id: Some("particle_smoke".to_string()), ... })
        .with_lifetime(Lifetime { ttl_ms: 600, ... })
        .with_physics(PhysicsBody2D { drag: 0.05, ... })
}

pub fn blood() -> PrefabSpec {
    PrefabSpec::new("particle")
        .with_visual(VisualBinding { visual_id: Some("particle_blood".to_string()), ... })
        .with_lifetime(Lifetime { ttl_ms: 800, ... })
        .with_physics(PhysicsBody2D { vx: 0.0, vy: 0.0, ax: 0.0, ay: 0.5, drag: 0.08, ... })  // ay: 0.5 is gravity!
}
```

"An explosion lasts 200ms with 0.1 drag" is a **game design decision**, not engine infrastructure. Same for smoke lingering longer, or blood having gravity.

The engine provides the **primitives** (Lifetime, VisualBinding, PhysicsBody2D). Scripts compose them:

```rhai
// In asteroids script, when explosion happens:
let particle = world.spawn("particle", #{});
world.set_transform(particle, explosion_x, explosion_y, 0.0);
world.bind_visual(particle, "particle_explosion");
world.set_lifetime(particle, 200, false);  // 200ms TTL, despawn visual on expire
world.set_physics(particle, 0.0, 0.0, 0.0, 0.0, 0.1, 0.0);  // drag: 0.1
```

No `spawn_effect()` needed.

---

### ✗ PHASE 3.2: Entirely Wrong (DELETE)

#### Files to delete
- `engine-game/src/spawner.rs` (entire file)

#### GameplayWorld fields to delete
| Field | Lines | Why |
|-------|-------|-----|
| `spawner: SpawnerState` | `gameplay.rs:37` | Game state |

#### GameplayWorld methods to delete
| Method | Lines | Why |
|--------|-------|-----|
| `spawner_state()` | 885-892 | Game state |
| `set_spawner_state(state)` | 893-904 | Game state |
| `spawn_wave(config)` | 905-925 | Game logic |
| `with_spawner_state(f)` | 926-937 | Game state |
| `wave_number()` | 938-945 | Game state |
| `next_wave()` | 946-954 | Game state |
| `is_wave_complete()` | 955-962 | Game state |

#### Rhai API to delete
| Function | Lines | Why |
|----------|-------|-----|
| `wave_number(world)` | 1251-1256 | Game logic |
| `next_wave(world)` | 1257-1262 | Game logic |
| `is_wave_complete(world)` | 1263-1268 | Game logic |
| `spawn_wave(world, prefab_id, count, pattern)` | 1269-1274 | Game logic |
| All corresponding `ScriptGameplayApi` methods | 2907-... | Game logic |

#### Why wrong
Wave management (spawning N enemies in a pattern, tracking waves, checking completion) is **100% game logic**. The engine shouldn't know about waves.

What spawner.rs adds:
```rust
pub enum SpawnPattern {
    Random,
    Circle { center_x, center_y, radius },
    Line { x, y, horizontal, spacing },
    Edges,  // Hardcoded 640x480!
}

pub struct WaveConfig {
    count: u32,
    prefab_id: String,
    pattern: SpawnPattern,
    bounds: Option<(f32, f32, f32, f32)>,
    spawn_delay_ms: u32,
}

pub struct SpawnerState {
    wave_number: u32,
    spawned_count: u32,
    active_enemy_count: u32,
    spawn_accumulator_ms: u32,
}
```

This is entirely game state. The asteroids script should manage it:

```rhai
// In asteroids-game-loop.rhai
let game = #{
    wave_number: 1,
    spawned_count: 0,
    active_enemy_count: 0,
    is_wave_complete: false,
};

fn spawn_wave(world, wave_num) {
    let count = 3 + wave_num;  // 4, 5, 6, ... asteroids
    for i in range(0, count) {
        let angle = (i as float / count as float) * 6.28318;  // 2π
        let x = 320.0 + 200.0 * angle.cos();
        let y = 240.0 + 200.0 * angle.sin();
        let asteroid_id = spawn_asteroid_large(world, x, y);
        game.active_enemy_count += 1;
    }
}

fn update_wave(world) {
    // Count alive asteroids
    game.active_enemy_count = count_asteroids_alive(world);
    if game.active_enemy_count == 0 && game.spawned_count > 0 {
        game.wave_number += 1;
        game.spawned_count = 0;
        spawn_wave(world, game.wave_number);
    }
}
```

---

## Correct Architecture

### Engine Responsibility (What we got right in Phase 1)
- **Typed component storage**: Transform, Physics, Collider, Health, TopDownShipController, etc.
- **Clean component access APIs**: `set_transform()`, `health()`, `apply_damage()`, etc.
- **Systems**: Physics integration, collision detection, event emission, ship controller math
- **Event bus**: `emit_event()`, `poll_events()`, `clear_events()`
- **Entity lifecycle**: `spawn()`, `despawn()`, component attachment

### Game/Mod Responsibility (What should move to scripts)
- **Prefab/template definitions**: Define "asteroid_large", "asteroid_small" in Rhai
- **Particle effect definitions**: Define explosion lifetime, smoke drag, blood gravity in Rhai
- **Wave management**: Track waves, spawn counts, wave completion in Rhai
- **Game state**: Score, lives, current wave in Rhai
- **Gameplay loops**: Input handling, collision response, AI, spawning in Rhai

---

## Files to Delete

```
engine-game/src/prefabs.rs          # Entire file
engine-game/src/particles.rs        # Entire file
engine-game/src/spawner.rs          # Entire file
engine/src/systems/destructible.rs  # Entire file
```

## Files to Edit

### engine-game/src/gameplay.rs
Delete:
- Imports for prefabs, particles, spawner (lines 13-15)
- GameplayStore fields: `prefabs`, `spawner` (lines 37-38)
- Default impl entries for `prefabs` and `spawner` (lines 63-64)
- All Prefab API methods (lines 760-788, 790-856)
- All ParticleFX API methods (lines 864-880)
- All Spawner API methods (lines 885-962)

### engine-game/src/lib.rs
Delete:
- `pub mod prefabs;` (line)
- `pub mod particles;` (line)
- `pub mod spawner;` (line)
- Exports for PrefabSpec, SpawnParams, WaveConfig, SpawnPattern, SpawnerState (line)

### engine/src/systems/mod.rs
Delete:
- `pub mod destructible;` (line 8)

### engine/src/game_loop.rs
Delete:
- Import for `GameplayWorld` (if only used for destructible)
- Destructible system call (lines 153-155)

### engine-behavior/src/lib.rs
Delete:
- Rhai registrations for: `spawn_prefab`, `register_prefab`, `spawn_effect`, `wave_number`, `next_wave`, `is_wave_complete`, `spawn_wave` (lines ~1233-1274)
- ScriptGameplayApi methods: `spawn_prefab`, `register_prefab`, `spawn_effect`, `wave_number`, `next_wave`, `is_wave_complete`, `spawn_wave` (lines ~2830-2950+)

---

## What Remains (Keep)

### Components & Systems
- `TopDownShipController` — Arcade ship control
- `Health` — HP tracking
- `SplitOnDestroy` — Child spawn config (data only, no system)
- `GameplayEvent` — Event emission
- `ship_controller_system` — Heading + thrust integration
- `collision_system` with `CollisionEnter` events — Collision detection

### Rhai APIs
- All ship controller functions (`ship_set_turn`, `ship_heading`, etc.)
- All health functions (`health_set`, `health_dead`, `damage_apply`)
- Event functions (`poll_collisions`, `clear_events`)
- Prefab config function (`destructible_configure`)

### GameplayWorld Methods
- Everything in Phase 1 (transform, physics, collider, health, events)
- SplitOnDestroy storage/access (data only)

---

## Handoff Next Steps

1. **Delete 4 files** (prefabs.rs, particles.rs, spawner.rs, destructible.rs)
2. **Edit 4 files** as specified above
3. **Verify compilation**: `cargo check --all` should pass
4. **Commit**: "Phase 1-3 cleanup: Remove game logic from engine, keep Phase 1 APIs"
5. **Asteroids mod rewrite** can now use clean Phase 1 APIs + script-level game logic

---

## Example: How Asteroids Script Would Use Clean Phase 1 API

```rhai
// asteroids-game-loop.rhai

let game = #{
    wave_number: 1,
    active_enemies: 0,
    score: 0,
    lives: 3,
};

fn spawn_asteroid(world, x, y, size) {
    let id = world.spawn("asteroid", #{});
    world.set_transform(id, x, y, 0.0);
    world.set_physics(id, 0.0, 0.0, 0.0, 0.0, 0.05, 0.0);
    world.set_collider(id, size * 5.0);
    world.bind_visual(id, "asteroid_" + size);
    health_set(world, id, size * 10, size * 10);

    // Configure splitting when destroyed
    destructible_configure(world, id, #{
        delay_ms: 50,
        child_count: 2,
        size_delta: -1,
        velocity_factor: 1.5,
    });

    game.active_enemies += 1;
    id
}

fn spawn_wave(world) {
    let count = 3 + game.wave_number;
    for i in range(0, count) {
        let angle = (i as float / count as float) * 6.28318;
        let x = 320.0 + 200.0 * angle.cos();
        let y = 240.0 + 200.0 * angle.sin();
        spawn_asteroid(world, x, y, 3);  // size 3
    }
}

fn spawn_particle(world, type, x, y) {
    let id = world.spawn("particle", #{});
    world.set_transform(id, x, y, 0.0);
    world.bind_visual(id, "particle_" + type);

    // Define lifetime per type
    let ttl_ms = match type {
        "explosion" => 200,
        "smoke" => 600,
        "hit" => 50,
        _ => 100,
    };

    let drag = match type {
        "explosion" => 0.1,
        "smoke" => 0.05,
        _ => 0.2,
    };

    world.set_lifetime(id, ttl_ms, true);
    world.set_physics(id, 0.0, 0.0, 0.0, 0.0, drag, 0.0);
    id
}

fn on_update(world) {
    // Handle ship input
    ship_set_turn(world, ship_id, turn_input);
    ship_set_thrust(world, ship_id, thrust_input);

    // Check collisions
    let collisions = poll_collisions(world);
    for collision in collisions {
        // Spawn hit effect
        spawn_particle(world, "hit", collision.x, collision.y);

        // Apply damage
        damage_apply(world, collision.b_id, ship_id, 10);
    }

    // Check for dead enemies and spawn children
    for enemy_id in world.query_kind("asteroid") {
        if health_dead(world, enemy_id) {
            let split_cfg = split_on_destroy(world, enemy_id);
            if split_cfg.is_some() {
                // Spawn children
                for _ in range(0, split_cfg.child_count) {
                    let x = transform(world, enemy_id).x + rand(-10, 10);
                    let y = transform(world, enemy_id).y + rand(-10, 10);
                    spawn_asteroid(world, x, y, split_cfg.size_delta);
                }
            }
            world.despawn(enemy_id);
            game.score += 100;
            game.active_enemies -= 1;
        }
    }

    // Check wave completion
    if game.active_enemies == 0 {
        game.wave_number += 1;
        spawn_wave(world);
    }
}
```

No prefab registry, no wave spawner system, no hardcoded particle definitions. Just clean Phase 1 APIs.

---

## Summary

| Phase | Status | Action |
|-------|--------|--------|
| **Phase 1** | ✓ Correct | Keep all |
| **Phase 2.1** | ⚠️ Mixed | Keep component, delete system |
| **Phase 2.2** | ✗ Wrong | Delete entirely (prefabs.rs) |
| **Phase 2.3** | ✗ Wrong | Delete entirely (particles.rs) |
| **Phase 3.2** | ✗ Wrong | Delete entirely (spawner.rs) |

**Critical insight**: Engine = primitives & APIs. Game = composition & logic. Don't put game logic in engine.
