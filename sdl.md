# SDL2 Renderer Backend — Interchangeable Output Plan

**Goal**: Make the render backend interchangeable so `--output terminal` (default) and `--output sdl2` both work with zero changes to compositor, behaviors, animations, or scene runtime.

**New crates**: 1 (`engine-render-sdl2`)  
**Modified crates**: 7 (`engine-core`, `engine-events`, `engine-pipeline`, `engine-render`, `engine-render-terminal`, `engine`, `app`)

---

## Phase 0: Own Color Type (engine-core)

### Problem
`Cell` in `engine-core/src/buffer.rs:6-10` stores `crossterm::style::Color` for `fg` and `bg`. This makes every buffer consumer depend on a terminal library. The `crossterm` import propagates into ~55 files across engine-core, engine-compositor, engine-pipeline, engine-capture, engine-render, engine-behavior, engine-scene-runtime, and editor.

### What to do

**0.1 — Create `engine-core/src/color.rs`**

Define an engine-owned color enum that mirrors what we actually use:
```
pub enum Color {
    Rgb { r: u8, g: u8, b: u8 },
    Reset,
    // Named variants map to RGB internally:
    Black, Red, Green, Blue, Yellow, Cyan, Magenta, White,
    DarkGrey, DarkRed, DarkGreen, DarkBlue, DarkYellow, DarkCyan, DarkMagenta, Grey,
}
```
Must implement `Debug, Clone, Copy, PartialEq, Default` (default = `Rgb{0,0,0}`).

Add helper methods:
- `Color::rgb(r, g, b) -> Color`
- `Color::to_rgb(&self) -> (u8, u8, u8)` — resolves named variants to RGB values
- `Color::BLACK`, `Color::WHITE` etc. as associated constants

Export via `engine-core/src/lib.rs`: add `pub mod color;` and `pub use color::Color;`

**0.2 — Update `engine-core/src/buffer.rs`**

- Line 1: change `use crossterm::style::Color;` → `use crate::color::Color;`
- `Cell` struct (lines 6-10): now uses engine-owned Color — no API change
- `TRUE_BLACK` constant (line 13): change to `crate::color::Color::Rgb { r: 0, g: 0, b: 0 }`
- `CellDiff` (line 32-36): automatically uses new Color
- All `Buffer` methods (`set`, `fill`, `diff_into`, etc.): signatures unchanged, type changes automatically

**0.3 — Update `engine-core/src/strategy/diff.rs`**

- Line 1 imports `crossterm::style::Color` → change to `crate::color::Color`
- `DiffStrategy::diff_into()` signature uses `Vec<(u16, u16, char, Color, Color)>` — type changes automatically

**0.4 — Update `engine-core/src/scene/color.rs`**

- Line 1: `use crossterm::style::Color;` → `use crate::color::Color;`
- `TermColour` stays as the YAML deserialization type
- `From<&TermColour> for Color` impl (lines 103-135): update to produce `crate::color::Color` variants instead of `crossterm::style::Color` variants
- Rename or keep `TermColour` — it's a scene/authoring concept, still valid

**0.5 — Update all engine-core effects**

24 effect files in `engine-core/src/effects/builtin/*.rs` use `crossterm::style::Color`:
- `blur.rs`, `brighten.rs`, `clear_to_colour.rs`, `crt_on.rs`, `crt_reflection.rs`, `cutout.rs`, `devour.rs`, `fade.rs`, `fade_to_black.rs`, `glitch.rs`, `lightning.rs`, `neon_edge_glow.rs`, `posterize.rs`, `power_off.rs`, `shatter.rs`, `shine.rs`, `terminal_crt.rs`, `whiteout.rs`, `artifact.rs`
- Also `engine-core/src/effects/utils/color.rs`

For each: replace `use crossterm::style::Color;` with `use crate::color::Color;`

Most effects only use `Color::Rgb { r, g, b }` pattern matching and construction — mechanical replacement, no logic changes.

**0.6 — Remove crossterm from engine-core/Cargo.toml**

After all imports are updated, remove `crossterm` from engine-core's dependencies. This is the validation step — if it compiles, the decoupling is complete.

**Verification**: `cargo test -p engine-core`

---

## Phase 1: Own Key Event Type (engine-events)

### Problem
`engine-events/src/lib.rs:5` imports `crossterm::event::KeyEvent` and wraps it in `EngineEvent::KeyPressed(KeyEvent)`. This forces every event consumer to depend on crossterm. The `KeyEvent` type then leaks into:
- `engine/src/systems/scene_lifecycle.rs:9,23` — `key_presses: Vec<KeyEvent>`
- `engine-scene-runtime/src/lib.rs:15` — `use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers}`
- `engine-scene-runtime/src/terminal_shell.rs:223,240,264,637`
- `engine-scene-runtime/src/lifecycle_controls.rs:30,44`
- `engine-scene-runtime/src/ui_focus.rs:32,38`
- `engine-animation/src/menu.rs` (6 matches)
- `editor/src/state/scene_run.rs`, `editor/src/app.rs`, `editor/src/input/keys.rs`

### What to do

**1.1 — Create `engine-events/src/key.rs`**

Define engine-owned key types:
```
pub enum KeyCode {
    Char(char),
    Enter, Backspace, Tab, Esc,
    Up, Down, Left, Right,
    Home, End, PageUp, PageDown,
    Delete, Insert,
    F(u8),
}

pub struct KeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}
```

NOTE: `engine-core/src/scene_runtime_types.rs:127-136` already has `RawKeyEvent` with `code: String, ctrl: bool, alt: bool, shift: bool`. The new `engine_events::KeyEvent` should be the structured version. `RawKeyEvent` can then be derived FROM `KeyEvent` via a simple conversion (the `key_event_to_raw()` function at `engine/src/systems/scene_lifecycle.rs:505-531` already does this for crossterm → RawKeyEvent; just change the source).

**1.2 — Update `engine-events/src/lib.rs`**

- Remove `use crossterm::event::KeyEvent;` (line 5)
- Change `KeyPressed(KeyEvent)` to `KeyPressed(key::KeyEvent)` using the new engine-owned type
- Add `pub mod key;` and re-export `pub use key::{KeyCode, KeyEvent, KeyModifiers};`

**1.3 — Add crossterm → EngineKeyEvent conversion in engine-render-terminal**

Create `engine-render-terminal/src/input.rs`:
- `pub fn crossterm_key_to_engine(key: crossterm::event::KeyEvent) -> Option<engine_events::KeyEvent>`
- Filter out Release events here (currently at game_loop.rs:86)
- Map `crossterm::event::KeyCode` → `engine_events::KeyCode`
- Map `crossterm::event::KeyModifiers` → `engine_events::KeyModifiers`

**1.4 — Update game_loop.rs input polling**

`engine/src/game_loop.rs:83-120` — the crossterm event loop:
- Import the conversion: `use engine_render_terminal::input::crossterm_key_to_engine;`
- Replace `EngineEvent::KeyPressed(key)` with `EngineEvent::KeyPressed(crossterm_key_to_engine(key))` 
- The crossterm polling loop STAYS in game_loop.rs for now (moved behind trait in Phase 4)

**1.5 — Update all consumers of KeyEvent**

Every file that currently imports `crossterm::event::KeyEvent` and matches on it:

| File | Current | Change to |
|------|---------|-----------|
| `engine/src/systems/scene_lifecycle.rs:9,23` | `crossterm::event::{KeyCode, KeyEvent, KeyModifiers}` | `engine_events::{KeyCode, KeyEvent, KeyModifiers}` |
| `engine/src/systems/scene_lifecycle.rs:505-531` | `fn key_event_to_raw(key: &KeyEvent) -> RawKeyEvent` | Same function, match on `engine_events::KeyCode` instead of `crossterm::event::KeyCode` |
| `engine-scene-runtime/src/lib.rs:15` | `use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers}` | `use engine_events::{KeyCode, KeyEvent, KeyModifiers}` |
| `engine-scene-runtime/src/terminal_shell.rs:223,240,637` | Takes `&[KeyEvent]` (crossterm) | Takes `&[engine_events::KeyEvent]` |
| `engine-scene-runtime/src/lifecycle_controls.rs:30,44` | Takes `&[KeyEvent]` (crossterm) | Takes `&[engine_events::KeyEvent]` |
| `engine-scene-runtime/src/ui_focus.rs:32,38` | Takes `&[KeyEvent]` (crossterm) | Takes `&[engine_events::KeyEvent]` |
| `engine-animation/src/menu.rs` | Uses KeyEvent for menu navigation | Use `engine_events::KeyEvent` |

Note: `KeyEventKind::Press | KeyEventKind::Repeat` checks (e.g., terminal_shell.rs:264,640; ui_focus.rs:38) disappear — the conversion function in 1.3 already filters to press events only.

**1.6 — Remove crossterm from engine-events/Cargo.toml and engine-scene-runtime/Cargo.toml**

**Verification**: `cargo test -p engine-events -p engine-scene-runtime -p engine-animation`

---

## Phase 2: Decouple Pipeline from Terminal (engine-pipeline)

### Problem
`engine-pipeline/src/strategies/flush.rs` defines `TerminalFlusher` trait that takes `BufWriter<Stdout>` and `crossterm::Color`. `engine-pipeline/src/strategies/display.rs` defines `DisplayFrame` with `crossterm::Color` tuples. `PipelineStrategies` (mod.rs:27-33) aggregates a `Box<dyn TerminalFlusher>` as a field.

This makes `engine-pipeline` (meant to be backend-agnostic) depend on terminal specifics.

### What to do

**2.1 — Move `TerminalFlusher` trait to engine-render-terminal**

- Move `engine-pipeline/src/strategies/flush.rs` (entire file, 15 lines) to `engine-render-terminal/src/strategy/flush_trait.rs`
- Update `engine-render-terminal/src/strategy/flush.rs` to import from the new location (its `AnsiBatchFlusher` and `NaiveFlusher` already impl this trait)
- The trait can now use `crossterm::style::Color` freely — it's in a terminal crate

**2.2 — Update `DisplayFrame` to use engine-owned Color**

`engine-pipeline/src/strategies/display.rs:1` — change `use crossterm::style::Color;` → `use engine_core::color::Color;`
- `DisplayFrame.diffs` type changes from `Vec<(u16, u16, char, crossterm::Color, crossterm::Color)>` to `Vec<(u16, u16, char, engine_core::Color, engine_core::Color)>`
- `DisplaySink` trait stays in engine-pipeline (it's generic)

**2.3 — Remove `flush` field from `PipelineStrategies`**

`engine-pipeline/src/strategies/mod.rs:32`:
- Remove `pub flush: Box<dyn TerminalFlusher>,` from the struct
- Remove the `flush` parameter from `PipelineStrategies::new()` (line 38) and `from_flags()` (line 56-62)
- Flushing becomes the backend's internal concern, not a pipeline strategy

**Where does the flusher go?** Into the `TerminalRenderer` struct itself. The renderer already owns `stdout` and `async_sink` — it should also own its flusher strategy.

Update `engine-render-terminal/src/renderer.rs`:
- `TerminalRenderer` struct: add field `flusher: Box<dyn TerminalFlusher>`
- `renderer_system()` (line 98): instead of reading flusher from PipelineStrategies, read from renderer's own field

**2.4 — Update diff tuple type throughout pipeline**

All diff tuples `(u16, u16, char, Color, Color)` must use `engine_core::color::Color`:
- `engine-pipeline/src/strategies/display.rs:8` — DisplayFrame.diffs
- `engine-core/src/strategy/diff.rs` — DiffStrategy::diff_into() output type
- `engine-render-terminal/src/renderer.rs:20` — DIFF_SCRATCH thread_local type
- `engine-render-terminal/src/strategy/flush.rs` — TerminalFlusher::flush() takes these tuples

The terminal flusher will need a color conversion step: `engine_core::Color` → `crossterm::style::Color` before writing ANSI sequences.

**2.5 — Create `engine-render-terminal/src/color_convert.rs`**

```
pub fn to_crossterm(c: engine_core::color::Color) -> crossterm::style::Color { ... }
pub fn from_crossterm(c: crossterm::style::Color) -> engine_core::color::Color { ... }
```

The flusher implementations (`AnsiBatchFlusher`, `NaiveFlusher`) call `to_crossterm()` when writing ANSI sequences.

**2.6 — Remove crossterm from engine-pipeline/Cargo.toml**

**2.7 — Update engine/src/strategy/mod.rs re-exports**

`engine/src/strategy/mod.rs:10-18`:
- Remove re-export of `TerminalFlusher` (it now lives in engine-render-terminal)
- `PipelineStrategies` no longer has `flush` field, so `AnsiBatchFlusher`/`NaiveFlusher` stop being re-exported here

**Verification**: `cargo test -p engine-pipeline -p engine-render-terminal`

---

## Phase 3: Generalize Renderer Provider (engine + engine-render-terminal)

### Problem
`RendererProvider` trait (`engine-render-terminal/src/provider.rs:27`) returns `&mut TerminalRenderer` concretely. `EngineWorldAccess` (`engine/src/services.rs:40`) does the same. The renderer system (`engine-render-terminal/src/renderer.rs:98`) is generic over `T: RendererProvider` but accesses concrete terminal internals.

### What to do

**3.1 — Define `OutputBackend` trait in engine-render**

`engine-render/src/lib.rs` already has `RenderBackend` but it's not used. Extend it or create a practical trait:

```
pub trait OutputBackend: Send {
    /// Present a frame's worth of cell diffs to the output surface.
    fn present_diffs(&mut self, diffs: &[(u16, u16, char, Color, Color)]);

    /// Query the output surface size (terminal cols×rows or window pixels÷cell-size).
    fn output_size(&self) -> (u16, u16);

    /// Graceful shutdown (restore terminal, close window, etc.)
    fn shutdown(&mut self);

    /// Paint entire surface black (startup clear).
    fn clear(&mut self);
}
```

This replaces the existing unused `RenderBackend` trait (or extends it — decide based on what makes sense).

**3.2 — Implement `OutputBackend` for `TerminalRenderer`**

In `engine-render-terminal/src/renderer.rs`:
- `impl OutputBackend for TerminalRenderer`
- `present_diffs()`: contains the current flusher logic (lines 154-167)
- `output_size()`: calls `crossterm::terminal::size()`
- `shutdown()`: wraps existing `shutdown()` method
- `clear()`: wraps existing `clear_black()` + `reset_console()`

**3.3 — Change `RendererProvider::renderer_mut()` to return trait object**

`engine-render-terminal/src/provider.rs:27`:
- Change `fn renderer_mut(&mut self) -> Option<&mut TerminalRenderer>;`
- To `fn renderer_mut(&mut self) -> Option<&mut dyn OutputBackend>;`

**3.4 — Update `EngineWorldAccess::renderer_mut()`**

`engine/src/services.rs:40,98,206`:
- Change return type from `Option<&mut TerminalRenderer>` to `Option<&mut dyn engine_render::OutputBackend>`
- `World` impl (line 98-100): `self.get_mut::<TerminalRenderer>()` → need to register as trait object or keep concrete but return as `&mut dyn OutputBackend`

NOTE: This is the trickiest part. The `World` is a type-map (`AnyMap`). You can't `get_mut::<dyn OutputBackend>()`. Options:
- **Option A**: Register `Box<dyn OutputBackend>` in World instead of `TerminalRenderer`. Change `world.register(renderer)` to `world.register(Box::new(renderer) as Box<dyn OutputBackend>)`. Then `world.get_mut::<Box<dyn OutputBackend>>()` works.
- **Option B**: Keep concrete type in World, add a thin wrapper trait with blanket impl. More boilerplate.

**Recommend Option A** — it's cleaner and directly supports SDL2 backend registration.

`engine/src/lib.rs:316-327`:
```rust
// BEFORE:
let mut renderer = TerminalRenderer::new_with_async(self.config.opt_async_display)?;
renderer.reset_console()?;
renderer.clear_black()?;
world.register(renderer);

// AFTER:
let mut renderer = TerminalRenderer::new_with_async(self.config.opt_async_display)?;
renderer.clear();
world.register(Box::new(renderer) as Box<dyn OutputBackend>);
```

**3.5 — Generalize `renderer_system()`**

`engine-render-terminal/src/renderer.rs:98`:
- Current: `pub fn renderer_system<T: RendererProvider>(world: &mut T)` — calls terminal-specific flush
- Change: the diff computation (lines 135-147) is generic. The flush (lines 154-167) calls into `OutputBackend::present_diffs()`
- Debug overlay (lines 113-114) writes to buffer — already generic
- Virtual-to-output (line 99-101) — already generic

The renderer system function can move to `engine-render/src/lib.rs` or stay in engine-render-terminal but call `OutputBackend::present_diffs()` instead of directly accessing `TerminalRenderer` internals.

**Verification**: `cargo test -p engine -p engine-render-terminal`

---

## Phase 4: Backend Selection & Input Abstraction (engine + app)

### Problem
The game loop (`engine/src/game_loop.rs:83-120`) hardcodes crossterm event polling. The startup (`engine/src/lib.rs:316`) hardcodes `TerminalRenderer::new_with_async()`. There's no way to select a backend.

### What to do

**4.1 — Add `BackendKind` to EngineConfig**

`engine/src/lib.rs:82-110`:
```
pub enum BackendKind {
    Terminal,
    Sdl2,
}

pub struct EngineConfig {
    pub output_backend: BackendKind,  // NEW
    // ... existing fields
}
```

Default: `BackendKind::Terminal`

**4.2 — Add CLI flag to app**

`app/src/main.rs`:
```
/// Output backend: terminal (default) or sdl2.
#[arg(long = "output", default_value = "terminal")]
output: String,
```

Map to `EngineConfig.output_backend`.

**4.3 — Define `InputBackend` trait in engine-events**

```
pub trait InputBackend {
    /// Poll all pending input events, converting to EngineEvents.
    /// Returns events since last call. Non-blocking.
    fn poll_events(&mut self) -> Vec<EngineEvent>;
}
```

This lives in `engine-events/src/input_backend.rs`.

**4.4 — Implement `InputBackend` for terminal in engine-render-terminal**

Create `engine-render-terminal/src/input.rs` (or expand what was created in 1.3):
```
pub struct TerminalInputBackend;

impl InputBackend for TerminalInputBackend {
    fn poll_events(&mut self) -> Vec<EngineEvent> {
        // Move crossterm event polling from game_loop.rs here
        // crossterm::event::poll() + event::read() loop
        // Translate crossterm KeyEvent → engine_events::KeyEvent
        // Translate crossterm Resize → EngineEvent::TerminalResized
        // Translate crossterm Mouse → EngineEvent::MouseMoved
    }
}
```

**4.5 — Update game_loop.rs to use InputBackend**

`engine/src/game_loop.rs`:
- Change signature: `pub fn game_loop(world: &mut World, target_fps: u16, input: &mut dyn InputBackend, ...)` 
- Replace crossterm event loop (lines 83-120) with: `let events = input.poll_events(); for ev in events { world.events_mut().unwrap().push(ev); }`
- Remove `use crossterm::event::*` imports
- Remove `MouseCaptureGuard` struct (terminal-specific) — move to TerminalInputBackend
- Remove `is_quit_key()`, `is_debug_fast_forward_toggle()` — move to TerminalInputBackend
- The benchmark results polling loop (lines 58-69) also uses crossterm — abstract similarly

**4.6 — Backend factory in ShellEngine::run()**

`engine/src/lib.rs:155`:
```
match self.config.output_backend {
    BackendKind::Terminal => {
        let mut renderer = TerminalRenderer::new_with_async(...)?;
        renderer.clear();
        world.register(Box::new(renderer) as Box<dyn OutputBackend>);
        let mut input = TerminalInputBackend::new();
        game_loop::game_loop(&mut world, target_fps, &mut input, ...);
    }
    BackendKind::Sdl2 => {
        let mut renderer = Sdl2Backend::new(virtual_w, virtual_h)?;
        world.register(Box::new(renderer) as Box<dyn OutputBackend>);
        let mut input = Sdl2InputBackend::new(...);
        game_loop::game_loop(&mut world, target_fps, &mut input, ...);
    }
}
```

Also move terminal-specific startup (alt-screen enter, splash rendering with crossterm) behind the backend:
- `splash::show_splash()` (`engine/src/splash.rs`) uses crossterm directly — needs terminal guard or skip for SDL2
- Alt-screen enter/leave: part of TerminalRenderer lifecycle

**4.7 — Update engine/Cargo.toml**

- `engine-render-sdl2` becomes an optional dependency: `engine-render-sdl2 = { path = "../engine-render-sdl2", optional = true }`
- Feature flag: `[features] sdl2 = ["dep:engine-render-sdl2"]`

**Verification**: `cargo test -p engine` (terminal backend should work identically)

---

## Phase 5: SDL2 Backend Crate (engine-render-sdl2) — NEW

### What to create

**5.1 — Crate scaffold**

```
engine-render-sdl2/
  Cargo.toml          # deps: sdl2, engine-core, engine-events, engine-render
  src/
    lib.rs            # pub mod renderer; pub mod input; pub mod color_convert;
    renderer.rs       # Sdl2Backend impl OutputBackend
    input.rs          # Sdl2InputBackend impl InputBackend
    color_convert.rs  # engine_core::Color → SDL2 pixel format
```

`Cargo.toml` dependencies:
```toml
[dependencies]
engine-core = { path = "../engine-core" }
engine-events = { path = "../engine-events" }
engine-render = { path = "../engine-render" }
sdl2 = { version = "0.37", features = ["bundled"] }
```

**5.2 — `Sdl2Backend` (renderer.rs)**

```
pub struct Sdl2Backend {
    sdl_context: sdl2::Sdl,
    canvas: sdl2::render::WindowCanvas,
    cell_width: u32,   // pixels per cell column (e.g., 8)
    cell_height: u32,  // pixels per cell row (e.g., 16)
    virtual_w: u16,
    virtual_h: u16,
}
```

`impl OutputBackend`:
- `present_diffs()`: for each diff `(x, y, char, fg, bg)`:
  - Draw a filled rectangle at `(x * cell_width, y * cell_height)` with `bg` color
  - Optionally render the `char` glyph with `fg` color using SDL_ttf (or skip text for pure-pixel mode)
  - Call `canvas.present()` after all diffs
- `output_size()`: returns `(virtual_w, virtual_h)` (or window size ÷ cell size)
- `shutdown()`: drops SDL context
- `clear()`: `canvas.set_draw_color(black); canvas.clear(); canvas.present();`

**Key design decision**: In "pixel mode" (user's preference), each Cell's `bg` color IS the pixel. The `symbol` and `fg` are ignored. This means the existing compositor's cell output translates directly to colored rectangles. The halfblock renderer mode produces 2× vertical resolution because it uses the upper/lower halfblock chars to encode two pixels per cell — for SDL2, this still works: each cell in HalfBlock mode has different fg/bg that represent upper/lower pixel.

For HalfBlock mode specifically, `present_diffs()` would draw:
- Upper half of cell rect with `fg` color (the ▀ character's foreground)
- Lower half with `bg` color (the ▀ character's background)
- This gives 2× vertical resolution automatically

**5.3 — `Sdl2InputBackend` (input.rs)**

```
pub struct Sdl2InputBackend {
    event_pump: sdl2::EventPump,
}
```

`impl InputBackend`:
- `poll_events()`: 
  - `self.event_pump.poll_iter()` 
  - Map `sdl2::event::Event::KeyDown` → `EngineEvent::KeyPressed(engine_events::KeyEvent)`
  - Map `sdl2::event::Event::Quit` → `EngineEvent::Quit`
  - Map `sdl2::event::Event::Window { win_event: Resized(w, h) }` → `EngineEvent::TerminalResized` (reuse same variant or rename to `OutputResized`)
  - Map `sdl2::event::Event::MouseMotion` → `EngineEvent::MouseMoved`

**5.4 — SDL2 key mapping**

Map SDL2 keycodes to `engine_events::KeyCode`:
- `Sdl2Keycode::A..Z` → `KeyCode::Char('a'..'z')`
- `Sdl2Keycode::Return` → `KeyCode::Enter`
- `Sdl2Keycode::Escape` → `KeyCode::Esc`
- `Sdl2Keycode::Up/Down/Left/Right` → `KeyCode::Up/Down/Left/Right`
- `Sdl2Keycode::F1..F12` → `KeyCode::F(1..12)`
- Modifier keys from `event.keymod()`

**Verification**: `cargo build -p engine-render-sdl2`, then `cargo run -p app -- --output sdl2`

---

## Phase 6: Cleanup & Polish

**6.1 — Rename `TerminalResized` event variant**

`engine-events/src/lib.rs:16`: rename to `OutputResized { width, height }` — the resize is backend-agnostic.

**6.2 — Splash screen abstraction**

`engine/src/splash.rs` hardcodes crossterm. Options:
- Skip splash for SDL2 (`if backend == Terminal { show_splash() }`)
- Render splash to Buffer and let the backend present it (cleaner but more work)

Recommend: skip for now, add `if terminal` guard.

**6.3 — Update all re-exports**

`engine/src/systems/renderer.rs:2`: currently `pub use engine_render_terminal::*;`
- Change to only re-export what's needed, or re-export the generic `renderer_system`

`engine/src/strategy/mod.rs:16-18`: remove terminal-specific re-exports that moved.

**6.4 — Update documentation**

- `engine-render/README.md` — document OutputBackend trait
- `engine-render-terminal/README.md` — document as terminal implementation
- `engine-render-sdl2/README.md` — new crate docs
- `ARCHITECTURE.md` — add renderer backend section
- `app/README.AGENTS.MD` — document `--output` flag

**6.5 — Feature-gate SDL2 dependency**

In workspace `Cargo.toml`, add SDL2 as optional. Users who don't need SDL2 don't need SDL2 system libraries installed:
```toml
[features]
default = []
sdl2 = ["engine/sdl2"]
```

Build with: `cargo build --features sdl2`

**Verification**: Full test suite: `cargo test -p engine-core -p engine-events -p engine-pipeline -p engine-render -p engine-render-terminal -p engine -p app`

---

## Dependency Graph (Target State)

```
engine-core (Color, Buffer, Cell — NO crossterm)
    ↓
engine-events (EngineEvent, KeyEvent, InputBackend — NO crossterm)
    ↓
engine-pipeline (DiffStrategy, LayerCompositor, DisplaySink — NO crossterm, NO flush)
    ↓
engine-render (OutputBackend trait, RenderFrame, BackendKind)
   ↓                        ↓
engine-render-terminal       engine-render-sdl2 (optional)
   (crossterm)               (sdl2-rs)
   ↓                        ↓
engine (orchestrator — picks backend via EngineConfig.output_backend)
   ↓
app (CLI: --output terminal|sdl2)
```

## Files Changed Summary

### New files
- `engine-core/src/color.rs`
- `engine-events/src/key.rs`
- `engine-events/src/input_backend.rs`
- `engine-render-terminal/src/input.rs`
- `engine-render-terminal/src/color_convert.rs`
- `engine-render-sdl2/` (entire new crate)

### Files with major changes
- `engine-core/src/buffer.rs` — Color type swap
- `engine-events/src/lib.rs` — own KeyEvent type
- `engine-pipeline/src/strategies/mod.rs` — remove flush field
- `engine-pipeline/src/strategies/flush.rs` — DELETE (moved to engine-render-terminal)
- `engine-pipeline/src/strategies/display.rs` — Color type swap
- `engine-render/src/lib.rs` — OutputBackend trait
- `engine-render-terminal/src/renderer.rs` — implement OutputBackend, own flusher
- `engine-render-terminal/src/provider.rs` — generalize return types
- `engine/src/lib.rs` — backend factory
- `engine/src/services.rs` — trait object instead of concrete TerminalRenderer
- `engine/src/game_loop.rs` — InputBackend instead of crossterm
- `engine/src/systems/renderer.rs` — generic re-exports
- `engine/src/strategy/mod.rs` — remove terminal re-exports
- `app/src/main.rs` — --output flag

### Files with mechanical Color swap (~55 files)
- All `engine-core/src/effects/builtin/*.rs` (24 files)
- `engine-core/src/effects/utils/color.rs`
- `engine-core/src/strategy/diff.rs`
- `engine-core/src/scene/color.rs`
- All `engine-compositor/src/*.rs` that import crossterm::Color (~15 files)
- `engine-capture/src/capture.rs`
- `engine/src/bench.rs`
- `engine/src/systems/compositor/mod.rs`
- `editor/src/domain/preview_renderer.rs`

### Cargo.toml changes
- `engine-core/Cargo.toml` — remove crossterm
- `engine-events/Cargo.toml` — remove crossterm
- `engine-pipeline/Cargo.toml` — remove crossterm
- `engine-scene-runtime/Cargo.toml` — remove crossterm (or reduce to dev-dependency)
- `engine-render-terminal/Cargo.toml` — add engine-events dep
- `engine/Cargo.toml` — add optional engine-render-sdl2 dep
- `Cargo.toml` (workspace) — add engine-render-sdl2 member
