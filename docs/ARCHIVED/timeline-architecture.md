# Timeline Architecture

## Overview

This document describes the timeline system for sprite visibility in Shell Quest, its architecture, limitations, and validation approach implemented in March 2026.

## Core Timeline Model

### Sprite Timing

Sprites in Shell Quest use **absolute timing** relative to scene start:

```yaml
# Sprite timing example
layers:
  - name: intro
    sprites:
      - type: text
        id: welcome
        content: "Welcome"
        appear_at_ms: 1000    # Appears 1s from SCENE START
        disappear_at_ms: 5000 # Disappears 5s from SCENE START
```

**Key principle**: `appear_at_ms` and `disappear_at_ms` are **absolute timestamps** relative to scene `on_enter` start, NOT relative to layer lifecycle.

### Scene Duration

Scene duration is calculated from the `on_enter` stage:

```rust
// engine-core/src/scene/model.rs
impl Stage {
    pub fn duration_ms(&self) -> u64 {
        self.steps.iter().map(|step| step.duration_ms()).sum()
    }
}

impl Scene {
    pub fn on_enter_duration_ms(&self) -> u64 {
        self.stages.on_enter.duration_ms()
    }
}
```

The `on_enter` stage is the primary cutscene/intro phase where most sprite timing occurs.

### Visibility Check

At render time, sprite visibility is checked in `engine/src/systems/compositor/render/common.rs`:

```rust
pub fn check_visibility(
    appear_at_ms: Option<u64>,
    disappear_at_ms: Option<u64>,
    scene_elapsed_ms: u64,
    hide_on_leave: bool,
    stage: SceneStage,
) -> Option<()> {
    let appear_at = appear_at_ms.unwrap_or(0);
    if scene_elapsed_ms < appear_at {
        return None;  // Sprite hasn't appeared yet
    }
    if let Some(disappear_at) = disappear_at_ms {
        if scene_elapsed_ms >= disappear_at {
            return None;  // Sprite has disappeared
        }
    }
    Some(())
}
```

**Important**: This check is per-sprite, independent of layer visibility.

## Layer Visibility

Layers have a **static visibility flag**:

```rust
// engine-core/src/scene/model.rs
pub struct Layer {
    pub name: String,
    pub z_index: i32,
    pub visible: bool,  // Static flag, no timeline
    pub ui: bool,
    pub stages: LayerStages,
    pub behaviors: Vec<BehaviorSpec>,
    pub sprites: Vec<Sprite>,
}
```

**Limitations**:
- Layers do NOT have `appear_at_ms` / `disappear_at_ms` fields
- Layer visibility is checked at compositor level (`layer_compositor.rs:42-46`)
- When `layer.visible == true`, ALL sprites are rendered independently
- **Layer visibility does NOT cascade timeline control to children**

Layer visibility can be controlled at runtime via:
- Static YAML `visible: false`
- Runtime state `layer_state.visible`
- Rhai scripts: `scene.set("layer-name", "visible", false)`

## Timeline Validation

### Problem: Orphaned Sprites

Before March 2026, sprites could be authored with `appear_at_ms` beyond scene duration:

```yaml
# BAD: Scene ends at 6s but sprite appears at 8.2s
stages:
  on_enter:
    steps:
      - duration: 6000
layers:
  - name: terminal
    sprites:
      - type: text
        id: boot-output
        appear_at_ms: 8200  # ⚠️ Never visible!
        content: "Boot sequence..."
```

**Result**: Sprite definition exists in scene model but is never visible. This can cause confusion and was mistaken for a "sprite leak" bug.

### Solution: Compile-Time Validation

Added timeline validation in `engine-authoring/src/validate/mod.rs`:

```rust
pub enum TimelineDiagnostic {
    SpriteAppearsAfterSceneEnd {
        layer_name: String,
        sprite_index: usize,
        appear_at_ms: u64,
        scene_duration_ms: u64,
    },
    SpriteDisappearsBeforeAppear {
        layer_name: String,
        sprite_index: usize,
        appear_at_ms: u64,
        disappear_at_ms: u64,
    },
}

pub fn validate_sprite_timeline(scene: &Scene) -> Vec<TimelineDiagnostic>
```

Validation runs during `SceneDocument::compile()` (debug mode only) and prints warnings:

```
⚠️  Scene 'intro-cpu-on': sprite #2 in layer 'terminal' has appear_at_ms=8200 
    but on_enter ends at 6000ms (sprite will never be visible)
```

### Checks Performed

1. **SpriteAppearsAfterSceneEnd**: `appear_at_ms >= on_enter_duration`
   - Sprite will never be visible during cutscene phase
   - Common authoring mistake

2. **SpriteDisappearsBeforeAppear**: `disappear_at_ms <= appear_at_ms`
   - Sprite is always hidden (impossible timing)
   - Logic error

### Test Coverage

Three regression tests in `engine-authoring/src/validate/mod.rs`:
- `valid_sprite_timeline_passes` — correct timing passes validation
- `sprite_appears_after_scene_end_warns` — orphaned sprite detected
- `sprite_disappears_before_appear_warns` — backwards timing detected

## Scene Transition Cleanup

Scene transitions properly clean up sprite state via `world.clear_scoped()`:

```rust
// engine/src/systems/scene_lifecycle.rs:158
fn apply_transitions(world: &mut World, transitions: Vec<String>) -> bool {
    for to_scene_ref in transitions {
        let new_scene = world.scene_loader()
            .and_then(|loader| loader.load_by_ref(&to_scene_ref).ok())?;
        
        Self::apply_virtual_size_override(world, &new_scene);
        world.clear_scoped();  // ← Cleans up SceneRuntime
        world.register_scoped(SceneRuntime::new(new_scene));
        world.register_scoped(Animator::new());
    }
}
```

`world.clear_scoped()` drops all scoped resources (including SceneRuntime with its sprite definitions). There is **no sprite state leak** between scenes — orphaned sprites are purely an authoring issue.

## Architectural Limitations

### What Works ✅

- Static `layer.visible` flag skips entire layer rendering
- Runtime `layer_state.visible` control via behaviors/Rhai
- Per-sprite `appear_at_ms` / `disappear_at_ms` within visible layers
- `hide_on_leave` flag for transition cleanup
- Scene transition cleanup via `clear_scoped()`
- **Compile-time validation catches authoring errors**

### What Doesn't Work (By Design) ❌

- **No layer timeline**: Layers cannot have `appear_at_ms` / `disappear_at_ms` in YAML
- **No relative timing**: Sprite timing is always scene-absolute, never layer-relative
- **No hierarchical visibility**: `layer.visible = false` doesn't force-hide children during their timeline windows
- **No automatic clamping**: Sprites with `appear_at_ms > scene_duration` are not auto-adjusted

## Future Improvements (Not Implemented)

These would require significant engine redesign:

### 1. Layer Timeline Support

```yaml
# Hypothetical future syntax
layers:
  - name: chapter-1
    appear_at_ms: 0
    disappear_at_ms: 10000
    sprites:
      - type: text
        appear_at_ms: 1000  # Relative to layer start? Or absolute?
```

**Challenges**:
- Should sprite timing be relative to layer or still absolute?
- How do layer stages interact with sprite stages?
- Does `layer.disappear_at_ms` force-hide all children?

### 2. Relative Sprite Timing

```yaml
# Hypothetical: sprite timing relative to layer
layers:
  - name: intro
    appear_at_ms: 5000
    sprites:
      - type: text
        relative_appear_at_ms: 1000  # 6s absolute (layer + sprite)
```

**Challenges**:
- Two timing modes (absolute vs relative) increase complexity
- Harder to reason about when debugging
- More authoring modes = more confusion

### 3. Timeline Compiler

Compile-time transformations:
- Auto-clamp sprite timing to scene duration
- Convert relative to absolute timing
- Validate layer/sprite stage interactions
- Generate timeline visualization

**Challenges**:
- Significant compiler complexity
- May hide authoring intent (auto-fixing)
- Harder to debug when compilation is opaque

### 4. Hierarchical Visibility Cascade

```yaml
# Hypothetical: layer.visible cascades to children
layers:
  - name: overlay
    visible: false  # Force-hides ALL sprites regardless of their appear_at_ms
```

**Challenges**:
- Current architecture checks visibility independently
- Would need render pipeline changes
- Interaction with runtime `layer_state.visible` unclear

## Authoring Best Practices

### 1. Keep Sprite Timing Within Scene Duration

```yaml
# Good
stages:
  on_enter:
    steps:
      - duration: 6000

layers:
  - name: main
    sprites:
      - type: text
        appear_at_ms: 1000
        disappear_at_ms: 5500  # ✅ Within 6s scene duration
```

### 2. Use Debug Builds for Validation

Validation only runs in debug mode:
```bash
cargo build  # Debug mode: validation warnings
cargo build --release  # Release: no validation overhead
```

### 3. Layer Visibility for Runtime Control

Use Rhai to control layer visibility dynamically:
```rhai
// Hide layer after condition
if game.get("tutorial_complete") {
    scene.set("tutorial-overlay", "visible", false);
}
```

### 4. Scene Duration = on_enter Total

Remember: scene duration for timing purposes is `on_enter` stage only:
- `on_idle` timing is event-driven (any-key, timeout)
- `on_leave` is for transition effects (usually brief)
- Most sprite timing should target `on_enter`

## Implementation History

**March 2026**: Timeline validation system implemented
- Added `Stage::duration_ms()` and `Scene::on_enter_duration_ms()`
- Created `validate_sprite_timeline()` with two diagnostic checks
- Integrated validation into `SceneDocument::compile()`
- Added regression tests (3 test cases)
- All tests pass (206 engine, 73 authoring, 79 core)

**Root cause identified**: VGA sprite "leak" in boot sequence was authoring error (sprite `appear_at_ms: 8200` in 6s scene), not engine bug.

**Commits**:
- `e488eab` feat(authoring): add sprite timeline validation

## See Also

- `scene-centric-authoring.md` — Full scene YAML contract
- `authoring.md` — Metadata-first authoring approach
- `AGENTS.md` — Tooling commands and workflow
