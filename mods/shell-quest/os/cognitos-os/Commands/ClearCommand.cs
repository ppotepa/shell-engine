using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class ClearCommand : ICommand
{
    public string Name => "clear";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
        => new(Array.Empty<string>(), ExitCode: 0, ClearScreen: true);
}
