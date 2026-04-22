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
- `BehaviorContext` — frame-local snapshot of stage, timing, object state, UI state, key state, game state, gameplay world, collision hits, and mouse position (`mouse_x: f32`, `mouse_y: f32` in output-space pixels)
- `BehaviorCommand` — side-effect envelope such as `SetVisibility`, `SetOffset`, `SetText`, `SetGuiValue`, and script errors
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

- typed runtime root (`runtime.scene`, `runtime.world`, `runtime.services`, `runtime.stores`) with top-level aliases kept for compatibility,
- GUI widget helpers (`gui.slider_value`, `gui.toggle_on`, `gui.button_clicked`,
  `gui.has_change`, `gui.changed_widget`, `gui.widget_hovered`, `gui.widget_pressed`,
  `gui.set_widget_value`, `gui.set_panel_visible`, `gui.mouse_x/y/mouse_left_down`),
- gameplay world helpers (`world.spawn_visual`, `world.spawn_object`, `world.entity`, `world.objects`, query/count APIs, `world.any_alive`, `world.distance`),
- gameplay ownership helpers (`world.set_controlled_entity`, `world.controlled_entity`, `world.clear_controlled_entity`) for scene-agnostic active actor selection,
- dedicated vehicle-domain helpers (`vehicle.set_active`, `vehicle.active`, `vehicle.clear_active`) layered over the same runtime ownership seam,
- typed gameplay component helpers (`world.set_transform`, `world.set_physics`,
  `world.set_collider_circle`, `world.set_lifetime`, `world.set_visual`, `world.bind_visual`,
  `world.attach_controller` with support for flat arcade config or grouped vehicle stack maps like `arcade`, `angular_body`, `linear_brake`, `thruster_ramp`),
- atomic spawn (`world.spawn_visual(kind, template, data)` — creates entity + visual + binding + transform + collider in one call),
- auto-despawn (`world.despawn_object(id)` and `entity.despawn()` auto-clean all bound scene visuals),
- entity ref API (`world.entity(id)` returns typed handle with `get_i`, `get_f`, `get_s`, `get_b`, `flag`, `set_flag`, `set_many`, `data`, `set_position`, `set_velocity`, `despawn`, `id`, cooldown/status timers, arcade controller, etc.),
- runtime scene handles (`runtime.scene.objects.find/all/by_tag/by_name`) and runtime world handles (`runtime.world.objects.find/all/by_tag/by_name`) for live discovery and mutation,
- scene-root helpers (`scene.object`, `scene.objects.find/all/by_tag/by_name`, `scene.inspect`, `scene.region`, `scene.instantiate`, `scene.despawn`, `scene.set_bg`) backed by the same runtime-first scene surface,
- collision events (`world.collision_enters/stays/exits(kind_a, kind_b)`) — kind-filtered, named-field maps,
- toroidal wrap (`world.enable_wrap_bounds`, `world.set_world_bounds`, `world.enable_wrap`, `world.disable_wrap`),
- RNG (`world.rand_i`, `world.rand_seed`),
- tags (`world.tag_add`, `world.tag_remove`, `world.tag_has`),
- children (`world.spawn_child`, `world.despawn_children`),
- input actions (`input.bind_action`, `input.action_down`) with `KEY_*` constants,
- debug helpers (`diag.info/warn/error`, `diag.layout_info/warn/error`) which surface in the runtime debug console and `Layout` overlay panel,
- game state typed getters (`game.get_i/s/b/f`),
- audio controls (`audio.cue`, `audio.event`, `audio.play_song`, `audio.stop_song`),
- Rhai module system (`import "module-name" as alias;` resolves from `{mod}/scripts/` directory),
- standalone math/geometry functions (`unit_vec32`, `sin32`, `clamp_i`, `clamp_f`, `rotate_points`, etc.).

See `SCRIPTING-API.md` at repo root for the full API reference.

Vehicle-specific stack parsing now routes through
`engine-vehicle::assembly::VehicleAssemblyPlan` from `scripting::vehicle`, so
new neutral assembly/input/handoff helpers can be adopted there without
growing more vehicle glue inside `gameplay_impl.rs`.

The intended boundary is:

- `engine-api` owns the root runtime handle contracts plus the script-facing
  `runtime.*`, `scene.*`, `world.*`, and `vehicle.*` surfaces,
- `engine-game` owns only primitive components plus cached/projected vehicle DTOs,
- `engine-behavior` adapts typed vehicle assembly/config maps onto those
  primitives and should not re-own vehicle control semantics locally,
- required ship runtime passthrough for mods still comes through
  `engine-api::vehicle.*`; `engine-behavior` only forwards that surface when it
  registers Rhai.

## Script API notes that matter in practice

- `runtime` is the canonical root. `runtime.scene`, `runtime.world`,
  `runtime.services`, and `runtime.stores` are the behavior-side entry points.
  Top-level `scene`, `world`, `game`, `level`, `persist`, `input`, `gui`, `ui`,
  `diag`, `palette`, `audio`, `effects`, and `collision` remain compatibility
  aliases to the same live APIs for now.
- `world.set_world_bounds` uses the natural Rhai argument order
  `min_x, min_y, max_x, max_y`.
- `spawn_prefab("ship", #{ cfg: ... })` deep-merges `args["cfg"]` into the
  catalog controller config. This is the intended path for per-level controller
  tuning and nested overrides.
- `runtime.scene.objects.find(target)` is the primary live scene-handle path.
  `scene.object(target)` is the concise shorthand for the same live handle.
  `scene.inspect(target)` stays a snapshot and will not reflect pending
  same-frame handle writes.
- There is no standalone `objects` compatibility map in scope anymore. Use
  `runtime.scene.objects.*`, `scene.objects.*`, `runtime.world.objects.*`, or
  `world.objects.*` depending on the domain you want.
- `world.objects.find(target)` is the lookup-oriented gameplay object surface.
  Use `world.entity(id)` when the script needs transform/physics/controller
  methods rather than generic data inspection and mutation.
- `world.spawn_visual(...)` and `world.set_visual(...)` target the runtime clone
  layer/object, not an arbitrary child sprite inside that clone. Use scene-side
  sprite `id` values when mutating a specific child after spawn.
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
