using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class EnvCommand : ICommand
{
    public string Name => "env";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        return new CommandResult(new[]
        {
            $"USER={ctx.Session.User}",
            $"HOME={ctx.Session.Home}",
            $"SHELL=/bin/sh",
            $"PATH=/bin:/usr/bin",
            $"TERM=minix",
            $"HOSTNAME={ctx.Session.Hostname}",
            $"PWD={ctx.Session.Cwd}",
            $"LOGNAME={ctx.Session.User}",
        });
    }
}
