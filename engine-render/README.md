# engine-render

RenderBackend trait abstraction for output targets.

## Purpose

Defines the `RenderBackend` trait that decouples the engine's render
pipeline from any specific output target. Concrete backends (e.g.,
terminal, capture) implement this trait to present composed frames.

## Key Types

- `RenderBackend` — trait with methods: `present()`, `clear()`, `shutdown()`
- `RenderError` — error type for backend failures

## Dependencies

- `engine-core` — buffer types passed to `present()`
- `thiserror` — error type derivation

## Usage

The runtime holds a `Box<dyn RenderBackend>` and calls `present()`
each frame. Swap backends to redirect output (terminal, file capture,
headless).
