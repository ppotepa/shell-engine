# engine-core

Scene model, buffer management, effects, metadata, and strategy traits.

## Purpose

The foundation crate for the entire engine. Defines the scene data model
(scenes, layers, sprites), the virtual buffer and cell system used for
rendering, all built-in effect definitions and metadata, and the strategy
traits that allow the render pipeline to be composed from pluggable
implementations.

## Key Types

- `Scene` / `Layer` / `Sprite` — hierarchical scene data model
- `Buffer` / `Cell` — virtual framebuffer and per-cell terminal state
- `Effect` — trait for visual effects (fade, typewriter, glow, etc.)
- `DiffStrategy` — trait for detecting dirty regions between frames
- `LayerCompositor` — trait for compositing layers into a single buffer
- `HalfblockPacker` — trait for packing cells into halfblock characters
- `VirtualPresenter` — trait for presenting the virtual buffer to output
- `TerminalFlusher` — trait for flushing composed output to the terminal

## Dependencies

- `crossterm` — terminal style and color types
- `serde` / `serde_yaml` — serialization of scene model
- `chrono` — timestamp support

## Usage

Nearly every other engine crate depends on `engine-core`. See
`engine-core/README.AGENTS.MD` for detailed subsystem documentation.
