# editor

Authoring tool stub for Shell Engine content.

## Purpose

`editor/` is the editor application for browsing, inspecting, and editing mod
content. The original terminal TUI (crossterm/ratatui) has been removed; the
editor currently runs as a minimal runtime stub while the next UI path is being
built. It combines domain indexes built on top of `engine-authoring` and
`engine-core`.

## Main areas

- `app` — editor lifecycle and main loop
- `cli` — editor CLI arguments
- `domain` — scene/effect/asset indexes and diagnostics
- `io` — file scanning and YAML I/O
- `input` — key mappings and command dispatch
- `state` — editor state machine
- `ui` — draw, layout, and theme (stub)

## Common usage

```bash
cargo run -p editor
```

## Related docs

- `editor/README.AGENTS.MD` for subsystem details and workflow notes
- root `AUTHORING.md` for content conventions
