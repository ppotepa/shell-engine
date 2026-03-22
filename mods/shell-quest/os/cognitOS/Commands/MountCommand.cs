using CognitOS.Core;
using CognitOS.Kernel;

namespace CognitOS.Commands;

[CognitOS.Framework.Ioc.Command("mount", OsTag = "minix")]
internal sealed class MountCommand : IKernelCommand
{
    public string Name => "mount";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        var res = uow.Resources;
        uow.Out.WriteLine($"/dev/hd1 on / type minix (rw) [{res.DiskTotalKb}K]");
        uow.Out.WriteLine($"/dev/hd2 on /usr type minix (rw) [{res.DiskTotalKb / 2}K]");
        return 0;
    }
}
