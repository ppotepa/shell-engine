# mods

Bundled content mods used for renderer development, gameplay experiments, and
authoring validation.

## Included mods

- `playground/` — general engine sandbox
- `planet-generator/` — procedural world and planet tuning
- `lighting-playground/` — lighting and scene look experiments
- `gui-playground/` — widget system playground
- `terrain-playground/` — terrain/worldgen experiments
- `asteroids/` — gameplay-heavy orbital prototype

## Usage

```bash
cargo run -p app -- --mod playground
cargo run -p app -- --mod-source=mods/planet-generator
cargo run -p app -- --mod-source=mods/asteroids --check-scenes
```

You can also point the app at any unpacked or zipped mod with `--mod-source`.

## Related docs

- root `MODS.md`
- per-mod `README.md` files inside `mods/`
