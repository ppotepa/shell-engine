using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class CdCommand : ICommand
{
    public string Name => "cd";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        var target = ctx.Argv.Count > 0 ? ctx.Argv[0] : "~";
        var resolved = ctx.Session.ResolvePath(target);
        var vfsPath = ctx.Os.FileSystem.ToVfsPath(resolved);

        if (!ctx.Os.FileSystem.DirectoryExists(vfsPath))
            return new CommandResult(new[] { $"cd: {target}: No such file or directory" }, 1);

        ctx.Session.SetCwd(resolved);
        return new CommandResult(Array.Empty<string>());
    }
}
