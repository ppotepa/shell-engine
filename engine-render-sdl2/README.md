# engine-render-sdl2

> Status: Deprecated / Inactive.
> This crate is intentionally retained on disk for reference, but it is not an
> active workspace member and is not wired into the current `engine` runtime.

SDL2 software render backend for the Shell Engine runtime.

## Purpose

`engine-render-sdl2` is the software side of the render backend split. It owns
the SDL2 windowed presentation path and the input bridge used by the runtime
today. The eventual hardware backend is not implemented in this repository yet,
so this crate remains the concrete backend that turns engine output into pixels
on screen.

It implements the shared `engine-render::RendererBackend` software
presentation contract and exposes SDL2-backed input through the shared
`InputBackend` interface.

The current design keeps SDL objects on an SDL-owned worker thread and bridges
the engine to that thread through channels. This avoids forcing SDL window or
canvas types to satisfy thread-safety guarantees they do not provide.

## Role in the split

- Software backend for window creation, presentation, and input handling
- Reference implementation for the post-split render pipeline
- Compatibility layer for existing engine output until a hardware backend is
  added

## Key modules

- `renderer` — `Sdl2Backend` and backend construction helpers
- `input` — `Sdl2InputBackend`
- `runtime` — SDL worker-thread loop, render/input command bridge
- `color_convert` — engine color to SDL color conversion

## Current behavior

- SDL keeps a fixed offscreen render texture sized from the engine output buffer
- Glyphs are rendered to a pixel canvas and uploaded as texture patches
- The window presents that texture using the shared `presentation_policy`
  (`stretch`, `fit`, or `strict`) without emitting engine buffer resize events
- Key, mouse, and quit events are translated into `engine-events` typed variants:
  - `KeyDown { key: KeyEvent, repeat: bool }` / `KeyUp { key: KeyEvent }`
  - `MouseMoved { x: f32, y: f32 }` — output-space float coords
  - `MouseButtonDown/Up { button: MouseButton, x: f32, y: f32 }`
  - `MouseButton` is the typed enum from `engine-events` (`Left/Right/Middle`)
- Historical note: this crate used to be enabled through `engine/sdl2`

## Limitations

- This is still a software raster/present backend, not a hardware-accelerated
  backend
- There is no companion hardware backend in this repo yet
- Presentation is constrained by the SDL windowing model and the engine's
  output-buffer contract
- Texture uploads and offscreen rendering remain part of the current path
