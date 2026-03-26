# engine-behavior: Behavior System & Script Runtime

## Overview
Extracted from `engine` (3,719 LOC total), this crate provides the complete behavior scripting system for Shell Quest. It handles:
- **Built-in Behaviors** (10 implementations): Scene stage transitions, audio, shell I/O, camera control, etc.
- **Rhai Script Execution**: Dynamic behavior scripting via Rhai VM
- **Behavior Registry**: Mod-defined behavior lookup and binding
- **Command Queue**: BehaviorCommand dispatch (SetVisibility, SetOffset, SetText, etc.)

## Key Types

### Traits
- **`Behavior`** — Interface for any behavior (on_enter, on_idle, on_leave phases)
- **`BehaviorFactory`** — Factory for creating behavior instances from YAML specs
- **`BehaviorProvider`** — Trait for accessing registered behaviors by ID

### Structs
- **`BehaviorCommand`** — Commands emitted by behaviors (SetVisibility, SetOffset, SetText, etc.)
- **`BehaviorContext`** — Runtime context passed to behavior methods (scene, objects, elapsed time, etc.)
- **`RhaiScriptBehavior`** — Wraps Rhai script execution with scene state bindings
- **`BuiltInBehaviorFactory`** — Factory for 10 built-in behavior types

### Built-in Behaviors (10)
1. **SceneAudioBehavior** — Play/stop/loop audio clips on scene/behavior entry
2. **SetTextBehavior** — Dynamically update sprite text content
3. **SetVisibilityBehavior** — Show/hide sprites
4. **SetOffsetBehavior** — Move sprites by pixel offset
5. **ShellIoControlsBehavior** — Handle terminal shell input/output binding
6. **ObjCameraControlBehavior** — Pan/rotate 3D camera
7. **SceneStageControlBehavior** — Trigger stage transitions (on_enter, on_idle, on_leave)
8. **EmitterBehavior** — Particle emission (placeholder)
9. **SetStageDurationBehavior** — Override stage timing
10. **SceneTransitionBehavior** — Trigger scene load on action

## Dependencies
- `engine-core` — Scene model, game objects, RawKeyEvent
- `engine-animation` — SceneStage timeline
- `engine-behavior-registry` — Mod behavior lookup
- `serde_json` — JSON scene specs
- `rhai` — Script VM

## Usage Example

```rust
// In a scene YAML:
behaviors:
  - id: my-button
    on_enter:
      - type: SetVisibility
        target: button-label
        visible: true
      - type: SetText
        target: button-text
        content: "Click me!"
    on_leave:
      - type: SetVisibility
        target: button-label
        visible: false

// In Rhai script:
on_idle() {
    if key_pressed("space") {
        set_visibility("button", true);
        play_audio("click", false);
    }
}
```

## Architecture Notes

**Behavior Lifecycle**:
1. **Creation** — BehaviorFactory.create() from YAML spec
2. **Binding** — Behavior instance stored in object's behavior list
3. **Execution** — Called by scene_runtime.update_behaviors() each frame
   - `on_enter()` — called once when object becomes visible/active
   - `on_idle()` — called each frame while active
   - `on_leave()` — called when object is hidden/deactivated
4. **Command Emission** — Behavior returns BehaviorCommand vec
5. **Command Application** — Commands processed by scene_runtime.apply_behavior_commands()

**Script Bindings** (Rhai scope):
- `scene_id` — current scene ID
- `object_id` — target object ID
- `stage` — current scene stage index
- `elapsed_ms` — frame elapsed time
- `key_pressed(code)` — check if key was pressed
- `set_visibility(target, visible)` — emit SetVisibility command
- `set_offset(target, dx, dy)` — emit SetOffset command
- `set_text(target, text)` — emit SetText command
- etc.

**Error Handling**: Script compilation/runtime errors emit `BehaviorCommand::ScriptError` which is consumed into `DebugLogBuffer` for engine debug overlay display.

## Testing
- 48 tests covering all 10 built-in behaviors, Rhai script execution, command application
- Located in behavior.rs bottom test block

## Future Improvements
- Hot-reload behavior scripts without scene reload
- Async behavior support (e.g., wait_ms, yield_frame)
- Behavior breakpoints / debugger in editor
- Behavior composition / inheritance
