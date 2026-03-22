using CognitosOs.Core;
using CognitosOs.Kernel;

namespace CognitosOs.Commands;

[CognitosOs.Framework.Ioc.Command("clear", OsTag = "universal")]
internal sealed class ClearCommand : IKernelCommand
{
    public string Name => "clear";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv) => 901;
}
