using CognitosOs.Core;
using CognitosOs.Kernel;

namespace CognitosOs.Commands;

internal sealed class HistoryCommand : IKernelCommand
{
    public string Name => "history";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    internal List<string> CommandLog { get; } = new();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        for (int i = 0; i < CommandLog.Count; i++)
            uow.Out.WriteLine($"  {i + 1}  {CommandLog[i]}");
        return 0;
    }
}
