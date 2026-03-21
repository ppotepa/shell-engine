using CognitosOs.State;

namespace CognitosOs.Core;

internal interface ICommand
{
    string Name { get; }
    IReadOnlyList<string> Aliases { get; }
    CommandResult Execute(CommandContext ctx);
}

internal interface IOperatingSystem
{
    MachineState State { get; }
    MachineSpec Spec { get; }
    IReadOnlyDictionary<string, ICommand> CommandIndex { get; }
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
    IReadOnlyList<BootStep> BuildBootSteps(IOperatingSystem os);
}

internal sealed record BootStep(string Text, ulong DelayMs);

internal sealed record CommandContext(
    IOperatingSystem Os,
    string User,
    string Cwd,
    string CommandName,
    IReadOnlyList<string> Argv
);

internal sealed record CommandResult(IReadOnlyList<string> Lines, int ExitCode = 0, bool ClearScreen = false);
