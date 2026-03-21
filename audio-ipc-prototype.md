# Audio IPC Prototype

This prototype introduces a process-separated audio path aligned with the current engine architecture:

- `app` / `engine` is the **audio client** (runtime + command queue).
- `tools/sound-server` is the **audio server** process.
- Transport is newline-delimited JSON (`JSONL`) over child `stdin`.

## Components

- `engine/src/audio.rs`
  - `AudioRuntime::from_env()` chooses backend at runtime.
  - `NullAudioBackend` remains default fallback.
  - `StdIoSoundBackend` spawns a child process and forwards `play`/`shutdown` commands.

- `tools/sound-server/src/main.rs`
  - Receives JSON requests (`play`, `stop`, `set-master`, `ping`, `shutdown`).
  - Prototype behavior is stateful logging/ack handling (no hardware audio yet).

- `app/src/main.rs`
  - `--sound-server` enables external backend.
  - `--sound-server-cmd` overrides spawn command.

## Runtime toggles

- `SHELL_QUEST_SOUND_SERVER=1`
- `SHELL_QUEST_SOUND_SERVER_CMD='cargo run -p sound-server --quiet -- --verbose'`

If spawn fails, engine logs a warning and continues with `NullAudioBackend`.

## Command protocol (client -> server)

```json
{"type":"play","cue":"menu-move","volume":0.8}
{"type":"shutdown"}
```

Server also accepts future-facing commands:

```json
{"type":"stop","cue":"menu-move"}
{"type":"set-master","volume":0.7}
{"type":"ping"}
```

## Local run examples

Start game with prototype sound server:

```bash
cargo run -p app -- --sound-server
```

Start game with explicit server command:

```bash
cargo run -p app -- --sound-server-cmd "cargo run -p sound-server --quiet -- --verbose"
```

Run server standalone (manual testing):

```bash
cargo run -p sound-server -- --ack --verbose
```
