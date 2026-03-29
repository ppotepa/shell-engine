# engine-audio

In-process audio runtime with pluggable backends.

## Purpose

`engine-audio` provides the audio subsystem used by the runtime to play sound
effects and music cues emitted by scenes and behaviors.

The default production path is embedded rodio playback. This is not a separate
sound server process.

This crate is playback-focused. Semantic event/song sequencing lives in
`engine-audio-sequencer`; engine systems resolve semantic events there and then
dispatch concrete cue ids to `engine-audio`.

## Key modules and types

- `audio` — backend types and runtime command handling
- `systems_audio` — frame-driven audio system integration
- `access` — provider traits for host integration
- `AudioBackend`
- `AudioRuntime`
- `AudioCommand`
- `RodioAudioBackend`
- `NullAudioBackend`
- `AudioProvider`

## Working with this crate

- keep backend selection and runtime command flushing separate,
- preserve the no-audio/null path for tests and headless runs,
- when changing asset loading or cue lookup, verify behavior against real mod asset directories.
- keep cue-id behavior stable because sequencer output depends on cue stems.
