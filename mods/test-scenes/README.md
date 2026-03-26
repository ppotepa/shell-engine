# test-scenes

Focused test content mod for scene and effect validation.

## Purpose

`test-scenes/` contains small authored scenes used to exercise specific scene,
object, and effect behaviors without the full complexity of the main game mod.

## Contents

- `mod.yaml` — manifest and runtime defaults
- `assets/` — assets used by the test scenes
- `effects/` — effect definitions
- `objects/` — reusable objects
- `scenes/` — targeted scene fixtures
- `schemas/` — generated mod-local schema fragments
- `stages/` — stage definitions

## Typical usage

```bash
cargo run -p app -- --mod test-scenes
```
