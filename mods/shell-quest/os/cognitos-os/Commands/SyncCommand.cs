using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class SyncCommand : ICommand
{
    public string Name => "sync";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        return new CommandResult(new[] { "(syncing disks...)" });
    }
}
