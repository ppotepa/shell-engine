# editor

SDL2-backed authoring tool stub for Shell Quest content.

## Purpose

`editor/` is the editor application for browsing, inspecting, and editing mod
content. The original terminal TUI (crossterm/ratatui) has been removed; the
editor now runs as an SDL2-backed stub while a full SDL2 UI is being built. It
combines domain indexes built on top of `engine-authoring` and `engine-core`.

## Main areas

- `app` — editor lifecycle and main loop
- `cli` — editor CLI arguments
- `domain` — scene/effect/asset indexes and diagnostics
- `io` — file scanning and YAML I/O
- `input` — key mappings and command dispatch
- `state` — editor state machine
- `ui` — draw, layout, and theme (SDL2-backed stub)

## Common usage

```bash
cargo run -p editor
```

## Related docs

- `editor/README.AGENTS.MD` for subsystem details and workflow notes
- root `AUTHORING.md` for content conventions
