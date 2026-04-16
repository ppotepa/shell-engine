# Editor Subsystem

## Architecture Overview

The editor is an SDL2-backed stub for authoring shell-engine scenes. The original
terminal TUI (crossterm/ratatui) has been replaced; a full SDL2 editor UI is not
yet implemented.

**7 main modules:**

| Module | Purpose | Key Types |
|--------|---------|-----------|
| `app.rs` | Main editor lifecycle, event loop | `AppState`, `EditorApp` |
| `cli.rs` | CLI argument parsing | `CliArgs` |
| `domain/` | Scene/effect/asset indexes | `SceneIndex`, `EffectRegistry`, `AssetCache` |
| `io/` | File scanning, YAML load/save | `FileScanner`, `YamlIO` |
| `input/` | Key mapping, command dispatch | `KeyMap`, `Command` |
| `state/` | App state machine | `EditorState`, `FocusTarget` |
| `ui/` | Draw, layout, focus, theme | `Renderer`, `Layout`, `Theme` |

## Editor ↔ Engine Bridge

**Hot-reload system:**
- Editor watches for file changes in mod directory
- On file change: rebuild scene/effect/asset registry
- Recompile affected scenes
- Notify engine to reload scene if active
- Engine validates changes before accepting

**Scene editing:**
1. Editor loads scene.yml
2. User edits (sprites, effects, layers, behaviors)
3. Save to disk
4. Compile via `engine-authoring`
5. Validate (schema, timeline, references)
6. Engine loads new version

**Asset preview:**
- Editor can preview GIFs, sprites inline
- Uses engine's rendering pipeline
- Shows composition result real-time

## Key Subsystems

### Domain Index (domain/)
- **SceneIndex**: All scenes + metadata
- **EffectRegistry**: Available effects (built-in + mod)
- **AssetCache**: All assets (sprites, gifs, 3d, sounds)
- Auto-updated via FileScanner

### File I/O (io/)
- **FileScanner**: Recursive directory scan, watches for changes
- **YamlIO**: Load/save scenes, effects, behaviors
- **ValidationReporter**: Lint messages, warnings, errors

### Input System (input/)
- **KeyMap**: Keybind configuration
- **Command**: Available editor commands (new scene, add sprite, etc.)
- Dispatch: key → command → state change

### UI Rendering (ui/)
- **Renderer**: Drawing layer (SDL2-backed stub)
- **Layout**: Viewport grid (left panel: tree, right panel: editor/preview)
- **Theme**: Color scheme (light/dark)
- **FocusTarget**: Currently focused element (tree item, text input, etc.)

## Hot-Reload Integration

```
FileScanner detects file.yml changed
  → YamlIO re-loads file
  → Compiler validates
  → EffectRegistry/SceneIndex updated
  → UI refreshes display
  → If scene active in engine: notify via IPC
```

## State Machine

- **Browsing**: Navigate scene tree
- **Editing**: Modify sprite/effect/layer properties
- **Previewing**: Show composition preview
- **ErrorState**: Display validation errors
