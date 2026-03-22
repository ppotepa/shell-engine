# engine-io — sidecar ipc bridge

transport-agnostic bridge between the rust game engine and external sidecar processes.
this crate has zero game logic — pure message transport and protocol handling.

## protocol

wire format: one json object per line, newline-terminated.

### engine → sidecar (stdin)

```rust
pub enum IoRequest {
    Hello { cols, rows, boot_scene, difficulty? },
    SetInput { text },
    Submit { line },
    Key { code, ctrl, alt, shift },
    Resize { cols, rows },
    Tick { dt_ms },
}
```

example:
```json
{"type":"hello","cols":120,"rows":30,"boot_scene":true,"difficulty":"I CAN EXIT VIM"}
{"type":"submit","line":"ls"}
{"type":"tick","dt_ms":16}
```

### sidecar → engine (stdout)

```rust
pub enum IoEvent {
    Out { lines },
    Clear,
    SetPromptPrefix { text },
    SetPromptMasked { masked },
    ScreenDiff { clear, lines },
    ScreenFull { lines, cursor_x, cursor_y },
    Custom { payload },
}
```

example:
```json
{"type":"out","lines":["total 3","linux-0.01/  mail/  notes/"]}
{"type":"set-prompt-prefix","text":"linus@kruuna:~ [0]$ "}
```

## sidecar process

`SidecarProcess` manages the child process lifecycle:

- `spawn(command, args, cwd)` — start process, spin up i/o threads
- `send(req)` — serialize IoRequest to json, write to stdin
- `try_drain_events(max)` — poll stdout for IoEvent messages
- `is_alive()` — check process status
- `kill()` — terminate

two background threads per sidecar: stdin writer + stdout reader.
thread-safe via `Mutex<Child>` and mpsc channels.

## usage

from the engine (rust):
```rust
let sidecar = SidecarProcess::spawn("cognitOS", &[], Some(mod_root))?;
sidecar.send(IoRequest::Hello {
    cols: 120, rows: 30,
    boot_scene: true,
    difficulty: Some("I CAN EXIT VIM".into()),
})?;
```

from the sidecar (c# — see cognitOS/Program.cs):
```csharp
var root = JsonDocument.Parse(line).RootElement;
var type = Protocol.GetType(root);
if (type == "hello") { /* init */ }
```

## recent changes

- added `difficulty: Option<String>` to `IoRequest::Hello`
- engine reads difficulty from GameState and passes to sidecar on spawn
- sidecar parses via `MachineSpec.ParseLabel()` → `MachineSpec.FromDifficulty()`
