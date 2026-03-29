# Asteroids Game Modernization — Handoff Document

## Executive Summary
The Asteroids mod in Shell Quest was broken after incomplete Rhai scripting API modernization. Preflight checks pass, but gameplay doesn't work: ship stuck in center, asteroids invisible, no input response. Root cause is **component state desync**: scripts still write x/y/vx/vy to entity state maps, but engine physics operates on Transform2D/PhysicsBody2D components, causing conflicting position updates.

## Work Completed

### Phase 1: Fixed Low-Level Crashes (DONE ✅)
- **Despawn crashes**: Replaced 6x undefined `despawn_entity_visual()` calls with `world.entity(id).despawn()` (A1 API)
- **Type mismatches**: Added `to_i(f64)` and `abs_i(f64)` overloads to Rhai engine-behavior
- **Helper signatures**: Fixed `smoke_points()`, `flash_fill_hex()` call sites in render-sync
- **Result**: Preflight checks pass, no Rhai compile/runtime errors

### Phase 2: Attempted Float Migration (PARTIAL)
- Migrated ship state from fixed-point (`ship_x_fp`, `vx_fp`) to floats (`ship_x`, `vx`)
- Updated thrust/max-speed/bullet-speed to use `clamp_f` instead of `clamp_i`
- Fixed input key codes: normalized to `"Left"`, `"Up"`, `"Space"`, etc.
- Updated game.set/get paths to use `/ast/ship_x` (float) instead of `/ast/ship_x_fp`
- **Problem**: Asteroid state still uses entity map (ast["x"], ast["y"]) but physics updates position via Transform2D

### Phase 3: Identified Root Cause (IN PROGRESS 🔴)
**The Core Issue**: Two competing position sources
1. **Entity state maps**: Scripts read/write `ast["x"]`, `ast["y"]`, `ast["vx"]`, `ast["vy"]` from entity.get("/")
2. **Physics components**: Engine updates Transform2D.x/y and PhysicsBody2D.vx/vy each frame
3. **Conflict**: world.set_transform() writes to Transform2D, then next frame entity state still has stale x/y

**Why This Breaks Gameplay**:
- Asteroids spawn at positions read from entity state maps (ast["x"], ast["y"])
- Engine physics loop moves them (Transform2D updates)
- Render-sync reads from Transform2D for rendering (correct positions)
- BUT: Game loop reads from entity state maps for collision, spawn logic, state updates
- Result: Positions diverge; visuals appear centered (Transform2D often resets); spawning logic fails

**Current Script State** (mods/asteroids/behaviors/):
```
asteroids-game-loop.rhai: 724 LOC
  - Lines 257-261: Init ship x/y as floats in game state (GOOD)
  - Lines 241-273: Config loads thrust/max_speed/bullet_speed as floats (GOOD)
  - Lines 450-460: Thrust loop uses clamp_f, float math (GOOD)
  - Lines 503-577: Asteroid loop reads/updates ast["x"], ast["y"], ast["vx"], ast["vy"] (BAD)
  - Lines 561: world.set_transform() writes to Transform2D (GOOD)
  - Lines 563-573: entity.set_many() writes ast["x/y/vx/vy"] back to entity state (BAD - conflicts)
  - Lines 593-640: Collisions use entity refs, work correctly (GOOD)
  - Lines 670-680: End-of-frame readback reads from game.get("/ast/ship_x") for HUD (GOOD for ship, but asteroid logic still broken)

asteroids-render-sync.rhai: 99 LOC
  - Rewritten to modern API (GOOD)
  - Reads from entity.data() (GOOD)
  - Only updates visual properties: vector.visible, vector.points, vector.fg (GOOD)
  - Delegates position sync to engine A3 visual_sync system (CORRECT)
```

## What Is Working ✅

1. **Engine APIs are solid**:
   - `entity.set_position(x, y)` works (modifies Transform2D)
   - `entity.set_velocity(vx, vy)` works (modifies PhysicsBody2D)
   - `world.collisions_between(kind1, kind2)` returns correct hits
   - `world.set_many()` batches state updates
   - `entity.data()` returns all current state (components + custom fields)

2. **Preflight validation passes**:
   - All 4 Rhai scripts compile without errors
   - Scene graph, levels, audio, effects all verified

3. **Collision system works**:
   - Ship-asteroid collisions fire correctly
   - Bullet-asteroid collisions fire correctly
   - Physics bodies have correct colliders

4. **Render-sync is modern**:
   - Properly reads ship visibility from entity state
   - Correctly syncs asteroid rotations and flash colors
   - No manual position writes (delegates to A3)

## What Doesn't Work ❌

1. **Asteroids don't render**:
   - Reason: Positions stay at center or incorrect because of state/component conflict
   - Expected: Asteroids spawn at edges, move across screen
   - Actual: Either invisible or stuck in place

2. **Ship doesn't move**:
   - Reason: Thrust updates game.get("/ast/vx") but engine physics may overwrite it
   - Expected: Ship moves when W/Up pressed
   - Actual: No visible movement

3. **No visual feedback**:
   - Reason: Render-sync reads from Transform2D (correct) but render-sync runs per-frame; asteroid positions in entity state diverge
   - Expected: Asteroids visible, moving, rotating
   - Actual: Blank or stuck

4. **Wave spawning broken**:
   - Reason: `spawn_asteroid_entity()` uses entity state ast["x"], ast["y"] which are stale
   - Expected: New wave spawns at borders after previous destroyed
   - Actual: Spawns fail or invisible

## The Fix Required 🎯

**Complete rewrite of asteroid and ship physics to eliminate entity state maps**:

### For Asteroids:
1. Stop reading/writing `ast["x"]`, `ast["y"]`, `ast["vx"]`, `ast["vy"]` from entity.get("/")
2. Use `entity.set_position(x, y)` and `entity.set_velocity(vx, vy)` instead
3. Read current position/velocity from `world.transform()` and `world.physics()` when needed
4. Store only game-logic state: rotation phase, flash timers, collision cooldowns, split_pending
5. Let engine physics handle integration

### For Ship:
1. Already migrated to float state in game.get("/ast/ship_x/y/vx/vy") ✓
2. MUST use `ship_ref.set_velocity(vx, vy)` instead of world.set_physics() with magic params
3. Read current position from `world.transform(ship_id)` for collision/spawn logic instead of game state

### For Input:
1. Already normalized to standard key codes ✓
2. `input.down("Left")`, `input.down("Up")`, `input.down(" ")` all work ✓

### Render-sync:
1. Already correct ✓
2. No changes needed

## Files to Modify

```
mods/asteroids/behaviors/asteroids-game-loop.rhai
  - Line ~500-577: Asteroid loop — eliminate ast["x/y/vx/vy"] reads/writes
  - Line ~561: Keep world.set_transform() call (correct)
  - Line ~563-573: REMOVE entity.set_many() call that writes x/y/vx/vy
  - Line ~530-548: Fragment split spawning — use component reads instead of entity state
  - Total savings: ~150 LOC expected

mods/asteroids/scripts/asteroids-shared.rhai
  - No changes needed (helpers are generic)

mods/asteroids/behaviors/asteroids-render-sync.rhai
  - No changes needed (already modern)
```

## Estimated Effort

- **Rewrite asteroid loop**: ~2-3 hours (understand component read/write, test spawning, test collisions)
- **Testing**: ~1 hour (smoke test, verify spawning, verify collisions, verify HUD)
- **Result**: Playable game, ~150 LOC reduction, all modern APIs in use

## Key Insights from This Work

1. **Component vs. State Map Conflict**: 
   - Engine provides Transform2D/PhysicsBody2D components for structured movement
   - Scripts were mixing custom entity state maps with component reads
   - This desync breaks position consistency across physics, rendering, and logic

2. **Rhai API Surface**:
   - `entity.set_position()` / `entity.set_velocity()` modify components directly (preferred)
   - `world.set_transform()` / `world.set_physics()` also work but less convenient for entity references
   - `entity.set_many()` is for custom game state, not physics

3. **Render-sync Architecture**:
   - A3 visual_sync_system handles Transform2D → scene position automatically
   - Manual position writes in render-sync are unnecessary and conflict with A3
   - Render-sync should ONLY sync visual properties (color, rotation, visibility, effects)

4. **Input Normalization**:
   - Keys normalize to: "a", "d", "w", "Left", "Up", " ", "f", etc.
   - `input.down(code)` and `key.code` use same normalized strings

## Next Agent Checklist

- [ ] Read this handoff completely
- [ ] Understand the state/component conflict (core issue)
- [ ] Rewrite asteroid loop to use component APIs (Transform2D reads, set_position/set_velocity calls)
- [ ] Test: `cargo run -p app --release -- --mod-source mods/asteroids --start-scene /scenes/game/scene.yml --audio`
- [ ] Verify: Ship moves, asteroids spawn and animate, collisions work, HUD updates, wave progression works
- [ ] Commit with summary: "Remove entity state maps, use component APIs for physics"
- [ ] Update this doc with final metrics

## Session Artifacts

- **Prior checkpoint**: 005-fix-broken-asteroids-rhai-scri.md (describes Phase 1-2)
- **Git commits**:
  - e999bee: Use to_i() Rhai built-in
  - 95d2a5b: Float migration attempt
  - 6799651: Render-sync rewrite + despawn fixes
  - 95100e4: Collision API implementations
- **Modified files in this session**:
  - engine-behavior/src/lib.rs: Added to_i(f64), abs_i(f64) overloads
  - mods/asteroids/behaviors/asteroids-game-loop.rhai: Float state + despawn fixes
  - mods/asteroids/behaviors/asteroids-render-sync.rhai: Complete rewrite (modern API)

---

**Status**: Ready for Phase 4 (Clean Component Rewrite)  
**Blocker**: Entity state/component desync in asteroid logic  
**Owner**: Next agent
