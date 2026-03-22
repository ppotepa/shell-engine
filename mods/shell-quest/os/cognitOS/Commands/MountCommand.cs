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
        foreach (var m in uow.Mounts.GetMounts())
            uow.Out.WriteLine($"{m.Device} on {m.MountPoint} type {m.FsType} ({m.Options}) [{m.SizeKb}K]");
        return 0;
    }
}
