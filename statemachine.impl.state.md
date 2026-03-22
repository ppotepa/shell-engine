# State Machine Implementation Status

## Worktree
- Branch: `state-machine`
- Worktree path: `/home/ppotepa/git/shell-quest-state-machine`
- Last pushed commit: `b8aef21` — `feat(cognitos-os): Phase 0+2 — IoC container + ISyscallGate`

## Completed phases
- Phase 0: IoC container
  - `Framework/Ioc/ServiceContainer.cs`
  - `Framework/Ioc/ServiceLifetime.cs`
  - `Framework/Ioc/ServiceDescriptor.cs`
  - `Framework/Ioc/IOperatingSystemModule.cs`
  - `Framework/Ioc/CommandAttribute.cs`
  - `Framework/Ioc/CommandScanner.cs`
- Phase 2: ISyscallGate
  - `Framework/Kernel/SyscallKind.cs`
  - `Framework/Kernel/SyscallRequest.cs`
  - `Framework/Kernel/SyscallResult.cs`
  - `Framework/Kernel/SyscallException.cs`
  - `Framework/Kernel/ISyscallGate.cs`
  - `Minix/Kernel/MinixSyscallGate.cs`
  - Subsystems routed through gate:
    - `Kernel/Disk/SimulatedDisk.cs`
    - `Kernel/Process/SimulatedProcessTable.cs`
    - `Kernel/Network/SimulatedNetwork.cs`

## Current phase in progress
- Phase 3: Kernel wiring
  - `Framework/Kernel/IKernel.cs` created
  - `Framework/Kernel/IUnitOfWork.cs` created
  - `Minix/MinixModule.cs` created
  - Partial rewrites started in:
    - `Core/AppHost.cs`
    - `Program.cs`
    - `Boot/BootSequence.cs`
    - `Applications/ShellApplication.cs`
    - `Applications/FtpApplication.cs`
    - `Kernel/Kernel.cs`
    - `Kernel/IUnitOfWork.cs`
    - `Kernel/LegacyUnitOfWork.cs`
    - `Core/Interfaces.cs`

## Current build state
- Build is currently **failing**
- Command used:
  - `cd mods/shell-quest/os/cognitos-os && dotnet build -c Release`

### Current errors
1. `Applications/ShellApplication.cs`
   - `CognitosOs.Framework.Kernel.IUnitOfWork` cannot be passed where `CognitosOs.Kernel.IUnitOfWork` is expected
   - Cause: mixed old/new `IUnitOfWork` types across command/easter egg call sites

2. `Minix/MinixModule.cs`
   - `IKernel` ambiguous between `CognitosOs.Kernel.IKernel` and `CognitosOs.Framework.Kernel.IKernel`
   - `Kernel` used as namespace instead of concrete type
   - Cause: namespace/type collision during new framework kernel registration

## Working tree changes not yet committed
- Modified:
  - `mods/shell-quest/os/cognitos-os/Applications/FtpApplication.cs`
  - `mods/shell-quest/os/cognitos-os/Applications/ShellApplication.cs`
  - `mods/shell-quest/os/cognitos-os/Boot/BootSequence.cs`
  - `mods/shell-quest/os/cognitos-os/Core/AppHost.cs`
  - `mods/shell-quest/os/cognitos-os/Core/Interfaces.cs`
  - `mods/shell-quest/os/cognitos-os/Kernel/IUnitOfWork.cs`
  - `mods/shell-quest/os/cognitos-os/Kernel/Kernel.cs`
  - `mods/shell-quest/os/cognitos-os/Kernel/LegacyUnitOfWork.cs`
  - `mods/shell-quest/os/cognitos-os/Program.cs`
- Untracked:
  - `mods/shell-quest/os/cognitos-os/Framework/Kernel/IKernel.cs`
  - `mods/shell-quest/os/cognitos-os/Framework/Kernel/IUnitOfWork.cs`
  - `mods/shell-quest/os/cognitos-os/Minix/MinixModule.cs`

## Todo state
- Done: `8`
- In progress: `6`
- Pending: `18`

## Recommended next fixes
1. Unify `IUnitOfWork` usage so command/easter egg paths use one canonical type
2. Resolve `IKernel` ambiguity with explicit namespace aliasing
3. Fix `MinixModule` to instantiate the concrete kernel type explicitly
4. Rebuild until Phase 3 is green
5. Commit and push Phase 3 to `origin/state-machine`
