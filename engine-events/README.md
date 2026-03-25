# engine-events

Input event type definitions for the engine runtime.

## Purpose

Defines the `EngineEvent` enum that the runtime uses to represent
all external inputs — keyboard presses, terminal resizes, scene
transitions, and quit signals.

## Key Types

- `EngineEvent` — enum with variants: `KeyPress`, `Resize`, `SceneTransition`, `Quit`

## Dependencies

- `crossterm` — key event and modifier types used in `KeyPress`

## Usage

The input subsystem converts raw crossterm events into `EngineEvent`
values. The main loop pattern-matches on these to dispatch behavior.
