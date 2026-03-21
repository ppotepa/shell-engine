# cognitos-os — simulated minix sidecar

standalone c# process that simulates a 1991-era minix terminal environment.
runs as ipc sidecar to the main game engine (rust) via json lines on stdin/stdout.

## how it works

1. engine spawns this process when entering a terminal scene
2. engine sends `hello` with screen size, boot flag, and difficulty level
3. sidecar runs boot sequence → login → shell → ftp (depending on game state)
4. all output goes back as `out`, `screen-diff`, `set-prompt-prefix` messages
5. engine renders the output into terminal sprites on screen

## structure

```
Core/
  AppHost.cs        — main state machine (boot → login → shell → ftp)
  Difficulty.cs     — 5 difficulty levels + MachineSpec hardware config
  Interfaces.cs     — ICommand, IOperatingSystem, CommandContext, ResolvePath()
  OperatingSystem.cs — MinixOS implementation
  Protocol.cs       — json ipc helpers
  Style.cs          — ansi color formatting

Commands/
  LsCommand.cs      — list files (cwd-aware)
  CatCommand.cs     — read files (cwd-aware)
  CdCommand.cs      — change directory
  PwdCommand.cs     — print working directory
  CpCommand.cs      — copy files (mutable filesystem)
  FtpCommand.cs     — enter ftp client mode
  HelpCommand.cs    — list available commands
  ClearCommand.cs   — clear screen
  PsCommand.cs      — process list
  TopCommand.cs     — system resources
  ServicesCommand.cs — running services

Network/
  FtpSession.cs     — full ftp client (open/binary/ascii/put/ls/bye)

State/
  Models.cs         — MachineState, QuestState, SessionMode
  VirtualFileSystem.cs — zip-backed filesystem + in-memory mutable overlay
  StateStore.cs     — persistence to state.obj zip

Boot/
  BootSequence.cs   — hardware detection, minix banner (reads MachineSpec)
```

## difficulty system

difficulty maps to simulated hardware via `MachineSpec.FromDifficulty()`:

| level | cpu | ram | nic | disk |
|-------|-----|-----|-----|------|
| mouse enjoyer | 486 DX2-66 MHz | 8192 KB | NE2000 10Mbps | 120 MB |
| script kiddie | 486 DX-33 MHz | 4096 KB | NE2000 2400 Kbps | 80 MB |
| i can exit vim | 386 DX-33 MHz | 4096 KB | NE2000 1200 Kbps | 40 MB |
| dvorak | 386 SX-16 MHz | 2048 KB | generic 600 Kbps | 20 MB |
| su | 386 SX-16 MHz | 1024 KB | generic 300 Kbps | 10 MB |

every subsystem reads `IOperatingSystem.Spec` — nothing is hardcoded.

## filesystem

`ZipVirtualFileSystem` loads from `state.obj` zip, entries under `users/linus/home/`.
`SeedEpochFiles()` adds the prologue working tree at boot:

```
~/linux-0.01/
  README
  RELNOTES-0.01
  linux-0.01.tar.Z    ← the archive to upload
  bash.Z
  update.Z
~/mail/welcome.txt
~/notes/starter.txt
```

mutable overlay: `TryCopy`, `TryWrite`, `TryMkdir` work in-memory (not persisted to zip).

## quest state

`QuestState` in Models.cs tracks prologue progress:

- `FtpTransferMode` — "ascii" (default) or "binary"
- `UploadAttempted` — player tried to upload
- `UploadSuccess` — transfer succeeded (binary mode)
- `BackupMade` — player made a backup with `cp`
- `FtpConnected` — active ftp connection

## building

```bash
cd mods/shell-quest/os/cognitos-os
dotnet build -c Release
```

the engine spawns this automatically — no manual run needed during gameplay.

## see also

- [engine-io/README.md](../../../../engine-io/README.md) — ipc protocol
- [docs/scripts.md](../docs/scripts.md) — prologue quest design
- [AGENTS.md](../../../../AGENTS.md) — change playbook for sidecar code
