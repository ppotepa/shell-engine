# engine-render-policy

Render scheduling and frame pacing policies.

## Purpose

Controls when the engine renders a new frame. Policies decide whether
to render on every tick, at a fixed rate, or synchronized to a vsync
signal, allowing the runtime to trade off responsiveness against CPU
usage.

## Key Types

- `RenderPolicy` — trait with `should_render()` and `on_frame_complete()` methods
- `FixedRatePolicy` — renders at a fixed interval (e.g., 60 FPS cap)
- `VsyncPolicy` — synchronizes frame production to vertical sync

## Dependencies

- `engine-core` — timing and frame context types

## Usage

The runtime selects a policy at startup. Each tick, the main loop
calls `should_render()` to decide whether to run the render pipeline.
