using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class HelpCommand : ICommand
{
    public string Name => "help";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx, IReadOnlyList<string> args)
    {
        return new CommandResult(new[] { "commands: ls  cat <file>  top  clear  help" });
    }
}
