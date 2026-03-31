# mods

Built-in content mods used for gameplay, testing, and experimentation.

## Included mods

- `shell-quest/` — main playable mod
- `shell-quest-tests/` — automation-friendly benchmark/regression variant
- `playground/` — development sandbox for experiments
- `demo-mod/` and `test-scenes/` — smaller sample/test content

## Usage

```bash
cargo run -p app -- --mod shell-quest
cargo run -p app -- --mod shell-quest-tests --bench 5
```

You can also point the app at a mod directory directly with `--mod-source`.

## Related docs

- `mods/shell-quest/README.AGENTS.MD`
- `mods/shell-quest-tests/README.AGENTS.MD`
- root `MODS.md`
