using CognitOS.Core;
using CognitOS.Kernel;

namespace CognitOS.Commands;

[CognitOS.Framework.Ioc.Command("clear", OsTag = "universal")]
internal sealed class ClearCommand : IKernelCommand
{
    public string Name => "clear";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv) => 901;
}
