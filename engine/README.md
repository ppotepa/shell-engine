# engine

Top-level runtime orchestrator for Shell Engine.

## Purpose

`engine/` wires the extracted engine crates together into a running game loop.
It owns engine-specific orchestration, world integration, startup/runtime glue,
and the systems that still require direct access to engine-owned resources.

## What lives here

- game-loop orchestration
- scene loading and lifecycle sequencing
- world/provider wiring between subsystem crates
- gameplay/collision/event bridge wiring for component-backed entities
- visual sync system (`visual_sync.rs` — auto-copies Transform2D → scene position after behavior step)
- visual binding cleanup (`visual_binding.rs` — processes despawn queue at end of frame)
- audio sequencer orchestration (`audio/sfx.yaml`, `audio/songs`, `audio/synth`)
- engine-side hot reload, debug, and render-thread integration
- thin system wrappers that delegate core domain logic to extracted crates

## Working with this crate

- keep reusable domain logic in focused crates when possible,
- keep orchestration and world extraction here,
- re-run `cargo test -p engine --lib` after behavior changes.

## Related docs

- `engine/README.AGENTS.MD` for per-frame order and invariants
- nearby `engine-*` crate READMEs for domain-specific ownership
