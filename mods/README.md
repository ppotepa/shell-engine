# mods

Built-in content mods used for gameplay, testing, and experimentation.

## Included mods

- `shell-engine/` — main playable mod
- `shell-engine-tests/` — automation-friendly benchmark/regression variant
- `playground/` — development sandbox for experiments
- `demo-mod/` and `test-scenes/` — smaller sample/test content

## Usage

```bash
cargo run -p app -- --mod shell-engine
cargo run -p app -- --mod shell-engine-tests --bench 5
```

You can also point the app at a mod directory directly with `--mod-source`.

## Related docs

- `mods/shell-engine/README.AGENTS.MD`
- `mods/shell-engine-tests/README.AGENTS.MD`
- root `MODS.md`
