using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class CatCommand : ICommand
{
    public string Name => "cat";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        var args = ctx.Argv;
        if (args.Count < 1)
        {
            return new CommandResult(new[] { Style.Fg(Style.Warn, "usage: cat <file>") }, ExitCode: 2);
        }

        var target = args[0];
        if (target is "mail" or "notes")
        {
            return new CommandResult(new[] { Style.Fg(Style.Error, $"cat: {target}: is a directory") }, ExitCode: 1);
        }

        if (!ctx.Os.FileSystem.TryCat(target, out var content))
        {
            return new CommandResult(new[] { Style.Fg(Style.Error, $"cat: {target}: no such file or directory") }, ExitCode: 1);
        }

        ctx.Os.MarkMailRead(target);
        var lines = content.Replace("\r\n", "\n").Split('\n');
        return new CommandResult(lines, ExitCode: 0);
    }
}
