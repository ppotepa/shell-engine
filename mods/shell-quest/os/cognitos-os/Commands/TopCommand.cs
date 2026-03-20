using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class TopCommand : ICommand
{
    public string Name => "top";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx, IReadOnlyList<string> args)
    {
        var now = ctx.Os.SimulatedNow();
        var (cpu, mem) = ctx.Os.UsageSnapshot();
        var lines = new[]
        {
            "minix top - simulated",
            $"time: {now:ddd MMM dd HH:mm:ss yyyy}",
            $"cpu: {cpu,5:0.0}%   mem: {mem,5:0.0}%",
            "tasks: 8 total, 1 running, 7 sleeping",
            "pid  user   cpu%   mem%   command",
            "1    root    0.5    2.1   init",
            "17   root    1.1    1.8   netd",
            "21   root    0.6    1.4   maild",
            "42   linus  " + $"{Math.Max(1.0, cpu - 1.8),5:0.0}" + "    3.2   shell",
        };
        return new CommandResult(lines);
    }
}
