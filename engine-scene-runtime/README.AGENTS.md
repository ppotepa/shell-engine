# engine-scene-runtime: Runtime Scene Materialization & Object Graph

## Overview
Extracted from `engine` (3,602 LOC), this crate manages the runtime state and lifecycle of a loaded scene. It handles:
- **Object Graph** — Materialized tree of scene objects with state snapshots
- **Behavior Execution** — Per-frame behavior updates and command application
- **UI Focus & Input** — Theme management, text input, key routing
- **Terminal Shell** — Integration with sidecar shell simulator
- **3D Camera Control** — OBJ sprite camera pan/orbit/scale
- **Frame State** — Key event tracking, cached snapshots

## Key Types

### Main Struct
- **`SceneRuntime`** — Central facade for scene state (24 fields, 50+ methods)

### Sub-Structs
- **`TerminalShellState`** — Shell I/O, input buffer, history, layout
- **`UiRuntimeState`** — UI theme, focused target, event snapshots
- **`ObjCameraState`** — Camera position, rotation, scale per 3D object
- **`ObjectBehaviorRuntime`** — Behavior instance + state

### Re-exported from engine-core
- **`TargetResolver`** — Path resolution for sprite/layer lookups
- **`ObjectRuntimeState`** — Per-frame mutable sprite state (visibility, offset, etc.)
- **`RawKeyEvent`** — Keyboard input event

## Core Methods (50+)

### Object Graph Access
- `object(id)` — get object by ID
- `objects()` — all objects
- `object_states_snapshot()` — cached Arc of current states (invalidated on mutation)
- `effective_object_state(id)` — state after parent propagation
- `target_resolver()` — cached path resolver

### Behavior Execution
- `update_behaviors()` — run all behaviors for frame
- `apply_behavior_commands()` — process SetVisibility, SetOffset, SetText, etc.
- `apply_mod_behavior_registry()` — resolve pending behaviors against mod registry

### UI Focus & Input
- `ui_theme_id()` — current theme name
- `handle_ui_focus_keys()` — process arrow/tab/enter for text input
- `focused_ui_target_id()` — current text input target

### Terminal Shell
- `has_terminal_shell()` — is shell active in scene?
- `terminal_push_output()` — append shell output line
- `handle_terminal_shell_keys()` — process input for shell
- `sidecar_mark_screen_full()` — signal sidecar screen state

### 3D Camera Control
- `adjust_obj_scale(id, delta)` — zoom 3D object
- `toggle_obj_orbit(id)` — start/stop auto-rotation
- `apply_obj_camera_pan(id, dx, dy)` — move camera

### Frame State
- `set_last_raw_key(event)` — store keyboard event
- `reset_frame_state()` — clear frame-transient state (keys, focus changes, etc.)

## Dependencies
- `engine-core` — Scene model, objects, effects, world types
- `engine-behavior` — Behavior trait, commands
- `engine-behavior-registry` — Mod behavior lookup
- `engine-animation` — SceneStage timeline
- `engine-render-terminal` — GenericMode rendering
- `serde_json` — JSON manipulation

## Architecture: Caching Strategy

**Mutation Generation Counter** (`object_mutation_gen`):
- Incremented on every `apply_behavior_commands()` that mutates state
- Snapshot methods check if gen changed; if not, return cached Arc (zero-copy)
- Example: `object_states_snapshot()` rebuilds only if `object_mutation_gen != cached_object_states_gen`

**Invalidation Pattern**:
```rust
// Before behavior execution:
self.effective_states_dirty = true;
self.cached_object_props = None;
self.cached_object_text = None;

// Apply commands:
for cmd in commands {
    match cmd {
        SetVisibility { target, visible } => {
            self.set_visibility_recursive(target, visible);
            self.object_mutation_gen += 1;  // Bump gen
        }
        // ... etc
    }
}
```

**Result**: Frame with 100 objects but only 5 visible-changes has ~95 unchanged snapshots returned from Arc cache (O(1)).

## Domain Split Plan (Phase 3.5, DEFERRED)

The 3,602 LOC are logically grouped by domain:

| Domain | LOC | Methods |
|--------|-----|---------|
| Object Graph | 1,258 | 15 methods + caching |
| Behavior Running | 1,160 | update + apply + registry |
| UI Focus | 614 | theme, input, focus |
| Terminal Shell | 702 | controls, output, keys |
| Camera 3D | 514 | scale, orbit, pan |
| Text Wrapping | 180 | word-wrap utilities |
| Color Utils | 64 | color parsing |
| Construction | 130 | new() + init |

**Future Plan**: Split into domain modules (object_graph.rs, behavior_runner.rs, etc.) with SceneRuntime as facade. Deferred due to test edge cases in text_wrap logic.

## Testing
- 23 tests covering object graph traversal, effective state computation, behavior commands, UI theme resolution
- Tests verify caching invalidation and snapshot correctness

## Performance Notes

**Bottlenecks Addressed**:
1. O(W×H) object state scan → O(objects) with generation counters
2. Repeated BTreeMap clones → Arc-backed snapshots
3. Nested loop sprite mutations → caching with dirty generation

**Safe for 500+ objects**: Tested with shell-quest-tests mod having deep hierarchy.

## Future Improvements
- Hot-reload scene without resetting runtime state
- Breakpoint/inspect UI for debugging object graph
- Animation timeline system integration
- Async awaitable behavior (e.g., animate_property)
