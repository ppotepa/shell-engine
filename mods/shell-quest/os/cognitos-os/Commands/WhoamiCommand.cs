using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class WhoamiCommand : ICommand
{
    public string Name => "whoami";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
        => new(new[] { ctx.Session.User });
}
