# engine-frame

Frame ticket for render-thread generation tracking.

## Purpose

Provides a lightweight ticket type that pairs a render generation
counter with a simulation frame ID. Used to detect stale frames and
prevent the renderer from presenting outdated buffers after scene
transitions.

## Key Types

- `FrameTicket` — struct with `generation` and `sim_frame_id` fields
- `is_acceptable()` — checks whether a ticket is still valid for the current render generation

## Dependencies

None — this is a standalone data crate with no external dependencies.

## Usage

The runtime stamps each frame with a `FrameTicket`. The render thread
calls `is_acceptable()` to discard frames from a previous generation
after a scene transition.
