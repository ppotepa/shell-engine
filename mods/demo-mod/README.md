# demo-mod

Small sample mod for basic runtime and authoring checks.

## Purpose

`demo-mod/` is a lightweight built-in mod that provides a minimal content set
for quick experiments, smoke testing, and simple authoring examples.

## Contents

- `mod.yaml` — manifest and runtime defaults
- `scenes/` — small authored scene set
- `stages/` — stage definitions
- `schemas/` — generated mod-local schema fragments

## Typical usage

```bash
cargo run -p app -- --mod demo-mod
```
