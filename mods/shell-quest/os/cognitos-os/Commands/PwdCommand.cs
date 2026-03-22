using CognitosOs.Core;
using CognitosOs.Kernel;

namespace CognitosOs.Commands;

internal sealed class PwdCommand : IKernelCommand
{
    public string Name => "pwd";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        uow.Out.WriteLine(uow.Session.Cwd);
        return 0;
    }
}
