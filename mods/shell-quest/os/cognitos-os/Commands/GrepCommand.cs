using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class GrepCommand : ICommand
{
    public string Name => "grep";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        if (ctx.Argv.Count < 2)
            return new CommandResult(new[] { "usage: grep <pattern> <file>" }, 1);

        var pattern = ctx.Argv[0];
        var filePath = ctx.Session.ResolvePath(ctx.Argv[1]);
        var vfsPath = ctx.Os.FileSystem.ToVfsPath(filePath);

        if (!ctx.Os.FileSystem.TryCat(vfsPath, out var content))
            return new CommandResult(new[] { $"grep: {ctx.Argv[1]}: No such file or directory" }, 2);

        var matches = content
            .Replace("\r\n", "\n")
            .Split('\n')
            .Where(line => line.Contains(pattern, StringComparison.OrdinalIgnoreCase))
            .ToArray();

        if (matches.Length == 0)
            return new CommandResult(Array.Empty<string>(), 1);

        return new CommandResult(matches);
    }
}
