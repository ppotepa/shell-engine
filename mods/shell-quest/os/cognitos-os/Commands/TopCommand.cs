using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class TopCommand : ICommand
{
    public string Name => "top";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        var now = ctx.Os.SimulatedNow();
        var (cpu, mem) = ctx.Os.UsageSnapshot();
        var processes = ctx.Os.ProcessSnapshot();
        var lines = new List<string>
        {
            "minix top - simulated",
            $"time: {now:ddd MMM dd HH:mm:ss yyyy}",
            $"cpu: {cpu,5:0.0}%   mem: {mem,5:0.0}%",
            $"tasks: {processes.Count} total, {processes.Count(p => p.State == "running")} running, {processes.Count(p => p.State != "running")} sleeping",
            "pid  user   cpu%   mem%   command",
        };
        foreach (var process in processes.OrderBy(p => p.Pid))
        {
            lines.Add($"{process.Pid,-4} {process.User,-6} {process.CpuPercent,5:0.0}   {process.MemoryPercent,5:0.0}   {process.Name}");
        }
        return new CommandResult(lines, ExitCode: 0);
    }
}
