# engine-behavior

Behavior runtime, built-in behaviors, and Rhai script execution.

## Purpose

`engine-behavior` owns the per-frame behavior model used by scenes. It defines:

- the `Behavior` trait,
- the `BehaviorContext` shared with each behavior tick,
- the `BehaviorCommand` queue used to report side effects back to the engine,
- built-in behavior implementations,
- Rhai-backed scripted behaviors,
- the built-in behavior factory and metadata lookup.

This crate does not own `World`; it operates on typed context snapshots and
emits commands for higher-level systems to apply.

## Key Types

- `Behavior` — per-tick behavior interface
- `BehaviorContext` — frame-local snapshot of stage, timing, object state, UI state, key state, game state, gameplay world, and collision hits
- `BehaviorCommand` — side-effect envelope such as `SetVisibility`, `SetOffset`, `SetText`, terminal output commands, and script errors
- `RhaiScriptBehavior` — mod or scene-defined scripted behavior
- `SceneAudioBehavior` — built-in scene audio cue emitter
- `BuiltInBehaviorFactory` — authoritative dispatcher for engine-defined behavior names

## Built-in behavior model

Built-in behaviors are constructed from `BehaviorSpec` and updated once per
frame. They do not mutate runtime state directly. Instead they push
`BehaviorCommand` values which the scene runtime or engine systems apply later.

That split is important:

- behavior code stays pure-ish and testable,
- runtime mutation stays centralized,
- script failures can be surfaced cleanly through `ScriptError`.

## Working with this crate

When adding or changing a built-in behavior:

1. add or update the behavior type here,
2. register it in `factory::BuiltInBehaviorFactory`,
3. update metadata exposed through `engine-core` authoring catalogs,
4. update authoring docs if the public script or YAML contract changes,
5. add or update behavior tests.

When changing Rhai scope variables or command shapes, keep the contract aligned
with the runtime behavior system and authored YAML expectations.

Current script-facing API surface includes:

- gameplay world helpers (`world.spawn_visual`, `world.spawn_object`, `world.entity`, query/count APIs, `world.any_alive`, `world.distance`),
- typed gameplay component helpers (`world.set_transform`, `world.set_physics`,
  `world.set_collider_circle`, `world.set_lifetime`, `world.set_visual`, `world.bind_visual`,
  `world.attach_ship_controller`),
- atomic spawn (`world.spawn_visual(kind, template, data)` — creates entity + visual + binding + transform + collider in one call),
- auto-despawn (`world.despawn_object(id)` and `entity.despawn()` auto-clean all bound scene visuals),
- entity ref API (`world.entity(id)` returns typed handle with `get_i`, `get_f`, `get_s`, `get_b`, `flag`, `set_flag`, `set_many`, `data`, `set_position`, `set_velocity`, `despawn`, `id`, cooldown/status timers, ship controller, etc.),
- collision events (`world.collision_enters/stays/exits(kind_a, kind_b)`) — kind-filtered, named-field maps,
- toroidal wrap (`world.enable_wrap_bounds`, `world.set_world_bounds`, `world.enable_wrap`, `world.disable_wrap`),
- RNG (`world.rand_i`, `world.rand_seed`),
- tags (`world.tag_add`, `world.tag_remove`, `world.tag_has`),
- children (`world.spawn_child`, `world.despawn_children`),
- input actions (`input.bind_action`, `input.action_down`) with `KEY_*` constants,
- scene helpers (`scene.set_vector`, `scene.set_visible`, `scene.batch`),
- game state typed getters (`game.get_i/s/b/f`),
- audio controls (`audio.cue`, `audio.event`, `audio.play_song`, `audio.stop_song`),
- Rhai module system (`import "module-name" as alias;` resolves from `{mod}/scripts/` directory),
- standalone math/geometry functions (`unit_vec32`, `sin32`, `clamp_i`, `clamp_f`, `rotate_points`, etc.).

See `scripting.md` at repo root for the full API reference.

## Script API notes that matter in practice

- `world.set_world_bounds` uses the natural Rhai argument order
  `min_x, min_y, max_x, max_y`.
- `spawn_prefab("ship", #{ cfg: ... })` merges `args["cfg"]` into the catalog
  controller config. This is the intended path for per-level controller tuning.
- `local[]` state is per behavior instance. Cross-script coordination should go
  through persistent game state (`game.set/get`) instead of assuming two
  behavior files share locals.
- The engine Rhai surface is intentionally generic. Mod-specific helpers
  belong in mod-side shared Rhai modules.

## Validation

Use the standard crate test command:

```bash
cargo test -p engine-behavior
```

## Integration points

- `engine-scene-runtime` owns attached behavior instances and applies emitted commands
- `engine-behavior-registry` supplies mod-defined behavior sources
- `engine-core` provides scene model types, object metadata, and shared runtime snapshot types
