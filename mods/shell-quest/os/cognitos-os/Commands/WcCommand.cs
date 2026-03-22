using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class WcCommand : ICommand
{
    public string Name => "wc";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        if (ctx.Argv.Count < 1)
            return new CommandResult(new[] { "usage: wc <file>" }, 1);

        var vfsPath = ctx.Os.FileSystem.ToVfsPath(ctx.Session.ResolvePath(ctx.Argv[0]));
        if (!ctx.Os.FileSystem.TryCat(vfsPath, out var content))
            return new CommandResult(new[] { $"wc: {ctx.Argv[0]}: No such file or directory" }, 1);

        var lines = content.Replace("\r\n", "\n").Split('\n').Length;
        var words = content.Split(new[] { ' ', '\n', '\r', '\t' }, StringSplitOptions.RemoveEmptyEntries).Length;
        var bytes = content.Length;

        return new CommandResult(new[] { $"  {lines}  {words}  {bytes} {ctx.Argv[0]}" });
    }
}
