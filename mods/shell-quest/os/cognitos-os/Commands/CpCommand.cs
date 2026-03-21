using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class CpCommand : ICommand
{
    public string Name => "cp";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        if (ctx.Argv.Count < 2)
            return new CommandResult(new[] { "usage: cp <source> <dest>" }, 1);

        var srcAbsolute = ctx.Session.ResolvePath(ctx.Argv[0]);
        var dstAbsolute = ctx.Session.ResolvePath(ctx.Argv[1]);
        var srcVfs = ctx.Os.FileSystem.ToVfsPath(srcAbsolute);
        var dstVfs = ctx.Os.FileSystem.ToVfsPath(dstAbsolute);

        if (!ctx.Os.FileSystem.TryCat(srcVfs, out _))
            return new CommandResult(new[] { $"cp: {ctx.Argv[0]}: No such file or directory" }, 1);

        if (ctx.Os.Spec.DiskFreeKb < 100)
            return new CommandResult(new[] { $"cp: {ctx.Argv[1]}: No space left on device" }, 1);

        if (ctx.Os.FileSystem is State.IMutableFileSystem mutableFs)
        {
            if (!mutableFs.TryCopy(srcVfs, dstVfs, out var error))
                return new CommandResult(new[] { $"cp: {error}" }, 1);

            ctx.Os.State.Quest.BackupMade = true;
            return new CommandResult(Array.Empty<string>());
        }

        return new CommandResult(new[] { "cp: read-only file system" }, 1);
    }
}
