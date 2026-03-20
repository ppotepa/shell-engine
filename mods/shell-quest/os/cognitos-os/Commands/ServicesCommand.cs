using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class ServicesCommand : ICommand
{
    public string Name => "services";
    public IReadOnlyList<string> Aliases => new[] { "service" };

    public CommandResult Execute(CommandContext ctx)
    {
        var lines = new List<string> { "name   status  last-tick" };
        foreach (var service in ctx.Os.ServiceSnapshot().OrderBy(s => s.Name))
        {
            lines.Add($"{service.Name,-6} {service.Status,-7} {service.LastTickUtc:HH:mm:ss}");
        }

        return new CommandResult(lines, ExitCode: 0);
    }
}
