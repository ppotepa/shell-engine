using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class CatCommand : ICommand
{
    public string Name => "cat";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx, IReadOnlyList<string> args)
    {
        if (args.Count < 1)
        {
            return new CommandResult(new[] { Style.Fg(Style.Warn, "usage: cat <file>") });
        }

        var target = args[0];
        if (target is "mail" or "notes")
        {
            return new CommandResult(new[] { Style.Fg(Style.Error, $"cat: {target}: is a directory") });
        }

        if (!ctx.Os.FileSystem.TryCat(target, out var content))
        {
            return new CommandResult(new[] { Style.Fg(Style.Error, $"cat: {target}: no such file or directory") });
        }

        var lines = content.Replace("\r\n", "\n").Split('\n');
        return new CommandResult(lines);
    }
}
