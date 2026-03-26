# editor

Terminal authoring tool for Shell Quest content.

## Purpose

`editor/` is the TUI application used to browse, inspect, and edit mod content.
It combines terminal UI code with authoring/domain indexes built on top of
`engine-authoring` and `engine-core`.

## Main areas

- `app` — editor lifecycle and main loop
- `cli` — editor CLI arguments
- `domain` — scene/effect/asset indexes and diagnostics
- `io` — file scanning and YAML I/O
- `input` — key mappings and command dispatch
- `state` — editor state machine
- `ui` — terminal layout, widgets, and theme

## Common usage

```bash
cargo run -p editor
```

## Related docs

- `editor/README.AGENTS.MD` for subsystem details and workflow notes
- root `AUTHORING.md` for content conventions
