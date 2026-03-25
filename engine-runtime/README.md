# engine-runtime

Runtime settings and virtual-size parsing.

## Purpose

Defines the runtime configuration loaded from mod YAML, including
the virtual buffer size policy that determines how the engine maps
its logical resolution to the actual terminal dimensions.

## Key Types

- `RuntimeSettings` — top-level runtime configuration (target FPS, features, virtual size)
- `VirtualSizePolicy` — enum: `MaxAvailable`, `Fixed(w, h)`, `Constrained { max_width, max_height }`

## Dependencies

- `engine-core` — shared types and virtual size model
- `serde` / `serde_yaml` — deserialization from mod configuration YAML

## Usage

The launcher loads `RuntimeSettings` from the mod manifest. The
`VirtualSizePolicy` is applied on startup and re-evaluated on
terminal resize events.
