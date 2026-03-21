using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class CpCommand : ICommand
{
    public string Name => "cp";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        if (ctx.Argv.Count < 2)
        {
            return new CommandResult(new[] { "usage: cp <source> <dest>" }, 1);
        }

        var source = ctx.Argv[0];
        var dest = ctx.Argv[1];

        if (!ctx.Os.FileSystem.TryCat(source, out _))
        {
            return new CommandResult(new[] { $"cp: {source}: No such file or directory" }, 1);
        }

        // Check disk space via MachineSpec
        var spec = ctx.Os.Spec;
        if (spec.DiskFreeKb < 100)
        {
            return new CommandResult(new[] { $"cp: {dest}: No space left on device" }, 1);
        }

        if (ctx.Os.FileSystem is State.IMutableFileSystem mutableFs)
        {
            if (!mutableFs.TryCopy(source, dest, out var error))
            {
                return new CommandResult(new[] { $"cp: {error}" }, 1);
            }
            ctx.Os.State.Quest.BackupMade = true;
            return new CommandResult(Array.Empty<string>());
        }

        return new CommandResult(new[] { "cp: read-only file system" }, 1);
    }
}
