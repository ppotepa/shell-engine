# engine-audio

Audio playback subsystem with pluggable backends.

## Purpose

Provides a trait-based audio interface so the engine can play sound
effects and music without coupling to a specific audio library. The
default backend uses rodio; a null backend exists for headless/test runs.

## Key Types

- `AudioProvider` — trait abstracting audio playback (play, stop, set_volume)
- `RodioAudioBackend` — production backend using the rodio crate
- `NullAudioBackend` — silent no-op backend for testing and CI

## Dependencies

- `engine-core` — shared types and audio command definitions
- `rodio` — cross-platform audio playback
- `thiserror` — error type derivation

## Usage

The runtime selects a backend at startup. Scene behaviors emit audio
commands (play/stop) which the runtime forwards to the active
`AudioProvider`.
