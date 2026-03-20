using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class HelpCommand : ICommand
{
    public string Name => "help";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        return new CommandResult(new[] { "commands: ls  cat <file>  top  ps  services  clear  help" }, ExitCode: 0);
    }
}
