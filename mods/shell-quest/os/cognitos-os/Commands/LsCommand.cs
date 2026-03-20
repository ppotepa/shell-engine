using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class LsCommand : ICommand
{
    public string Name => "ls";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx, IReadOnlyList<string> args)
    {
        var path = args.Count > 0 ? args[0] : null;
        var entries = ctx.Os.FileSystem.Ls(path).ToArray();
        if (entries.Length == 0)
        {
            return new CommandResult(new[] { Style.Fg(Style.Error, "ls: no such file or directory") });
        }

        return new CommandResult(new[] { string.Join("  ", entries) });
    }
}
