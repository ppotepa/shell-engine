using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class ClearCommand : ICommand
{
    public string Name => "clear";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx, IReadOnlyList<string> args)
        => new(Array.Empty<string>(), ClearScreen: true);
}
