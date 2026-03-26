# engine-error

Shared error type for startup and engine-adjacent workflows.

## Purpose

`engine-error` defines `EngineError`, the cross-crate error type used for mod
loading, manifest validation, terminal requirement checks, startup validation,
and related runtime setup failures.

## Main export

- `EngineError`

## Current scope

The enum currently covers:

- missing or unsupported mod sources,
- manifest read/parse failures,
- zip archive errors,
- missing required manifest fields,
- missing scene entrypoints,
- terminal requirement failures,
- startup check failures,
- render I/O failures.

## Working with this crate

- keep user-facing startup errors explicit and actionable,
- prefer adding structured variants over collapsing failures into generic strings,
- when new startup validation paths are introduced, extend this crate instead of inventing local error enums.
