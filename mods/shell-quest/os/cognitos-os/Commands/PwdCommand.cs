using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class PwdCommand : ICommand
{
    public string Name => "pwd";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        var cwd = ctx.Os.State.Cwd;
        var display = cwd == "~" ? "/home/linus" : cwd.Replace("~", "/home/linus");
        return new CommandResult(new[] { display });
    }
}
