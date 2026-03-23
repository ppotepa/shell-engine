# cognitOS Event-Driven Architecture & State Machines

This document describes the event-driven simulation architecture introduced to replace
synchronous `Thread.Sleep`-based blocking in cognitOS, and the hardware state machines
that model a Minix 1.1-era PC.

---

## 1. Overview

The original sidecar used `_hw.BlockFor(latencyMs)` — a direct `Thread.Sleep` wrapper — to
simulate hardware latency. This blocked the entire sidecar thread, causing all output to
arrive at once (the engine side queued lines until the thread unblocked).

The new architecture has three layers:

```
IPC Protocol (EmitLine)
    └─ Engine delay queue  (engine/src/systems/engine_io.rs)
           └─ C# UnitOfWork.ScheduleOutput  (Kernel/UnitOfWork.cs)
                  └─ KernelEventQueue        (Kernel/Events/KernelEventQueue.cs)
                         └─ Hardware state machines (Kernel/Hardware/)
```

---

## 2. IPC Protocol Extension

### `EmitLine` event (`engine-io/src/lib.rs`)

```rust
EmitLine { text: String, delay_ms: Option<u64> }
```

The sidecar sends `emit-line:<delay_ms>:<text>` instead of plain `output:<text>`.  
The engine intercepts this **before** `apply_event` and stores it in a per-`EngineIoRuntime`
delay queue.

### Engine delay queue (`engine/src/systems/engine_io.rs`)

```
EngineIoRuntime.delayed_lines: Vec<(due_at_ms, String)>
EngineIoRuntime.accumulated_ms: u64
```

Each frame: `accumulated_ms += dt_ms`. Lines whose `due_at <= accumulated_ms` are flushed
to terminal output in order. On scene change or sidecar exit the queue is cleared.

This decouples simulation time from wall-clock time — if the engine runs at 60fps the
lines still arrive at the right *game* timestamps, not real-world ones.

---

## 3. C# Output API

### `IUnitOfWork.ScheduleOutput(string line, ulong delayMs)`

The primary way commands emit delayed lines:

```csharp
uow.ScheduleOutput("64 bytes from 198.145.20.140: icmp_seq=0 ttl=51 time=183ms", 200);
uow.ScheduleOutput("64 bytes from 198.145.20.140: icmp_seq=1 ttl=51 time=179ms", 200);
```

Delays are **cumulative** within a UoW session: the first line is at +200ms, the second
at +400ms, and so on. This mirrors how `ping` output looks on a real machine.

`UnitOfWork` resolves the output sink:
1. `GameTextWriter.Sink` — fast path (used by terminal shell)
2. Reflection fallback on `_sink` field — for any other `TextWriter` wrapping

`PipedUnitOfWork` (redirect/pipe decorator in `MinixExecutionPipeline`) delegates to
the inner UoW, preserving the cumulative counter.

### `DelayedOutputWriter` (`Framework/Transport/DelayedOutputWriter.cs`)

A `TextWriter` that wraps an `IOutputSink` and calls `SetNextLineDelay(ulong)` before
`WriteLine`. Used internally by `EasterEggOutput.Delayed(uow)`.

### `EasterEggOutput.SimulatePing(IUnitOfWork uow, params string[] lines)`

Helper used by all easter-egg ping hosts. Applies delays based on line content:

| Line prefix / content       | Delay       |
|-----------------------------|-------------|
| `PING ...` header           | immediate   |
| `64 bytes from ...`         | 200ms       |
| `Request timeout`           | 1200ms      |
| First stats line (`---`)    | 150ms gap   |
| Remaining stats / `net:`    | immediate   |

All ~50 easter-egg hosts use this helper — no `uow.Out.WriteLine` remains.

---

## 4. KernelEventQueue (`Kernel/Events/KernelEventQueue.cs`)

A priority-queue based scheduler keyed on `dueAtMs` (uptime milliseconds):

```csharp
public enum KernelEventKind { Timer, Output, Completion, Disk, Network, Modem }

public record ScheduledKernelEvent(
    ulong DueAtMs, ulong Sequence,
    KernelEventKind Kind,
    Action Action,
    string? Tag);
```

### Key methods

| Method | Description |
|---|---|
| `ScheduleAt(dueAtMs, kind, action, tag)` | Absolute time |
| `ScheduleAfter(nowMs, delayMs, kind, action, tag)` | Relative time |
| `DrainReady(nowMs)` | Returns all events with `DueAtMs <= nowMs`, sorted by time then insertion order |

`Kernel.Tick()` calls `Events.DrainReady(NowMs)` after advancing the clock, executing
each action in order.

`IKernel.NowMs` delegates to `Clock.UptimeMs()` — monotonic uptime since simulation
epoch, not `DateTime` subtraction.

---

## 5. Hardware State Machines (Phase 3 plan)

Three hardware components are modelled as state machines. Currently they still use
`BlockFor` (a `Thread.Sleep` wrapper in `HardwareProfile`). The state machines describe
the intended target architecture.

### 5.1 Disk

**Status: ✅ IMPLEMENTED**

```
Running ──[idle >30s]──► Stopped
Running ──[idle 2-30s]──► Coasting
Stopped ──[access]──► Running (adds 300ms spindle spin-up)
Coasting ──[access]──► Running (adds 80ms coast→full speed)
Running ──[access]──► Running (no extra delay)
```

The spindle state is tracked in `DiskController` with transitions recalculated each kernel tick
via `UpdateSpindleState(nowMs)`. Disk access incurs spin-up latency only on state change.

**Implementation details:**
- `HardwareProfile`: Added `DiskSpinUpMs` (300ms), `DiskCoastMs` (80ms), `DiskIdleStopMs` (30s), `DiskCoastThresholdMs` (2s)
- `DiskController`: Tracks spindle `_lastAccessMs` and `_state` (enum). `Acquire(nowMs)` returns spin-up cost.
- `MinixSyscallGate`: Calls `_res.DiskCtrl.Acquire(_clock.UptimeMs())` in `LatencyFor()`, passing current time.
- `Kernel.Tick()`: Calls `Resources.DiskCtrl.UpdateSpindleState(NowMs)` each tick to advance state.

**Latency model:**
- DiskRead: `Acquire(spin-up) + TransferTime + Contention + CPU overhead`
- All disk ops use the state machine; network ops unaffected

### 5.2 Network / Modem

```
Idle ──[dial]──► ATH0 (200ms)
ATH0 ──► ATDT dialing (800ms)
ATDT ──► Ringing (3 × 2000ms rings)
Ringing ──► Handshake (1200ms)
Handshake ──► Connected
Connected ──[hangup]──► Idle
```

`SimulatedModem.Dial()` currently uses sequential `BlockFor` calls for each phase.
Target: each phase transition is a `Modem` event on `KernelEventQueue`.

**Latency sources:**
- 2400 baud modem: ~240 bytes/s
- Each ping packet: ~200ms round-trip (dial-up era)
- Timeout: 1200ms (standard ICMP timeout)

### 5.3 Network packet queue

Per-connection `Queue<NetworkPacket>`. Each packet has a `dueAtMs` based on RTT.
`Network` events on `KernelEventQueue` fire when a packet is ready to be consumed
by the waiting command.

---

## 6. Data-flow diagram

```
[Command.Execute(uow)]
        │
        ├─ uow.ScheduleOutput("line", 200)
        │       │
        │       └─ Protocol.EmitLine(sink, "line", cumulativeMs)
        │               │
        │               └─ IPC: "emit-line:400:line\n"
        │                           │
        │                     [engine_io_system]
        │                     accumulated_ms += dt_ms
        │                     if due_at <= accumulated_ms:
        │                         push to terminal output
        │
        └─ kernel.Schedule(KernelEventKind.Disk, 40, callback)
                │
                └─ KernelEventQueue.ScheduleAfter(nowMs, 40, ...)
                        │
                  [Kernel.Tick()]
                  DrainReady(NowMs) → fires callback
                  callback → uow.ScheduleOutput(result, 0)
```

---

## 7. Files reference

| File | Role |
|---|---|
| `engine-io/src/lib.rs` | `IoEvent::EmitLine` protocol variant |
| `engine/src/systems/engine_io.rs` | Engine-side delay queue |
| `Core/Protocol.cs` | `EmitLine(sink, text, delayMs)` helper |
| `Framework/Transport/GameTextWriter.cs` | Exposes `Sink` property |
| `Framework/Transport/DelayedOutputWriter.cs` | TextWriter with cumulative delay |
| `Framework/Kernel/IUnitOfWork.cs` | `ScheduleOutput` interface method |
| `Kernel/UnitOfWork.cs` | `ScheduleOutput` implementation + sink resolution |
| `Kernel/Events/KernelEventQueue.cs` | Priority-queue event scheduler |
| `Kernel/IKernel.cs` | `NowMs`, `Schedule()` on internal interface |
| `Kernel/Kernel.cs` | `Events`, `NowMs`, `Schedule()`, `Tick()` update |
| `Network/Hosts/EasterEggHosts.cs` | All ~50 hosts using `SimulatePing` |
| `Kernel/Hardware/HardwareProfile.cs` | `BlockFor` — target for Phase 2 removal |
| `Minix/Kernel/MinixSyscallGate.cs` | `BlockFor` call sites — Phase 2 target |
| `Minix/Shell/MinixExecutionPipeline.cs` | `PipedUnitOfWork` delegates `ScheduleOutput` |
