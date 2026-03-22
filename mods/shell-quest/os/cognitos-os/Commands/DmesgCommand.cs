using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class DmesgCommand : ICommand
{
    public string Name => "dmesg";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        var spec = ctx.Os.Spec;
        var kernelMem = 109;
        var freeMem = spec.RamKb - kernelMem;

        var lines = new List<string>
        {
            "MINIX 1.1 boot",
            $"memory: {spec.RamKb}K total, {kernelMem}K kernel, {freeMem}K free",
            $"hd driver: winchester, {spec.DiskKb}K",
            "clock: 100 Hz tick",
            "tty: 3 virtual consoles",
            $"ethernet: {spec.NicModel} at 0x300, IRQ 9",
            "root filesystem: /dev/hd1 (minix)",
            "/usr filesystem: /dev/hd2 (minix)",
            "init: starting /etc/rc",
        };

        // After all 3 anomalies: spooky kernel line
        var anomalyCount = ctx.Os.State.Quest.AnomaliesDiscovered?.Count ?? 0;
        if (anomalyCount >= 3)
            lines.Add("[????] process 0: unnamed: started");

        return new CommandResult(lines);
    }
}
