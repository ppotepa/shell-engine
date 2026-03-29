# Sprint 1: Gameplay Runtime APIs — Handoff

**Status:** ~85% complete. Engine APIs implemented; hitting 3 compile errors in behavior system. Needs fixes then asteroids script refactor.

**Goal:** Reduce asteroids game-loop.rhai boilerplate by moving cooldowns, timers, input actions, and world wrap into engine systems.

---

## What's Been Done ✅

### 1. Components Added (engine-game/src/components.rs)

✅ **EntityTimers** + **WrapBounds** — two new component types with methods
- Exported in `engine-game/src/lib.rs`

### 2. GameplayWorld Extended (engine-game/src/gameplay.rs)

✅ **Storage in GameplayStore**
- `timers: BTreeMap<u64, EntityTimers>` 
- `wrap_bounds: BTreeMap<u64, WrapBounds>`
- `despawn()` cleans up both

✅ **Cooldown/Status API (8 methods)**
- `cooldown_start/ready/remaining(id, name, ms)`
- `status_add/has/remaining(id, name, ms)`
- `ids_with_timers()`, `tick_timers(dt_ms)`

✅ **Wrap API (4 methods)**
- `set/remove_wrap_bounds(id, bounds)`
- `wrap_bounds_for(id)`, `ids_with_wrap()`
- `apply_wrap()` — toroidal clamp after physics

### 3. Gameplay System (engine/src/systems/gameplay.rs)

✅ Updated `gameplay_system()` to call:
1. Physics integration
2. `apply_wrap()` ← NEW
3. `tick_timers(dt_ms)` ← NEW
4. Lifetime cleanup

### 4. Input Actions (engine-behavior/src/lib.rs)

✅ **BehaviorCommand::BindInputAction**
```rust
BindInputAction { action: String, keys: Vec<String> }
```

✅ **BehaviorContext**
- New field: `pub action_bindings: Arc<HashMap<String, Vec<String>>>`

✅ **ScriptInputApi**
- New field: `queue: Arc<Mutex<Vec<BehaviorCommand>>>`
- New methods: `action_down(action)`, `bind_action(action, keys)`

✅ **Rhai registrations**
```rhai
input.action_down("action_name")
input.bind_action("action_name", ["Up", "w", ...])
entity.cooldown_start/ready/remaining(name, ms)
entity.status_add/has/remaining(name, ms)
world.enable_wrap(id, min_x, max_x, min_y, max_y)
world.disable_wrap(id)
```

✅ **Entity API methods** (ScriptGameplayEntityApi)
- All 6 timer methods implemented

✅ **World API methods** (ScriptGameplayApi)
- `enable_wrap()`, `disable_wrap()` implemented

---

## Current Compile Errors ⚠️

### Error 1: Missing arguments to ScriptInputApi::new()

**File:** `engine-behavior/src/lib.rs` line 3258

**Current:**
```rust
scope.push("input", ScriptInputApi::new(Arc::clone(&ctx.keys_down)));
```

**Problem:** ScriptInputApi::new() now takes 3 args, was 1 arg.

**Fix:**
```rust
scope.push(
    "input",
    ScriptInputApi::new(
        Arc::clone(&ctx.keys_down),
        Arc::clone(&ctx.action_bindings),  // ADD THIS
        Arc::clone(&helper_commands),       // ADD THIS
    ),
);
```

### Error 2: BehaviorContext missing action_bindings

**File:** `engine-behavior/src/lib.rs` around line 3388 (in `make_test_context()` or similar)

**Problem:** BehaviorContext initialization doesn't include `action_bindings` field.

**Fix:** Add to context construction:
```rust
action_bindings: Arc::new(HashMap::new()),  // or Arc::clone if from somewhere
```

Look for patterns like:
```rust
BehaviorContext {
    stage: ...,
    scene_elapsed_ms: ...,
    // ... other fields ...
    keys_down: Arc::new(HashSet::new()),
    // ADD:
    action_bindings: Arc::new(HashMap::new()),
    // ... rest ...
}
```

### How to Find & Fix All Occurrences

```bash
grep -n 'BehaviorContext {' /home/ppotepa/git/shell-quest/engine-behavior/src/lib.rs
```

Will show all construction sites. Check each one and add the `action_bindings` field.

---

## What Still Needs Doing

### TODO 1: Fix Compile Errors (10 min)

1. Fix line 3258: Add action_bindings + queue to ScriptInputApi::new()
2. Find all BehaviorContext {} constructors and add action_bindings field
3. Run `cargo check -p engine-behavior` to verify

### TODO 2: Handle BindInputAction in behavior_runner.rs

**File:** `engine-scene-runtime/src/behavior_runner.rs` around line 230 in `apply_behavior_commands()`

**Add case (stub for now):**
```rust
BehaviorCommand::BindInputAction { action, keys } => {
    // TODO: Store binding for next frame
    // For now just log it
    eprintln!("Binding action '{}' to keys: {:?}", action, keys);
}
```

### TODO 3: Update ScriptInputApi::new() callsite in behavior_runner.rs

**File:** `engine-scene-runtime/src/behavior_runner.rs` — find where input is pushed to scope

Look for pattern like:
```rust
scope.push("input", ScriptInputApi::new(...));
```

And update the call signature to match the new 3-argument version.

### TODO 4: Propagate action_bindings into BehaviorContext

**File:** `engine-scene-runtime/src/behavior_runner.rs` line ~170 where BehaviorContext is constructed

Add:
```rust
let action_bindings = std::sync::Arc::new(HashMap::new()); // placeholder
```

Then in context init:
```rust
ctx = BehaviorContext {
    // ... existing ...
    action_bindings,  // ADD THIS
};
```

### TODO 5: Test compilation

```bash
cd /home/ppotepa/git/shell-quest
cargo check -p engine-behavior
cargo check -p engine
cargo check -p app
```

Should pass with no errors.

### TODO 6: Refactor asteroids-game-loop.rhai

Once compilation passes, simplify the script:

**Replace:**
```rhai
// Manual cd countdown
if shot_cd_ms > 0 { shot_cd_ms -= dt; }

// With:
if entity.cooldown_ready(ship_id, "shoot") {
    spawn_bullet(...);
    entity.cooldown_start(ship_id, "shoot", cfg["shot_cooldown_ms"]);
}
```

**Replace:**
```rhai
// Manual wrap
let new_x = wrap(tx, min_x, max_x);
let new_y = wrap(ty, min_y, max_y);
ship_ref.set_position(new_x, new_y);

// With:
// Just enable once at spawn:
world.enable_wrap(ship_id, min_x, max_x, min_y, max_y);
// Handled automatically each frame
```

**Replace:**
```rhai
// Manual input polling
let thrust_down = input.down("Up") || input.down("w") || ...

// With:
if input.action_down("thrust") {
    // ...
}
```

### TODO 7: Test asteroids in SDL2

```bash
cargo build -p app --features sdl2
cargo run -p app -- --mod-source=mods/asteroids --console-log
```

Expected:
- Game loads
- Ship spawns, responds to input
- No manual cooldown counting needed
- Wrap prevents ship from leaving screen

---

## Key Code Locations

### Engine Implementations (DONE)
- `engine-game/src/components.rs:98-147` — EntityTimers, WrapBounds
- `engine-game/src/gameplay.rs:370-477` — Timer + wrap methods
- `engine/src/systems/gameplay.rs:4-41` — Gameplay system integration

### Behavior System (IN PROGRESS)
- `engine-behavior/src/lib.rs:68` — BehaviorContext.action_bindings field
- `engine-behavior/src/lib.rs:147-153` — BindInputAction command variant
- `engine-behavior/src/lib.rs:1330-1333` — ScriptInputApi struct
- `engine-behavior/src/lib.rs:1717-1761` — ScriptInputApi methods
- `engine-behavior/src/lib.rs:714-729` — Rhai input registrations
- `engine-behavior/src/lib.rs:1050-1098` — Entity + world timer registrations
- `engine-behavior/src/lib.rs:2549-2588` — Entity timer impls
- `engine-behavior/src/lib.rs:2537-2551` — World wrap impls
- `engine-behavior/src/lib.rs:3258` — ⚠️ ScriptInputApi::new() call (NEEDS FIX)
- `engine-behavior/src/lib.rs:3388` — ⚠️ BehaviorContext init (NEEDS FIX)

### Scene Runtime (TODO)
- `engine-scene-runtime/src/behavior_runner.rs:231-310` — apply_behavior_commands()
- `engine-scene-runtime/src/behavior_runner.rs:170-200` — BehaviorContext construction
- `engine-scene-runtime/src/lib.rs:136-149` — UiRuntimeState

### Script (TODO)
- `mods/asteroids/behaviors/asteroids-game-loop.rhai` — Refactor with new APIs

---

## Test Checklist

- [ ] `cargo check -p engine-behavior` passes
- [ ] `cargo check -p engine` passes
- [ ] `cargo build -p app --features sdl2` passes
- [ ] Game loads without errors
- [ ] Ship appears at center
- [ ] Arrow keys turn ship
- [ ] W/Up applies thrust
- [ ] Space fires bullets
- [ ] Ship doesn't leave screen (wrap works)
- [ ] Timers decrement without manual dt math

---

## Next Sprint (Sprint 2)

After Sprint 1 passes all tests:
- **Collision Events** (collision_enter, collision_stay, collision_exit)
- **Damage/Health** (apply_damage, is_dead, emit destroyed event)
- **SplitOnDestroy** (configure on asteroid, happens on death)
- **Prefab Spawn** (factory API for ship, asteroid, bullet, smoke)

This will reduce asteroids script to ~200 LOC (from 498).

---

## Immediate Next Action

1. Fix line 3258: `ScriptInputApi::new()` call signature
2. Fix all `BehaviorContext {}` constructors to include `action_bindings`
3. Run `cargo check -p engine-behavior` to verify
4. If passing, continue with behavior_runner.rs integration
