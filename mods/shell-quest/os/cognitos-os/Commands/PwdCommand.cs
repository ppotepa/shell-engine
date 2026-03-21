using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class PwdCommand : ICommand
{
    public string Name => "pwd";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
        => new CommandResult(new[] { ctx.Session.Cwd });
}
