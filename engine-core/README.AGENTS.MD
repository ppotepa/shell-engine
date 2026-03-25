# Engine Core — Model Definitions & Traits

## Scene Model

**Scene** hierarchy:

```
Scene {
  id: String ("00.intro.logo"),
  title: String,
  virtual_size_override: Option<(u32, u32)>,
  
  stages: Vec<Stage> {
    Stage {
      name: String ("on_enter", "on_idle", "on_leave"),
      duration: Duration,
      steps: Vec<TimelineStep>,
      effects: Vec<Effect>,
    },
    TimelineStep {
      at: Duration,
      duration: Duration,
      effects: Vec<Effect>,
    },
  },
  
  layers: Vec<Layer> {
    Layer {
      id: String,
      visible: bool,
      sprites: Vec<Sprite> {
        Sprite {
          id: String,
          type: SpriteType (Cell, Halfblock, Obj, Gif, Canvas),
          appear_at_ms: i32,
          disappear_at_ms: i32,
          content: SpriteContent,
          style: CellStyle,
          position: (u32, u32),
          size: (u32, u32),
          effects: Vec<Effect>,
        },
      },
    },
  },
  
  objects: Vec<Object>,     // Dynamic objects (behaviors attached)
  behaviors: Vec<Behavior>,  // Scene-level behaviors
  next: Option<String>,      // Next scene ID on completion
}
```

## Sprite Timing

**Key invariant**: Sprite `appear_at_ms` / `disappear_at_ms` are **absolute**, scene-relative.

- Based on `Scene.stages[0].duration` (on_enter stage)
- If `appear_at_ms >= on_enter.duration`, sprite never visible
- Compile-time validation warns of impossible timings

## Layer Ordering

**Render order**: First layer in YAML rendered first (bottom → top stacking).

Layers do NOT have `appear_at_ms`/`disappear_at_ms` — only `visible: bool`.

## Buffer Architecture

**Double-buffer** (back, front):
- Back-buffer: current render target
- Front-buffer: last frame (for diff)
- After composite/postfx: swap with render thread

**Dirty tracking**:
- `dirty_region` bounding box (min, max cell index)
- `dirty_rows` BitSet (per-row dirty flags)
- Enables RowSkipDiff optimization

**Diff strategies** (trait DiffStrategy):
- FullScanDiff: compare all cells (safe)
- DirtyRegionDiff: compare only dirty region
- RowSkipDiff: row-level dirty skip (fastest)

## Strategy Traits

### DiffStrategy

```rust
pub trait DiffStrategy {
    fn diff(&self, back: &Buffer, front: &Buffer) -> Vec<(usize, Cell)>;
}
```

Compares back vs front buffer, returns changed cells.

### LayerCompositor

```rust
pub trait LayerCompositor {
    fn composite(&self, layers: &[Layer], target: &mut Buffer);
}
```

Blits all layers onto target buffer.

Implementations:
- ScratchLayerCompositor: uses thread-local scratch (safe, slower)
- DirectLayerCompositor: direct blit (fast, requires careful dirty tracking)

### HalfblockPacker

```rust
pub trait HalfblockPacker {
    fn pack(&self, buffer: &Buffer) -> String;  // ANSI codes
}
```

Packs buffer cells into ANSI escape sequences.

### VirtualPresenter

```rust
pub trait VirtualPresenter {
    fn should_present(&self, buffer: &Buffer, last: &Buffer) -> bool;
}
```

Decides if buffer changed enough to flush (frame skip optimization).

### TerminalFlusher

```rust
pub trait TerminalFlusher {
    fn flush(&self, ansi: &str) -> std::io::Result<()>;
}
```

Writes ANSI codes to terminal.

### ModEffectFactory

```rust
pub trait ModEffectFactory {
    fn create_effect(&self, name: &str, params: &Map) -> Option<Box<dyn Effect>>;
}
```

Allows mods to define custom effects.

## Built-in Effects

17 built-in effects in `engine-core/src/effects/builtin/`:

| Effect | Purpose |
|--------|---------|
| fade-in, fade-out | Alpha blend transition |
| whiteout, blackout | Color sweep |
| crt-on, crt-off | CRT flicker + overscan |
| scanlines | Interlaced line effect |
| crt-distort | Wavy distortion |
| crt-glitch | Glitch lines |
| shine | Sweeping highlight |
| neon-edge-glow | Volumetric spillover glow |

All implement `Effect` trait: `apply(cell, progress, params) → cell`

## Built-in Behaviors

10 built-in behaviors:

| Behavior | Purpose |
|----------|---------|
| delay | Wait N ms |
| wait-for-input | Block until key press |
| jump-scene | Transition to another scene |
| emit-sound | Play audio file |
| set-sprite-style | Update sprite colors/font |
| rotate-sprite | Spin 3D object |
| animate-sprite | Frame-by-frame animation |
| menu-highlight | Dynamic menu selection |
| tween-position | Smooth position movement |
| rhai-script | User Rhai script |

Execute in `BehaviorSystem::update()` on `world.behaviors`.

## Scoped Cleanup

**Key invariant**: `world.clear_scoped()` on scene exit cleans:
- All sprites, layers, objects
- Scene-level behaviors
- Temporary buffers
- Effect state

No state leaks between scenes.

## Compilation Pipeline

Scene.yml → `engine-authoring/src/compile/scene.rs`:
1. Parse YAML
2. Normalize (fill defaults)
3. Validate (schema, timeline, references)
4. Compile to `Scene` struct
5. Return or error with diagnostics
