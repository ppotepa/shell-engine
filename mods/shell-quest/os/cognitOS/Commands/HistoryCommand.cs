using CognitOS.Core;
using CognitOS.Kernel;

namespace CognitOS.Commands;

[CognitOS.Framework.Ioc.Command("history", OsTag = "minix")]
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
