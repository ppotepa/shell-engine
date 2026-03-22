using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class HeadTailCommand : ICommand
{
    private readonly bool _isHead;
    public string Name { get; }
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public HeadTailCommand(bool isHead)
    {
        _isHead = isHead;
        Name = isHead ? "head" : "tail";
    }

    public CommandResult Execute(CommandContext ctx)
    {
        if (ctx.Argv.Count < 1)
            return new CommandResult(new[] { $"usage: {Name} <file>" }, 1);

        int count = 10;
        var fileArg = ctx.Argv[0];

        // Parse -N flag
        if (ctx.Argv.Count >= 2 && ctx.Argv[0].StartsWith('-') && int.TryParse(ctx.Argv[0][1..], out var n))
        {
            count = n;
            fileArg = ctx.Argv[1];
        }

        var vfsPath = ctx.Os.FileSystem.ToVfsPath(ctx.Session.ResolvePath(fileArg));
        if (!ctx.Os.FileSystem.TryCat(vfsPath, out var content))
            return new CommandResult(new[] { $"{Name}: {fileArg}: No such file or directory" }, 1);

        var allLines = content.Replace("\r\n", "\n").Split('\n');
        var result = _isHead
            ? allLines.Take(count).ToArray()
            : allLines.TakeLast(count).ToArray();

        return new CommandResult(result);
    }
}
