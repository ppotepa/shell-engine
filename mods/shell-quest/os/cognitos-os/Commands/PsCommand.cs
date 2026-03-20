using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class PsCommand : ICommand
{
    public string Name => "ps";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        var lines = new List<string> { "pid  user   state     command" };
        foreach (var process in ctx.Os.ProcessSnapshot().OrderBy(p => p.Pid))
        {
            lines.Add($"{process.Pid,-4} {process.User,-6} {process.State,-9} {process.Name}");
        }

        return new CommandResult(lines, ExitCode: 0);
    }
}
