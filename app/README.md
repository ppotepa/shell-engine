# app

CLI launcher for running Shell Quest.

## Purpose

`app/` is the thin executable entrypoint that parses command-line flags,
initializes logging, builds `EngineConfig`, and starts `ShellEngine`.

## What it owns

- CLI parsing via `clap`
- dev/logging mode resolution
- launcher-level flag aliases and defaults
- translation from CLI inputs into `engine::EngineConfig`

## Common usage

```bash
cargo run -p app
cargo run -p app -- --mod shell-quest-tests --bench 5
cargo run -p app -- --mod-source=mods/playground --dev
```

## Related docs

- `app/README.AGENTS.MD` for startup-flow details
- root `README.md` for repository navigation
