# CognitOS — State Machine & Simulated OS Architecture

> How the fake MINIX 1.1 operating system works, communicates, and maintains state.

---

## Overview

Shell Quest simulates a MINIX 1.1 machine from September 1991. The simulation runs as
a **C# sidecar process** (`cognitos-os`) spawned by the Rust game engine. The two
processes communicate via **newline-delimited JSON over stdin/stdout**.

```
┌──────────────────────────────┐          stdin (JSON)          ┌──────────────────────────────┐
│                              │  ─────────────────────────────▶│                              │
│    Rust Engine               │                                │    C# Sidecar (cognitos-os)  │
│    (renderer, input,         │  ◀─────────────────────────────│    (simulated OS, commands,  │
│     scene graph, audio)      │         stdout (JSON)          │     VFS, boot, login, shell) │
│                              │                                │                              │
└──────────────────────────────┘                                └──────────────────────────────┘
```

The sidecar is a **state machine** that transitions through boot → login → shell,
processing user commands against a virtual file system and simulated kernel subsystems.

---

## 1. IPC Protocol

### Wire Format

One JSON object per line, newline-terminated (`\n`). Both directions.

### Engine → Sidecar (stdin)

| Type         | Fields                                       | Purpose                          |
|--------------|----------------------------------------------|----------------------------------|
| `hello`      | `cols, rows, difficulty?, boot_scene?`       | Initialize session               |
| `tick`       | `dt_ms`                                      | Advance simulated time           |
| `resize`     | `cols, rows`                                 | Terminal dimensions changed       |
| `set-input`  | `text`                                       | Live input preview (for masking) |
| `submit`     | `line`                                       | User pressed Enter               |
| `key`        | `code, ctrl, alt, shift`                     | Raw key event (reserved)         |

### Sidecar → Engine (stdout)

| Type                | Fields                              | Purpose                         |
|---------------------|-------------------------------------|---------------------------------|
| `out`               | `lines[]`                           | Append text to transcript       |
| `screen-full`       | `lines[], cursor_x, cursor_y`      | Full frame replace (primary)    |
| `set-prompt-prefix` | `text`                              | Update prompt string            |
| `set-prompt-masked` | `masked`                            | Toggle password input masking   |

`screen-full` is the main output mode — the sidecar rebuilds the entire visible frame
(transcript + prompt + input) on every change and sends it as a complete buffer. The
engine renders it onto text sprites in the scene graph.

### Rust-Side Wiring

```
engine-io/src/lib.rs          — IoRequest / IoEvent enums, SidecarProcess (spawn, send, drain)
engine/src/systems/engine_io.rs — engine_io_system: sends requests, applies events to scene
engine/src/scene_runtime.rs     — terminal_push_output(), sidecar_mark_screen_full()
```

### C#-Side Wiring

```
Program.cs              — Main loop: ReadLine → JsonDocument.Parse → route to AppHost
Core/Protocol.cs        — Protocol.Send(object) → JsonSerializer → Console.Out.WriteLine
Core/ScreenBuffer.cs    — Builds visible frame, sends screen-full on every mutation
```

---

## 2. Session State Machine

### States (SessionMode)

```
                    ┌──────────┐
                    │ Booting  │  ← EmitBoot(): boot_scene=true
                    └────┬─────┘
                         │ boot queue drained + 500ms delay
                         ▼
                    ┌──────────┐
        ┌──────────│LoginUser  │◀─── StartAtLogin(): boot_scene=false
        │          └────┬─────┘
        │               │ valid username entered
        │               ▼
        │          ┌───────────────┐
        │          │LoginPassword  │
        │          └───┬───────┬───┘
        │              │       │
        │     bad creds│       │good creds
        │              │       │
        │              ▼       ▼
        │         LoginUser  ┌───────┐
        │                    │ Shell │  ← EnterShell(): create session + app stack
        │                    └───────┘
        │                        │
        │               exit / logout
        │                        │
        └────────────────────────┘
```

### MachineState (persisted across sessions)

```csharp
SessionMode Mode            // Current state
string?     UserName        // "linus" after first login
string?     Password        // ≤5 chars, set on first login
DateTime?   LastLogin       // Shown at login banner
ulong       UptimeMs        // Accumulated tick time
MachineSpec Spec            // Hardware profile (RAM, CPU, disk)
QuestState  Quest           // Game progression flags
List<ProcessEntry>  Processes   // Simulated process table
List<ServiceEntry>  Services    // Simulated daemons
List<MailMessage>   MailMessages // Mail spool
```

Persisted to `state.obj` (ZIP archive) via `ZipStateStore.Persist()`.

---

## 3. Boot Sequence

`MinixBootSequence.BuildBootSteps(os)` generates authentic MINIX 1.1 boot messages:

```
=MINIX boot                         ← boot monitor
MINIX 1.1  Copyright 1987, Prentice-Hall, Inc.
Memory size = 4096K  MINIX = 109K  Available = 3987K
clock task                           ← kernel tasks
memory task
winchester task                      ← HDD seek (slowest)
tty task
ethernet task                        ← only if NIC enabled
root file system on /dev/hd1  OK     ← fs mount
/usr file system on /dev/hd2  OK
Init: Starting system.               ← init + rc
/etc/rc
update
cron                                 ← only if RAM ≥ 2048K
/etc/getty tty0 &                    ← terminal ready
```

Timing is scaled by CPU speed: `factor = 33.0 / spec.CpuMhz`. Slower CPUs = longer
boot. Each step is a `BootStep { Text, DelayMs }` consumed by `AppHost.DriveBoot()`
which drains the queue on each `tick` message.

After the last step, a 500ms post-delay transitions to `LoginUser`.

---

## 4. Application Stack

The shell session uses a stack-based application model:

```
┌─────────────────────────────────────────┐
│  ApplicationStack                       │
│  ┌────────────────────────────────────┐ │
│  │ FtpApplication  (if ftp launched)  │ │  ← top (receives input first)
│  ├────────────────────────────────────┤ │
│  │ ShellApplication (always present)  │ │  ← bottom (command dispatch)
│  └────────────────────────────────────┘ │
└─────────────────────────────────────────┘
```

- `HandleInput(text, session)` goes to the **top** application.
- `CurrentPrompt(session)` returns the **top** app's prompt.
- When an app pops (e.g., `ftp> bye`), the next one down resumes.
- `ShellApplication` is always at the bottom and never pops.

### IApplication interface

```csharp
string PromptPrefix(UserSession session);
void OnEnter(UserSession session);
void OnExit(UserSession session);
ApplicationResult HandleInput(string input, UserSession session);
```

---

## 5. Command Dispatch

### Flow: keystroke → output

```
User types "ls -la" + Enter
        │
        ▼
Rust engine sends:  {"type":"submit","line":"ls -la"}
        │
        ▼
Program.Main → AppHost.HandleSubmit("ls -la")
        │
        │  (Mode == Shell)
        ▼
ApplicationStack.HandleInput("ls -la", session)
        │
        ▼
ShellApplication.HandleInput("ls -la", session)
        │
        ├─ 1. Reject --help (strict 1991: "illegal option -- -")
        │
        ├─ 2. Split into argv = ["ls", "-la"]
        │
        ├─ 3. Look up "ls" in _commandIndex
        │     (Dictionary<string, IKernelCommand>, 38 commands + aliases)
        │
        ├─ 4. Create scoped UoW:  _createUow(session)
        │     → new LegacyUnitOfWork(_os, session)
        │     → Out = new StringWriter()  (buffered)
        │     → Disk = LegacyDisk wrapping VFS
        │     → Process = LegacyProcessTable wrapping OS
        │     → Clock = LegacyClock wrapping OS
        │
        ├─ 5. Execute:  command.Run(uow, ["ls", "-la"])
        │     → reads directory via uow.Disk.ReadDir()
        │     → writes output via uow.Out.WriteLine()
        │     → returns exit code (0 = success)
        │
        ├─ 6. FlushOutput(uow):
        │     → uow.Out.ToString() → split into lines
        │     → _screen.Append(lines)
        │
        └─ 7. ScreenBuffer.Append() → SendFrame()
              → Protocol.Send({ type: "screen-full", lines, cursor_x, cursor_y })
              → Rust engine renders text sprite
```

### IKernelCommand interface

```csharp
string Name { get; }                           // "ls"
IReadOnlyList<string> Aliases { get; }         // [] or ["la"]
int Run(IUnitOfWork uow, string[] argv);       // argv[0] = name
```

### Exit Code Conventions

| Code | Meaning                       |
|------|-------------------------------|
| 0    | Success                       |
| 1    | General error                 |
| 2    | Usage / syntax error          |
| 127  | Command not found             |
| 900  | Special: launch FTP app       |
| 901  | Special: clear screen         |

### Command not found → Easter Eggs

If `_commandIndex` has no match, `EasterEggRegistry.TryHandle()` is tried. Easter eggs
handle things like `linux`, `minix`, `hello`, `make love`, etc. If no egg matches
either, the shell prints `{cmd}: command not found` with exit code 127.

---

## 6. Kernel Subsystems

Commands access OS resources through `IUnitOfWork`, which composes all subsystems:

```
┌─────────────────────────────────────────────────────────────┐
│  IUnitOfWork (scoped per command execution)                 │
│                                                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌───────────┐  │
│  │ Out/Err  │  │  IDisk   │  │ IProcess │  │  IClock   │  │
│  │(TextWriter)│  │(VFS+time)│  │ Table    │  │(sim time) │  │
│  └──────────┘  └──────────┘  └──────────┘  └───────────┘  │
│                                                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌───────────┐  │
│  │ INetwork │  │IMailSpool│  │ IJournal │  │ Resources │  │
│  │(DNS/conn)│  │(mail ops)│  │(syslog)  │  │(snapshot) │  │
│  └──────────┘  └──────────┘  └──────────┘  └───────────┘  │
│                                                             │
│  ┌──────────┐  ┌──────────┐  ┌───────────────────────────┐ │
│  │ Session  │  │  Quest   │  │  Spec (MachineSpec)       │ │
│  │(user,cwd)│  │(progress)│  │  (RAM, CPU, disk, NIC)    │ │
│  └──────────┘  └──────────┘  └───────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Subsystem Details

| Subsystem       | Interface       | Key Methods                                       |
|-----------------|-----------------|----------------------------------------------------|
| **Disk**        | `IDisk`         | `ReadFile`, `WriteFile`, `ReadDir`, `Stat`, `Exists`, `Mkdir`, `Unlink`, `RawRead` |
| **Process**     | `IProcessTable` | `List()`, `Get(pid)`, `Fork()`, `Exec()`, `Exit()`, `Kill()` |
| **Clock**       | `IClock`        | `Now()`, `UptimeMs()`, `Epoch`, `Advance()`       |
| **Network**     | `INetwork`      | `Resolve()`, `Connect()`, `Send()`, `Recv()`, `Ping()`, `Close()` |
| **Mail**        | `IMailSpool`    | `List()`, `Read()`, `Deliver()`, `MarkRead()`     |
| **Journal**     | `IJournal`      | `Append()`, `ReadLog()`                           |
| **Resources**   | (snapshot)      | `TotalRamKb`, `FreeRamKb`, `DiskFreeKb`, `OpenFds`, `CpuLoadFactor`, ... |

### Resource Accounting (Kernel layer)

```
ResourceState
├── RamAllocator        — RAM pool (kernel 109K fixed + process allocations)
├── BufferCache         — LRU disk buffer cache (hits/misses/entries)
├── DiskController      — Disk queue + seek timing
├── CpuScheduler        — Runnable process count, load factor
├── FdTable             — File descriptor pool (open/max)
└── NetworkController   — Connection tracking
```

Hardware timings come from `HardwareProfile.FromSpec(MachineSpec)`:
- RAM access latency, disk seek time, CPU cycle time
- All derived from difficulty level (CPU MHz, RAM KB, disk KB, NIC speed)

### Current State: LegacyUnitOfWork Bridge

The full `Kernel` class exists and composes all subsystems, but is **not yet wired**
into the main dispatch path. Instead, `LegacyUnitOfWork` bridges the old
`IOperatingSystem` to satisfy the `IUnitOfWork` interface:

```
LegacyUnitOfWork
├── Out/Err          → StringWriter (buffered, flushed by ShellApplication)
├── Disk             → LegacyDisk wrapping IVirtualFileSystem
├── Process          → LegacyProcessTable wrapping IOperatingSystem.ProcessSnapshot()
├── Clock            → LegacyClock wrapping IOperatingSystem.SimulatedNow()
├── Net/Mail/Journal → throw NotSupportedException (not yet wired)
├── Resources        → computed snapshot from OS state
├── Session          → passed through
├── Quest            → from MachineState.Quest
└── Spec             → from IOperatingSystem.Spec
```

**Next step:** replace `LegacyUnitOfWork` with real `UnitOfWork` backed by full `Kernel`.

---

## 7. Virtual File System

### ZipVirtualFileSystem

The VFS is an **in-memory file tree** with two layers:

```
┌───────────────────────────────────┐
│  Mutable write layer              │  ← cp, mail delivery, ed writes
│  (Dictionary<string, string>)     │
├───────────────────────────────────┤
│  Immutable epoch layer            │  ← SeedEpochFiles(): hard-coded 1991 content
│  (rebuilt on every load)          │
├───────────────────────────────────┤
│  state.obj (ZIP archive)          │  ← persisted user modifications
│  users/linus/home/...             │
└───────────────────────────────────┘
```

**Load order:** `ReloadFromStateArchive()` → read ZIP entries → `SeedEpochFiles()`

**Epoch files** (always present, never persisted):
- `/usr/linus/` — Linus's home (VFS root maps here)
  - `linux-0.01/RELNOTES-0.01`, `README`, `Makefile`
  - `linux-0.01.tar.Z` — compressed archive marker
  - `.profile`, `.plan`, `.sh_history`
- `/usr/ast/` — Tanenbaum's home with `.plan`, notes
- `/etc/passwd`, `/etc/hosts`, `/etc/services`, `/etc/motd`
- `/var/log/messages`, `/var/log/boot.log`
- `mail/welcome.txt`, `mail/ast.txt`

**Path resolution:**
- Absolute `/usr/linus/file` → VFS key `file`
- Absolute `/etc/passwd` → VFS key `etc/passwd`
- Relative `linux-0.01/` → resolved by `UserSession.ResolvePath()`

**Persistence:** `ZipStateStore.Persist(state)` writes `MachineState` JSON +
modified files back to `state.obj` ZIP. On reload, epoch files are re-seeded
so they're never stale.

---

## 8. Screen Buffer & Output

### ScreenBuffer

The sidecar maintains a **complete terminal buffer** and sends full-frame updates:

```
┌─────────────────────────────────────────────┐
│  _visible: List<string>                     │  ← transcript lines
│                                             │
│  "Minix 1.3  Copyright 1987, Prentice-Hall" │
│  "Console ready"                            │
│  ""                                         │
│  "linus@kruuna:~ [0]$ ls"                  │  ← committed input
│  "RELNOTES-0.01  linux-0.01/  mail/"       │  ← command output
│  ""                                         │
│  "linus@kruuna:~ [0]$ _"                   │  ← current prompt + input
└─────────────────────────────────────────────┘
```

**On every mutation** (Append, CommitInputLine, ClearViewport, SetInputLine):

1. `BuildVisibleFrameLines()` — concatenate transcript + prompt + input
2. `WrapLine()` — hard-wrap to viewport width (color tags are zero-width)
3. Trim to `_viewportRows` (keep bottom)
4. `SendFrame()` → `Protocol.Send({ type: "screen-full", lines, cursor_x, cursor_y })`

The Rust engine receives `screen-full` and writes it into the scene's text sprites.

---

## 9. Difficulty & Hardware

`MachineSpec` defines the simulated hardware profile per difficulty:

| Difficulty        | CPU    | RAM     | Disk   | NIC       |
|-------------------|--------|---------|--------|-----------|
| I Can Exit Vim    | 33 MHz | 4096 KB | 40 MB  | 10 Kbps   |
| Langley Terminal  | 16 MHz | 2048 KB | 20 MB  | 5 Kbps    |
| Kernel Panic      | 8 MHz  | 1024 KB | 10 MB  | 0 (none)  |

These affect:
- **Boot speed** — slower CPU = longer boot delays
- **Process table** — less RAM = tighter constraints
- **Disk operations** — smaller disk = less free space for `cp`
- **Network** — no NIC = no ethernet task in boot, no ftp connectivity

---

## 10. File Map

```
mods/shell-quest/os/cognitos-os/
│
├── Program.cs                     Main loop (stdin JSON → route → AppHost)
│
├── Core/
│   ├── AppHost.cs                 State machine driver (boot/login/shell transitions)
│   ├── ApplicationStack.cs        Stack of running apps (shell, ftp, ...)
│   ├── IApplication.cs            App interface (HandleInput, PromptPrefix)
│   ├── ScreenBuffer.cs            Full-frame terminal buffer → screen-full JSON
│   ├── Protocol.cs                JSON stdin/stdout messaging
│   ├── OperatingSystem.cs         MinixOperatingSystem (legacy, being replaced)
│   ├── UserSession.cs             User context (name, hostname, cwd, exit code)
│   ├── Interfaces.cs              IKernelCommand, IOperatingSystem, IBootSequence
│   ├── Difficulty.cs              MachineSpec per difficulty level
│   └── Style.cs                   ANSI color helpers
│
├── Kernel/
│   ├── IKernel.cs                 Kernel interface (subsystem composition)
│   ├── Kernel.cs                  Full kernel (not yet wired to dispatch)
│   ├── IUnitOfWork.cs             Per-command scope (Out, Disk, Process, ...)
│   ├── UnitOfWork.cs              Real UoW implementation (for full Kernel)
│   ├── LegacyUnitOfWork.cs        Bridge: IOperatingSystem → IUnitOfWork
│   ├── Clock/                     IClock, SimulatedClock (epoch: 1991-09-17)
│   ├── Disk/                      IDisk, SimulatedDisk (VFS + timing)
│   ├── Process/                   IProcessTable, SimulatedProcessTable
│   ├── Network/                   INetwork, SimulatedNetwork
│   ├── Mail/                      IMailSpool, SimulatedMailSpool
│   ├── Journal/                   IJournal, SimulatedJournal
│   ├── Resources/                 ResourceState, RamAllocator, BufferCache, ...
│   ├── Hardware/                  HardwareProfile (timing from MachineSpec)
│   └── Services/                  ServiceManager
│
├── Commands/                      38 IKernelCommand implementations
│   ├── LsCommand.cs, CatCommand.cs, CdCommand.cs, PwdCommand.cs, ...
│   ├── FtpCommand.cs (exit 900), ClearCommand.cs (exit 901)
│   ├── ManCommand.cs (built-in man pages), MailCommand.cs
│   └── PingCommand.cs, NslookupCommand.cs (use NetworkRegistry)
│
├── Applications/
│   ├── ShellApplication.cs        Command dispatch (commandIndex + UoW factory)
│   └── FtpApplication.cs          Interactive FTP client (IApplication)
│
├── Boot/
│   └── BootSequence.cs            MinixBootSequence (authentic MINIX 1.1 messages)
│
├── State/
│   ├── Models.cs                  SessionMode, MachineState, ProcessEntry, ...
│   ├── StateStore.cs              ZipStateStore (persist to state.obj ZIP)
│   └── VirtualFileSystem.cs       ZipVirtualFileSystem (epoch + mutable layers)
│
├── Network/
│   ├── NetworkRegistry.cs         DNS/FTP server definitions
│   └── IExternalServer.cs         Server interface
│
└── EasterEggs/
    ├── IEasterEgg.cs              EasterEggRegistry + IEasterEgg interface
    ├── MinixEgg.cs                "minix" → Tanenbaum quote
    ├── LinuxEgg.cs                "linux" → Linus quote
    └── OneLiners.cs               "hello", "make love", etc.
```

---

## 11. What's Next

| Item | Status | Description |
|------|--------|-------------|
| Wire full Kernel into dispatch | Planned | Replace `LegacyUnitOfWork` with real `UnitOfWork` backed by `Kernel` |
| Resource-aware commands | Planned | Commands consume RAM (fork), disk I/O (seek time), FDs |
| TCP transport | Planned | Replace stdin/stdout with local TCP for decoupled debugging |
| Migrate FtpApplication | Planned | FtpApplication still uses old `IOperatingSystem` + `ScreenBuffer` directly |
| Remove legacy types | Planned | Drop `ICommand`, `CommandResult`, `CommandContext`, eventually `IOperatingSystem` |
