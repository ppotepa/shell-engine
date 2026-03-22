using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class KillCommand : ICommand
{
    public string Name => "kill";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        if (ctx.Argv.Count < 1)
            return new CommandResult(new[] { "usage: kill <pid>" }, 1);

        if (!int.TryParse(ctx.Argv[0], out var pid))
            return new CommandResult(new[] { $"kill: {ctx.Argv[0]}: arguments must be process IDs" }, 1);

        var process = ctx.Os.ProcessSnapshot().FirstOrDefault(p => p.Pid == pid);
        if (process is null)
            return new CommandResult(new[] { $"kill: ({pid}) - No such process" }, 1);

        if (process.User != ctx.Session.User)
            return new CommandResult(new[] { $"kill: ({pid}) - Not owner" }, 1);

        return new CommandResult(new[] { $"kill: ({pid}) - Operation not permitted" }, 1);
    }
}
