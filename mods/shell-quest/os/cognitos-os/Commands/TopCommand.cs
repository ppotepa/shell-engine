using CognitosOs.Core;
using CognitosOs.Kernel;

namespace CognitosOs.Commands;

internal sealed class TopCommand : IKernelCommand
{
    public string Name => "top";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        var now = uow.Clock.Now();
        var res = uow.Resources;
        var processes = uow.Process.List();
        var cpuPct = res.CpuLoadFactor * 100;
        var memPct = res.TotalRamKb > 0 ? (double)(res.TotalRamKb - res.FreeRamKb) / res.TotalRamKb * 100 : 0;

        uow.Out.WriteLine("minix top - simulated");
        uow.Out.WriteLine($"time: {now:ddd MMM dd HH:mm:ss yyyy}");
        uow.Out.WriteLine($"cpu: {cpuPct,5:0.0}%   mem: {memPct,5:0.0}%");
        uow.Out.WriteLine($"tasks: {processes.Count} total");
        uow.Out.WriteLine("pid  user   sz     command");
        foreach (var p in processes.OrderBy(p => p.Pid))
            uow.Out.WriteLine($"{p.Pid,-4} {p.User,-6} {p.Sz,5}K  {p.Name}");

        return 0;
    }
}
