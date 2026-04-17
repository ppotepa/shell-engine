# app

CLI entrypoint for running Shell Engine.

## Purpose

`app/` parses command-line flags, initializes logging, resolves startup mode,
builds `EngineConfig`, and starts `ShellEngine`.

## What it owns

- CLI parsing via `clap`
- logging/dev-mode resolution
- translation from CLI inputs into `engine::EngineConfig`
- startup scene checks for a selected mod
- handoff to the interactive launcher when no mod is specified

## Common usage

```bash
cargo run -p app
cargo run -p app -- --mod playground
cargo run -p app -- --mod-source=mods/planet-generator
cargo run -p app -- --mod-source=mods/asteroids --check-scenes
```

## Related docs

- `app/README.AGENTS.MD` for startup-flow details
- root `README.md` for repository navigation
