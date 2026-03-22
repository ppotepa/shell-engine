using CognitOS.Core;
using CognitOS.Kernel;

namespace CognitOS.Commands;

[CognitOS.Framework.Ioc.Command("hostname", OsTag = "minix")]
internal sealed class HostnameCommand : IKernelCommand
{
    public string Name => "hostname";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        uow.Out.WriteLine(uow.Session.Hostname);
        return 0;
    }
}
