using CognitosOs.State;
using CognitosOs.Kernel;

namespace CognitosOs.Core;

/// <summary>
/// New-style command interface. Commands receive a <see cref="IUnitOfWork"/>
/// and write output directly via <c>uow.Out.WriteLine()</c>.
/// Returns exit code only.
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
