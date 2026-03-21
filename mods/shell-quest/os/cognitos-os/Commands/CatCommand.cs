using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class CatCommand : ICommand
{
    public string Name => "cat";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        if (ctx.Argv.Count < 1)
            return new CommandResult(new[] { Style.Fg(Style.Warn, "usage: cat <file>") }, 2);

        var absolute = ctx.Session.ResolvePath(ctx.Argv[0]);
        var vfsPath = ctx.Os.FileSystem.ToVfsPath(absolute);

        // directory guard
        if (ctx.Os.FileSystem.DirectoryExists(vfsPath) && !ctx.Os.FileSystem.TryCat(vfsPath, out _))
            return new CommandResult(new[] { Style.Fg(Style.Error, $"cat: {ctx.Argv[0]}: is a directory") }, 1);

        if (!ctx.Os.FileSystem.TryCat(vfsPath, out var content))
            return new CommandResult(new[] { Style.Fg(Style.Error, $"cat: {ctx.Argv[0]}: no such file or directory") }, 1);

        ctx.Os.MarkMailRead(vfsPath);
        var lines = content.Replace("\r\n", "\n").Split('\n');
        return new CommandResult(lines);
    }
}
