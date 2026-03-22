using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class UptimeCommand : ICommand
{
    public string Name => "uptime";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        var now = ctx.Os.SimulatedNow();
        var uptime = TimeSpan.FromMilliseconds(ctx.Os.State.UptimeMs);
        var days = (int)uptime.TotalDays;
        var hours = uptime.Hours;
        var minutes = uptime.Minutes;

        var uptimeStr = days > 0
            ? $"{days} day{(days != 1 ? "s" : "")}, {hours:D2}:{minutes:D2}"
            : $"{hours:D2}:{minutes:D2}";

        var load1 = 0.30 + Random.Shared.NextDouble() * 0.25;
        var load5 = 0.25 + Random.Shared.NextDouble() * 0.20;
        var load15 = 0.20 + Random.Shared.NextDouble() * 0.15;

        return new CommandResult(new[]
        {
            $" {now:HH:mm:ss} up {uptimeStr},  3 users,  load average: {load1:F2}, {load5:F2}, {load15:F2}"
        });
    }
}
