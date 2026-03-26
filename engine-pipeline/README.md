# engine-pipeline

Pipeline flags and reusable rendering strategy interfaces.

## Purpose

`engine-pipeline` is the shared configuration and strategy layer for the render
pipeline. It holds startup-time optimization flags and exports the trait-based
strategy building blocks used to assemble the active pipeline behavior.

## Main exports

- `PipelineFlags`
- `PipelineStrategies`
- strategy traits and implementations from `strategies`

## What `PipelineFlags` controls

- async render-thread usage,
- scene-change redraw guards,
- renderer-mode locking,
- compositor optimization gates,
- diff/present/skip optimizations,
- async display offload.

## Working with this crate

- keep flags as startup configuration, not ad-hoc runtime switches,
- prefer expressing pipeline variation through strategies instead of branching everywhere,
- when optimization flags change, update root optimization docs and benchmark workflows too.
