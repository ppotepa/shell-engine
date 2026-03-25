# Shell Quest Editor — Architecture Brief

## 1) Purpose

This document defines the architecture direction and design principles for the Shell Quest editor.
The editor is evolving from a YAML browser/inspector into a **game authoring tool** that supports
visual timeline editing, modular window management, and live YAML-first workflows.

## 2) Core Principles

### 2.1 YAML-First Authoring

The editor is **not** a standalone asset pipeline. YAML remains the source of truth.

- Runtime consumes compiled scene models from `engine-authoring`.
- Editor reads, displays, and optionally modifies YAML, but never bypasses the authoring contracts.
- All visual editing (timeline, effects, layers) writes back to YAML or operates on transient preview state.

### 2.2 Single Responsibility (SRP)

Each module/window/component has **one clear responsibility**:

- **Window modules** (Explorer, Timeline, Scene Preview, Effects Browser, Inspector) are independent features.
- **State modules** own lifecycle and data for one domain (project, scene preview, timeline, etc.).
- **UI layout/docking** is separate from window content logic.

### 2.3 Modular Window System

Windows are **first-class modules**, not just UI fragments:

- Each window has:
  - Its own state struct (cursor, selection, scroll, view mode)
  - Its own command handlers
  - Its own render logic
  - Optional background workers (live refresh, validation)
- Windows register with a central layout manager.
- Windows can be opened, closed, docked, or floated (future).

## 3) Current State (Implemented)

Editor currently has:

- **Basic project explorer**: mod.yaml, scenes, assets
- **Scene preview**: live rendering with layer toggling
- **Effects browser**: builtin effect inspection with live parameter tweaking
- **File editor**: read-only YAML viewer
- **Start screen**: recent projects + directory picker
- **Live refresh**: ~1.2s polling for filesystem changes

Architecture today:

- `editor/src/state/mod.rs`: root shell + routing + global UI helpers (odchudzony po refaktorze).
- `editor/src/state/start_screen.rs`: launch flow + schema/directory picker + open/close projektu.
- `editor/src/state/project_explorer.rs`: drzewo projektu i wejście do edit mode.
- `editor/src/state/editor_pane.rs`: lifecycle edit mode.
- `editor/src/state/effects_browser.rs`: effect selection/params/live preview lifecycle.
- `editor/src/state/scenes_browser.rs`: scene selection/layer visibility/fullscreen/preview lifecycle.
- `editor/src/state/cutscene.rs`: walidacja źródeł cutscenki.
- `editor/src/state/watch.rs`: polling zmian plików i synchronizacja indeksu.
- `editor/src/domain/preview_renderer.rs`: wspólny runtime preview dla UI.

Aktualny dług techniczny:

- `AppState` nadal agreguje wiele substates (to świadomy shell, nie osobne bounded context per crate).
- Layout manager/docking z sekcji „Target Architecture” jest jeszcze planem, nie wdrożonym systemem.
- Timeline window z sekcji 5 jest roadmapą, nie gotową funkcją produkcyjną.

## 4) Target Architecture

### 4.1 Module Structure

```
editor/
├── src/
│   ├── app.rs                   # Terminal lifecycle, main loop
│   ├── cli.rs                   # CLI args
│   ├── layout/
│   │   ├── manager.rs           # Window docking/focus/visibility
│   │   ├── rect_allocator.rs   # Terminal space splitting
│   │   └── mod.rs
│   ├── windows/                 # Window modules (one per feature)
│   │   ├── explorer/
│   │   │   ├── state.rs
│   │   │   ├── commands.rs
│   │   │   ├── render.rs
│   │   │   └── mod.rs
│   │   ├── timeline/
│   │   │   ├── state.rs
│   │   │   ├── commands.rs
│   │   │   ├── render.rs
│   │   │   └── mod.rs
│   │   ├── scene_preview/
│   │   ├── effects_browser/
│   │   ├── inspector/
│   │   ├── file_editor/
│   │   └── mod.rs
│   ├── services/                # Shared background workers
│   │   ├── file_watcher.rs
│   │   ├── validation.rs
│   │   └── mod.rs
│   ├── io/                      # File system, YAML, indexing
│   ├── domain/                  # Asset index, diagnostics
│   ├── input/                   # Key mapping, command routing
│   └── ui/                      # Theme, icons, shared widgets
```

### 4.2 State Organization

Replace monolithic `AppState` with:

- **EditorState**: top-level coordinator
  - Active mode (Start, Workspace)
  - Layout config (which windows open, docking)
  - Global shortcuts
- **Window states**: each window owns its state
  - `ExplorerState`, `TimelineState`, `ScenePreviewState`, etc.
- **Shared services**: background tasks
  - `FileWatcher`, `ValidationService`, `ProjectIndex`

Example:

```rust
pub struct EditorState {
    pub mode: EditorMode,
    pub layout: LayoutManager,
    pub windows: WindowRegistry,
    pub services: Services,
}

pub struct WindowRegistry {
    pub explorer: Option<ExplorerWindow>,
    pub timeline: Option<TimelineWindow>,
    pub scene_preview: Option<ScenePreviewWindow>,
    // ...
}
```

### 4.3 Command Routing

Commands flow:

1. **Key press** → `input/keys.rs` → `Command`
2. **Command** → `EditorState.route_command(cmd)`
3. **Route to window** → active window handles or ignores
4. **Fallback to global** → editor-level handlers (quit, layout toggle)

Each window implements:

```rust
trait Window {
    fn handle_command(&mut self, cmd: Command) -> bool; // true if handled
    fn render(&self, frame: &mut Frame, area: Rect);
    fn title(&self) -> &str;
}
```

### 4.4 Layout Management

`LayoutManager` orchestrates screen space:

- Tracks open windows
- Assigns terminal rects to windows
- Handles focus cycling
- Supports docking policies (sidebar, center, split)

No window knows about layout — it only renders in the rect given.

## 5) Timeline Window (New Feature)

### 5.1 Purpose

Timeline is the **game authoring UI** for scene composition.

It displays:

- Scene stages (`on_enter`, `on_idle`, `on_leave`)
- Steps within each stage (duration, effects)
- Layers and sprites at each step
- Effect timings

Users can:

- Navigate stages/steps/layers/sprites
- Add/remove/reorder effects
- Adjust effect parameters
- Preview effect timing visually
- Write changes back to scene YAML

### 5.2 Timeline State

```rust
pub struct TimelineState {
    pub scene_ref: String,
    pub scene: Scene,
    pub selected_stage: usize,
    pub selected_step: usize,
    pub selected_layer: Option<usize>,
    pub selected_sprite: Option<usize>,
    pub selected_effect: Option<usize>,
    pub playhead_ms: u64,
    pub zoom: f32,
    pub inspector_open: bool,
}
```

### 5.3 Timeline UI

Layout (horizontal split):

- **Left 25%**: Stage/Step tree navigator
- **Center 50%**: Timeline visualization (bars, effects, keyframes)
- **Right 25%**: Effect inspector (parameters, targets)

### 5.4 Timeline Commands

- `j/k`: Navigate stages/steps/layers
- `Enter`: Edit selected item (effect params, sprite properties)
- `a`: Add effect to selected step
- `d`: Delete selected effect
- `Space`: Toggle playhead preview
- `+/-`: Zoom timeline
- `Tab`: Cycle timeline panes

### 5.5 YAML Write-Back

When user modifies timeline:

1. Update in-memory `Scene` model
2. Serialize modified scene to YAML
3. Write to disk
4. Trigger file watcher → refresh other windows

This keeps YAML as source of truth while providing visual editing.

## 6) Refactor Roadmap

### Phase 1: State Decomposition (High Priority)

**Goal**: Break `state/mod.rs` into focused modules.

Tasks:

1. Extract `StartScreenState` → `windows/start/state.rs`
2. Extract `ExplorerState` → `windows/explorer/state.rs`
3. Extract `ScenePreviewState` → `windows/scene_preview/state.rs`
4. Extract `EffectsBrowserState` → `windows/effects_browser/state.rs`
5. Extract `FileWatcher` → `services/file_watcher.rs`
6. Create `EditorState` coordinator

**Risk**: LOW (pure refactoring)  
**Outcome**: `state/mod.rs` shrinks from 1764 → ~400 lines

### Phase 2: Window Module Interface (High Priority)

**Goal**: Standardize window protocol.

Tasks:

1. Define `Window` trait
2. Implement trait for existing windows
3. Create `WindowRegistry`
4. Create `LayoutManager` with basic docking
5. Update `ui/mod.rs` to iterate windows

**Risk**: MEDIUM (new abstraction)  
**Outcome**: Windows become pluggable modules

### Phase 3: Timeline Window (Medium Priority)

**Goal**: Add timeline authoring UI.

Tasks:

1. Create `windows/timeline/` module
2. Implement stage/step/effect navigator
3. Implement timeline bar visualization
4. Implement effect inspector pane
5. Add YAML write-back logic
6. Hook into file watcher for live refresh

**Risk**: MEDIUM (new feature)  
**Outcome**: Visual scene/effect editing

### Phase 4: Validation Pipeline (Medium Priority)

**Goal**: Show live authoring errors.

Tasks:

1. Create `services/validation.rs`
2. Hook into `engine-authoring` compile pipeline
3. Display diagnostics in inspector/status
4. Highlight invalid YAML in file editor

**Risk**: LOW (leverages existing pipeline)  
**Outcome**: Faster authoring feedback

### Phase 5: Layout Persistence (Low Priority)

**Goal**: Save/restore window layout.

Tasks:

1. Add layout serialization
2. Save to `~/.config/sq-editor/layout.toml`
3. Restore on startup

**Risk**: LOW  
**Outcome**: Personalized workspace

## 7) Design Constraints

### 7.1 Terminal Limitations

Editor runs in a terminal, not a GUI:

- No floating windows (yet)
- No mouse dragging for resize (yet)
- Limited color/styling
- No GPU acceleration for previews

Keep UI keyboard-first and text-based.

### 7.2 YAML Authoring Pipeline

Editor must respect `engine-authoring` contracts:

- Never bypass scene compilation
- Never invent non-standard YAML fields
- Always validate before write-back
- Always preserve human-readable YAML formatting

### 7.2.1 Scene-centric implications

The editor should increasingly treat authored scenes as:

- `scene.yml` = orchestration surface
- `layers/*.yml` = visual modules
- `objects/*.yml` = reusable prefabs

That has a few practical consequences:

- scene layer order should come from authored `scene.layers`, not inferred file order
- preview should compile through the same resolver path as runtime (`scene.layers[].ref`, `scene.objects`, `layer.objects`)
- authoring errors like missing refs or cycles should be visible as diagnostics, not silently hidden by the UI
- layer/object editing UX should map to the real YAML contract instead of inventing editor-only concepts

### 7.3 Live Refresh

Editor polls filesystem ~1.2s for changes:

- External edits (VSCode, git) should refresh lists/previews
- Editor writes should trigger same refresh
- No conflicts if user edits YAML externally

### 7.4 Performance

Editor should remain responsive on large mods:

- Lazy-load scenes (don't parse all on startup)
- Debounce validation (don't revalidate every keystroke)
- Cache compiled scenes for preview

## 8) Alignment with AGENTS.md

The editor refactor aligns with repository guidance:

- **§6 Editor Architecture**: Already follows `app/cli/domain/io/input/state/ui` structure; refactor deepens this.
- **§5 Mod Structure**: Editor respects `mod.yaml`, `scenes/`, `assets/` contracts.
- **§2 YAML-First**: Editor reads/writes YAML, runtime consumes compiled models.
- **§10 Change Playbook**: Timeline/validation changes touch authoring pipeline, not runtime.

## 9) Next Steps

### Immediate (This Sprint)

1. Review this brief with team
2. Start Phase 1: state decomposition
3. Set up `windows/` skeleton structure

Immediate authoring-facing gaps to keep in scope:

- dedicated UI for `scene.layers[].ref`
- dedicated UI for `layer.objects`
- clearer diagnostics for scene-centric reference failures
- better distinction between authored scene structure and compiled preview result

### Short-Term (Next Sprint)

1. Finish Phase 1
2. Start Phase 2: window trait
3. Prototype timeline UI (no write-back yet)

### Medium-Term (Next Month)

1. Finish Phase 2
2. Complete Phase 3: timeline with YAML write-back
3. Start Phase 4: validation pipeline

## 10) Open Questions

- **Timeline persistence**: Should timeline state (zoom, playhead) save to workspace config?
- **Undo/redo**: Should editor support undo for YAML edits, or rely on git?
- **Multi-file timeline**: Can timeline span multiple scenes for cutscene sequencing?
- **Effect library**: Should editor support custom effect presets (saved YAML snippets)?

---

**Status**: DRAFT — awaiting team review  
**Author**: AI Agent (user-directed)  
**Date**: 2026-03-18
