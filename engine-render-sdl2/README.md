# engine-render-sdl2

SDL2 output backend for the Shell Quest runtime.

## Purpose

`engine-render-sdl2` provides the optional windowed backend selected via
`--output sdl2`. It implements the shared `engine-render::OutputBackend`
contract and exposes SDL2-backed input through the shared `InputBackend`
interface.

The current design keeps SDL objects on an SDL-owned worker thread and bridges
the engine to that thread through channels. This avoids forcing SDL window or
canvas types to satisfy thread-safety guarantees they do not provide.

## Key modules

- `renderer` — `Sdl2Backend` and backend construction helpers
- `input` — `Sdl2InputBackend`
- `runtime` — SDL worker-thread loop, render/input command bridge
- `color_convert` — engine color to SDL color conversion

## Current behavior

- SDL keeps a fixed offscreen render texture sized from the engine output buffer
- Output rows are expanded to double-height logical pixels so halfblocks stay sharp
- The window presents that texture using the shared `presentation_policy` (`stretch`, `fit`, or `strict`) without emitting engine buffer resize events
- Key, mouse, and quit events are translated into `engine-events`
- The crate is enabled through the `engine/sdl2` feature
