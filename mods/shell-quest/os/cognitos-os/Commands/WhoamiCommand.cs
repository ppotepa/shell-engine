using CognitosOs.Core;
using CognitosOs.Kernel;

namespace CognitosOs.Commands;

[CognitosOs.Framework.Ioc.Command("whoami", OsTag = "minix")]
internal sealed class WhoamiCommand : IKernelCommand
{
    public string Name => "whoami";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        uow.Out.WriteLine(uow.Session.User);
        return 0;
    }
}
