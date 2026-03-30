# Sprint 1: Gameplay Runtime APIs — Handoff

## Status as of this session

**Everything is complete.** All work described in this document has been implemented and merged:

- engine-scene-runtime integration (behavior_runner.rs fixes) ✅
- asteroids script refactor ✅
- Sprint 2 additions (collision events, spawn_child, action bindings) ✅
- Full API polish (KEY_* constants, entity.flag/set_flag, scene.set_vector/batch, world.rand_i, game.get_i/s/b/f, world.any_alive, world.distance, unit_vec32) ✅

See `scripting.md` for the current API reference. This document is kept for historical context only.

---



**Goal:** Reduce asteroids game-loop.rhai boilerplate by moving cooldowns, timers, input actions, and world wrap into engine systems.

---

## COMPLETED ✅

### 1. Components (engine-game/src/components.rs)
✅ EntityTimers + WrapBounds — fully implemented with methods
- Exported in engine-game/src/lib.rs

### 2. GameplayWorld (engine-game/src/gameplay.rs)
✅ 8 cooldown/status methods + 4 wrap methods
✅ tick_timers() implementation (decrements, removes expired statuses)
✅ apply_wrap() implementation (toroidal clamping per entity)
✅ GameplayStore extended with timers + wrap_bounds maps
✅ despawn() cleanup for both

### 3. Gameplay System (engine/src/systems/gameplay.rs)
✅ gameplay_system() calls apply_wrap() then tick_timers() after physics

### 4. Behavior System (engine-behavior/src/lib.rs)
✅ BehaviorCommand::BindInputAction variant added
✅ BehaviorContext.action_bindings field added
✅ ScriptInputApi extended with new fields + methods
✅ ALL Rhai API registrations (input.action_down, bind_action, cooldown/status/wrap)
✅ ScriptGameplayEntityApi methods (cooldown/status)
✅ ScriptGameplayApi methods (enable_wrap/disable_wrap)
✅ BehaviorContext init sites fixed (now includes action_bindings)
✅ ScriptInputApi::new() callsites fixed (now 3 args)

**COMPILES:** ✅ `cargo check -p engine-behavior` passes

---

## IMMEDIATE FIXES NEEDED (5-10 min)

### File: engine-scene-runtime/src/behavior_runner.rs

#### Fix 1: Add action_bindings to BehaviorContext init

**Line:** ~170

**Find:**
```rust
let mut ctx = BehaviorContext {
    stage,
    scene_elapsed_ms,
    // ...
    keys_down,
    // ...
};
```

**Add (before closing brace):**
```rust
    action_bindings: Arc::new(HashMap::new()),
```

#### Fix 2: Handle BindInputAction command

**Line:** ~245 in `apply_behavior_commands()` match block

**Find:** Last command case, e.g. `BehaviorCommand::ScriptError { .. } => { ... }`

**Add after it:**
```rust
                BehaviorCommand::BindInputAction { action, keys } => {
                    // Store binding for next frame's context
                    // TODO: Persist to runtime state
                }
```

---

## VERIFICATION

After fixes, run:
```bash
cd /home/ppotepa/git/shell-quest
cargo check -p engine-scene-runtime
cargo check -p engine
cargo build -p app --features sdl2
```

All should pass with 0 errors.

---

## THEN: Asteroids Script Refactor

File: `mods/asteroids/behaviors/asteroids-game-loop.rhai`

### Before (current, 498 LOC):
```rhai
// Manual cooldown
let shot_cd_ms = to_i(game.get("/ast/shot_cd_ms"));
if shot_cd_ms > 0 { shot_cd_ms -= dt; }

// Manual wrap
let new_x = wrap(tx, min_x, max_x);
ship_ref.set_position(new_x, new_y);

// Manual input
let thrust_down = input.down("Up") || input.down("w") || ...
```

### After (refactored, ~300 LOC):
```rhai
// Cooldowns auto-tick
if entity.cooldown_ready(ship_id, "shoot") {
    spawn_bullet(...);
    entity.cooldown_start(ship_id, "shoot", cfg["shot_cooldown_ms"]);
}

// Wrap auto-applied (enable once at spawn)
world.enable_wrap(ship_id, min_x, max_x, min_y, max_y);

// Actions pre-mapped
if input.action_down("thrust") {
    ship_ref.set_acceleration(fx * thrust, fy * thrust);
}
```

### Changes to make:
1. Remove all manual dt countdown loops (shot_cd, smoke_cd, msg_ttl, invuln, rot_accum)
2. Remove all manual wrap logic (replace with world.enable_wrap() at spawn)
3. Replace all `input.down("key") || input.down("key2")` with `input.action_down("action_name")`
4. Add init-phase action bindings:
```rhai
if scene.entered() {
    input.bind_action("turn_left", ["Left", "a"]);
    input.bind_action("turn_right", ["Right", "d"]);
    input.bind_action("thrust", ["Up", "w"]);
    input.bind_action("fire", [" ", "f"]);
}
```

---

## FILES & LINE NUMBERS

### engine-behavior (DONE ✅)
- Line 68: action_bindings field added to BehaviorContext
- Line 147-153: BindInputAction variant added
- Line 1330-1333: ScriptInputApi updated
- Line 1717-1761: ScriptInputApi impl methods
- Line 714-729: Rhai registrations
- Line 1050-1098: Timer+wrap Rhai registrations
- Line 2549-2588: Entity timer impls
- Line 2537-2551: World wrap impls
- Line 3258-3268: Fixed ScriptInputApi::new() callsite
- Line 3431-3440: Fixed BehaviorContext init
- Line 4467-4468: Fixed test context init

### engine-scene-runtime (TODO)
- **Line 170:** BehaviorContext init — MISSING action_bindings ⚠️
- **Line 245:** apply_behavior_commands() match — MISSING BindInputAction case ⚠️

### mods/asteroids (TODO - after compilation passes)
- asteroids-game-loop.rhai: Refactor with new APIs

---

## NEXT STEPS (IN ORDER)

1. **Fix behavior_runner.rs** (2 locations above)
   ```bash
   cargo check -p engine-scene-runtime
   ```

2. **Verify build**
   ```bash
   cargo check -p engine
   cargo build -p app --features sdl2
   ```

3. **Refactor asteroids-game-loop.rhai**
   - Remove ~150 lines of manual cooldown/wrap/input logic
   - Use new engine APIs

4. **Test**
   ```bash
   cargo run -p app -- --mod-source=mods/asteroids --console-log
   ```
   - Ship spawns ✓
   - Input responds ✓
   - Timers work without dt ✓
   - Wrap prevents leaving screen ✓

---

## After This Sprint Completes

Sprint 2 will add:
- **Collision Events** (collision_enter, etc.)
- **Damage/Health** (apply_damage, is_dead)
- **SplitOnDestroy** (asteroid fragmentation)
- **Prefab Spawn** (factory API)

This reduces game-loop to ~150 LOC and moves core gameplay to engine.

---

## Key Commit

**Commit d6dfa92:** "Sprint 1: Implement cooldown/status/wrap/input-action engine APIs"
- All engine changes complete
- engine-behavior compiles
- Scene runtime integration ready for pickup
