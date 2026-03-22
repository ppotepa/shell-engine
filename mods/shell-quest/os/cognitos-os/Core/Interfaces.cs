using CognitosOs.State;
using CognitosOs.Kernel;

namespace CognitosOs.Core;

internal interface ICommand
{
    string Name { get; }
    IReadOnlyList<string> Aliases { get; }
    CommandResult Execute(CommandContext ctx);
}

/// <summary>
/// New-style command interface. Commands receive a <see cref="IUnitOfWork"/>
/// and write output directly via <c>uow.Out.WriteLine()</c>.
/// Returns exit code only. Replaces <see cref="ICommand"/> incrementally.
/// </summary>
internal interface IKernelCommand
{
    string Name { get; }
    IReadOnlyList<string> Aliases { get; }

    /// <summary>
    /// Execute the command. Write output to <c>uow.Out</c>.
    /// Return 0 for success, non-zero for error.
    /// </summary>
    int Run(IUnitOfWork uow, string[] argv);
}

internal interface IOperatingSystem
{
    MachineState State { get; }
    MachineSpec Spec { get; }
    IVirtualFileSystem FileSystem { get; }
    void Tick(ulong dtMs);
    DateTime SimulatedNow();
    (double CpuPercent, double MemoryPercent) UsageSnapshot();
    IReadOnlyList<ProcessEntry> ProcessSnapshot();
    IReadOnlyList<ServiceEntry> ServiceSnapshot();
    int UnreadMailCount();
    void MarkMailRead(string targetPath);
}

internal interface IMachineStart
{
    MachineState LoadOrCreate();
    void Persist(MachineState state);
}

internal interface IBootSequence
{
    IReadOnlyList<BootStep> BuildBootSteps(CognitosOs.Framework.Kernel.IKernel kernel);
}

internal sealed record BootStep(string Text, ulong DelayMs);

internal sealed record CommandContext(
    IOperatingSystem Os,
    UserSession Session,
    string CommandName,
    IReadOnlyList<string> Argv)
{
    /// <summary>Resolves a user-supplied path via the current session.</summary>
    public string ResolvePath(string? target) => Session.ResolvePath(target);
}

internal sealed record CommandResult(
    IReadOnlyList<string> Lines,
    int ExitCode = 0,
    bool ClearScreen = false,
    /// <summary>
    /// When non-null, the shell launches this named application after executing
    /// the command. Recognised values: "ftp".
    /// </summary>
    string? LaunchApp = null);
