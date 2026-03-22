using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class DateCommand : ICommand
{
    public string Name => "date";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        var now = ctx.Os.SimulatedNow();
        var anomalyCount = ctx.Os.State.Quest.AnomaliesDiscovered?.Count ?? 0;

        // ~5% chance of date glitch after all 3 anomalies
        if (anomalyCount >= 3 && Random.Shared.Next(20) == 0)
        {
            var realNow = DateTime.UtcNow;
            return new CommandResult(new[]
            {
                now.ToString("ddd MMM dd HH:mm:ss 'EET' yyyy"),
                realNow.ToString("ddd MMM dd HH:mm:ss 'EET' yyyy"),
                now.ToString("ddd MMM dd HH:mm:ss 'EET' yyyy"),
            });
        }

        return new CommandResult(new[] { now.ToString("ddd MMM dd HH:mm:ss 'EET' yyyy") });
    }
}
