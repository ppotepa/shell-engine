using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class WhoCommand : ICommand
{
    public string Name => "who";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        var now = ctx.Os.SimulatedNow();
        var lines = new List<string>
        {
            $"linus    tty0     {now:MMM dd HH:mm}",
            "ast      tty1     Sep 15 09:41",
            "         tty2     Jan  1 00:00",
        };

        // After quest complete, tty2 disappears
        if (ctx.Os.State.Quest.UploadSuccess)
        {
            lines.RemoveAt(2);
        }

        return new CommandResult(lines);
    }
}
