using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class HostnameCommand : ICommand
{
    public string Name => "hostname";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
        => new(new[] { ctx.Session.Hostname });
}
