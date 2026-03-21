using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class LsCommand : ICommand
{
    public string Name => "ls";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        var absolute = ctx.Argv.Count > 0
            ? ctx.Session.ResolvePath(ctx.Argv[0])
            : ctx.Session.Cwd;

        var vfsPath = ctx.Os.FileSystem.ToVfsPath(absolute);
        var entries = ctx.Os.FileSystem.Ls(vfsPath).ToArray();

        if (entries.Length == 0)
            return new CommandResult(new[] { Style.Fg(Style.Error, "ls: no such file or directory") }, 2);

        return new CommandResult(new[] { string.Join("  ", entries) });
    }
}
