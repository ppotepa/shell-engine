using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class FileCommand : ICommand
{
    public string Name => "file";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        if (ctx.Argv.Count < 1)
            return new CommandResult(new[] { "usage: file <path>" }, 1);

        var path = ctx.Argv[0];
        var vfsPath = ctx.Os.FileSystem.ToVfsPath(ctx.Session.ResolvePath(path));

        if (ctx.Os.FileSystem.DirectoryExists(vfsPath))
            return new CommandResult(new[] { $"{path}: directory" });

        if (!ctx.Os.FileSystem.TryCat(vfsPath, out var content))
            return new CommandResult(new[] { $"{path}: cannot open" }, 1);

        var type = path switch
        {
            _ when path.EndsWith(".tar.Z") => "compressed data (compress'd)",
            _ when path.EndsWith(".Z") => "compressed data",
            _ when path.EndsWith(".tar") => "POSIX tar archive",
            _ when content.StartsWith("[COMPRESSED") => "compressed data",
            _ when content.StartsWith("[binary") || content.StartsWith("[core") => "data",
            _ => "ASCII text",
        };

        return new CommandResult(new[] { $"{path}: {type}" });
    }
}
