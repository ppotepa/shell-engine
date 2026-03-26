# engine

Top-level runtime orchestrator for Shell Quest.

## Purpose

`engine/` wires the extracted engine crates together into a running game loop.
It owns engine-specific orchestration, world integration, startup/runtime glue,
and the systems that still require direct access to engine-owned resources.

## What lives here

- game-loop orchestration
- scene loading and lifecycle sequencing
- world/provider wiring between subsystem crates
- engine-side hot reload, debug, and render-thread integration
- thin system wrappers that delegate core domain logic to extracted crates

## Working with this crate

- keep reusable domain logic in focused crates when possible,
- keep orchestration and world extraction here,
- re-run `cargo test -p engine --lib` after behavior changes.

## Related docs

- `engine/README.AGENTS.MD` for per-frame order and invariants
- nearby `engine-*` crate READMEs for domain-specific ownership
