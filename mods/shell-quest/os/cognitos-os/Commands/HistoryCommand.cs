using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class HistoryCommand : ICommand
{
    public string Name => "history";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    // Maintained by ShellApplication on each input
    internal List<string> CommandLog { get; } = new();

    public CommandResult Execute(CommandContext ctx)
    {
        var lines = new List<string>();
        for (int i = 0; i < CommandLog.Count; i++)
            lines.Add($"  {i + 1}  {CommandLog[i]}");
        return new CommandResult(lines);
    }
}
