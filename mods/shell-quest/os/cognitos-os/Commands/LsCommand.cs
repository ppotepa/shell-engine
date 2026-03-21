using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class LsCommand : ICommand
{
    public string Name => "ls";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        var args = ctx.Argv;
        var resolved = args.Count > 0 ? ctx.ResolvePath(args[0]) : ctx.ResolvePath(null);
        var entries = ctx.Os.FileSystem.Ls(resolved).ToArray();
        if (entries.Length == 0)
        {
            return new CommandResult(new[] { Style.Fg(Style.Error, "ls: no such file or directory") }, ExitCode: 2);
        }

        return new CommandResult(new[] { string.Join("  ", entries) }, ExitCode: 0);
    }
}
