# engine-audio-sequencer

Semantic audio sequencing and synth note-sheet generation.

## Purpose

`engine-audio-sequencer` is the authoring/runtime bridge for generated and
sequenced audio. It loads mod-defined semantic SFX mappings, song timelines,
and synth note sheets, then resolves them into frame-ready events.

This crate does not output audio directly. It resolves semantic events into cue
ids and timing hits consumed by `engine-audio`.

## Main responsibilities

- parse and validate `audio/sfx.yaml` semantic event banks,
- parse and validate `audio/songs/*.yml` sequenced tracks/patterns,
- parse and validate `audio/synth/*.yml` note sheets,
- synthesize note sheets into in-memory cue buffers,
- resolve semantic events (`audio.event`) into concrete cues with gain/cooldown,
- tick active songs and emit frame-local sequenced event hits.

## Main exports

- `SfxBank`, `SfxEventRuntime`
- `SongFile`, `SongRuntime`, `SequencedEvent`
- `NoteSheetFile`, `SynthSound`, `synthesize_note_sheet`
- `load_sfx_bank`, `load_song_library`, `load_note_sheets`
- `SequencerError`

## Working with this crate

- keep parsing/validation deterministic and startup-friendly,
- preserve deterministic variant selection for repeatable gameplay SFX,
- keep cue-id derivation stable (audio backend indexes cues by stem),
- when changing schema-like fields, update `AUTHORING.md` and startup checks.
