using CognitOS.Core;
using CognitOS.Kernel;

namespace CognitOS.Commands;

[CognitOS.Framework.Ioc.Command("dmesg", OsTag = "minix")]
internal sealed class DmesgCommand : IKernelCommand
{
    public string Name => "dmesg";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        var spec = uow.Spec;
        var res = uow.Resources;

        uow.Out.WriteLine("MINIX 1.1 boot");
        uow.Out.WriteLine($"memory: {res.TotalRamKb}K total, {res.KernelKb}K kernel, {res.FreeRamKb}K free");
        uow.Out.WriteLine($"hd driver: winchester, {res.DiskTotalKb}K");
        uow.Out.WriteLine("clock: 100 Hz tick");
        uow.Out.WriteLine("tty: 3 virtual consoles");
        uow.Out.WriteLine($"ethernet: {spec.NicModel} at 0x300, IRQ 9");
        uow.Out.WriteLine("root filesystem: /dev/hd1 (minix)");
        uow.Out.WriteLine("/usr filesystem: /dev/hd2 (minix)");
        uow.Out.WriteLine("init: starting /etc/rc");

        var anomalyCount = uow.Quest.AnomaliesDiscovered?.Count ?? 0;
        if (anomalyCount >= 3)
            uow.Out.WriteLine("[????] process 0: unnamed: started");

        return 0;
    }
}
