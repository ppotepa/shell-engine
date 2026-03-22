# State Machine Implementation Status

## Worktree
- Branch: `state-machine`
- Worktree path: `/home/ppotepa/git/shell-quest-state-machine`
- Current head: transport phase completed locally

## Implemented phases
- Phase 0 — IoC container
  - `ServiceContainer`, lifetimes, OS module boundary, command attribute/scanner skeleton
- Phase 2 — syscall gate
  - `ISyscallGate`, syscall request/result types, MINIX gate, disk/net/process routed through gate
- Phase 3 — kernel wiring
  - framework `IKernel` / `IUnitOfWork`
  - `MinixModule`
  - `Program.cs` and `AppHost` boot through IoC
- Phase 4A — UoW application stack
  - `IKernelApplication`, `ApplicationStack`, `ShellApplication`, `FtpApplication`
- Phase 4B — MINIX execution pipeline
  - `IShellBuiltins`, `IScriptInterpreter`, `IExecutionPipeline`
  - `MinixBuiltins`, `MinixScriptInterpreter`, `MinixExecutionPipeline`
  - `MailApplication`
- Phase 5 — reflective command registration
  - command tagging via `[Command(Name, OsTag)]`
  - `Program.cs` builds command index reflectively
- Phase 6A — legacy bridge cleanup
  - deleted `OperatingSystem.cs`
  - deleted `LegacyUnitOfWork.cs`
  - removed old `ICommand`, `IOperatingSystem`, `CommandContext`, `CommandResult`
- Phase 1 — transport
  - `Framework/Transport/IOutputSink.cs`
  - `Framework/Transport/IInputSource.cs`
  - `ConsoleOutputSink` / `ConsoleInputSource`
  - `TcpOutputSink` / `TcpInputSource`
  - `GameTextWriter`
  - sink-based `Protocol`
  - `Program.cs` now supports stdio mode and `--game-port <port>` TCP mode
  - `engine-io` now has `TcpSidecar`
  - `engine_io_system` appends `--game-port` and uses localhost TCP

## Verification
- C# sidecar build: `dotnet build -c Release` ✅
- stdio smoke test: hello/login/pwd flow ✅
- TCP smoke test: sidecar server accepts client and returns `screen-full` / prompt / `pwd` output ✅
- Rust syntax gate: `rustfmt` successfully parsed and formatted `engine-io/src/lib.rs` and `engine/src/systems/engine_io.rs` ✅
- Full cargo workspace build is currently blocked by a pre-existing missing manifest:
  - `tools/ttf-rasterizer/Cargo.toml`

## Remaining cleanup opportunities
- unify remaining `CognitOS.Kernel.IUnitOfWork` aliases onto the framework interface
- optionally move eggs/apps onto scanner-based registration too
- optionally remove `ScreenBuffer` once transport owns all presentation directly
