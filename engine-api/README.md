# engine-api

Script-facing engine API facade.

## Purpose

`engine-api` is the public-facing surface used by Rhai scripts and other
engine-side consumers. It groups engine capabilities by domain instead of
exposing runtime internals directly.

## What it owns

- `BehaviorCommand`
- typed scene mutation request types
- scene/audio/effects/input/gameplay API registration helpers
- Rhai conversion helpers shared by script-facing modules

## Important note

Scene mutation flow is typed-first. `scene.mutate(...)` and supported
`scene.set(...)` paths are translated into typed mutation requests before
runtime application. A narrow `SetProperty` fallback still exists for paths
without typed coverage.

## Main modules

- `scene` — scene object access and typed mutation requests
- `commands` — command types and compatibility mappers
- `audio`, `effects`, `collision`, `input` — script-facing domain APIs
- `gameplay` — gameplay context/helpers used by behavior code
- `rhai` — conversion utilities
