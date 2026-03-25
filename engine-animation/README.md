# engine-animation

Stage/step animator system for scene timeline progression.

## Purpose

Drives timed stage and step advancement within scenes. The animator
tracks elapsed time, manages animation state transitions, and signals
when a stage's steps have completed so the runtime can advance.

## Key Types

- `Animator` — core driver that ticks animation state each frame
- `AnimationState` — current stage index, step index, and elapsed time
- `StageAdvancer` — logic for progressing through stage steps and handling completion

## Dependencies

- `engine-core` — scene model (stages, steps, timing definitions)

## Usage

The runtime creates an `Animator` per scene and calls `tick()` each frame.
The animator updates `AnimationState` and returns commands when steps or
stages complete.
