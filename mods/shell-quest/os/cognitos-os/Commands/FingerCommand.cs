using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class FingerCommand : ICommand
{
    public string Name => "finger";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        if (ctx.Argv.Count < 1)
            return new CommandResult(new[] { "usage: finger <user>" }, 1);

        var target = ctx.Argv[0].ToLowerInvariant();

        if (target is "linus")
        {
            return new CommandResult(new[]
            {
                "Login: linus                            Name: Linus B. Torvalds",
                "Directory: /home/linus                  Shell: /bin/sh",
                $"On since {ctx.Os.SimulatedNow():MMM dd HH:mm} on tty0",
                "No plan.",
            });
        }

        if (target is "ast" or "tanenbaum")
        {
            var planPath = "usr/ast/.plan";
            var plan = ctx.Os.FileSystem.TryCat(planPath, out var planContent)
                ? planContent
                : "No plan.";

            return new CommandResult(new[]
            {
                "Login: ast                              Name: Andy S. Tanenbaum",
                "Directory: /usr/ast                     Shell: /bin/sh",
                "On since Sep 15 09:41 on tty1",
                $"Plan:\n{plan}",
            });
        }

        if (target is "root")
        {
            return new CommandResult(new[]
            {
                "Login: root                             Name: Charlie Root",
                "Directory: /root                        Shell: /bin/sh",
                "Never logged in.",
            });
        }

        if (target is "tty2")
            return new CommandResult(new[] { "finger: tty2: no such user." }, 1);

        return new CommandResult(new[] { $"finger: {target}: no such user." }, 1);
    }
}
