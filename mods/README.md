# mods

Built-in content mods used for gameplay, testing, and experimentation.

## Included mods

- `shell-quest/` — main playable mod
- `shell-quest-tests/` — automation-friendly benchmark/regression variant
- `asteroids/` — SDL-focused gameplay showcase with runtime entities and synth audio
- `playground/` — development sandbox for experiments
- `demo-mod/` and `test-scenes/` — smaller sample/test content

## Usage

```bash
cargo run -p app -- --mod shell-quest
cargo run -p app -- --mod shell-quest-tests --bench 5
cargo run -p app -- --mod-source=mods/asteroids --sdl2 --audio
cargo run -p app -- --mod-source=mods/asteroids --check-scenes
```

You can also point the app at a mod directory directly with `--mod-source`.

## Related docs

- `mods/shell-quest/README.AGENTS.MD`
- `mods/shell-quest-tests/README.AGENTS.MD`
- root `MODS.md`
