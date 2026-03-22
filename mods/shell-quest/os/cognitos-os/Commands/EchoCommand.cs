using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class EchoCommand : ICommand
{
    public string Name => "echo";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        var text = string.Join(" ", ctx.Argv);

        // Basic variable expansion
        text = text.Replace("$USER", ctx.Session.User);
        text = text.Replace("$HOME", ctx.Session.Home);
        text = text.Replace("$SHELL", "/bin/sh");
        text = text.Replace("$HOSTNAME", ctx.Session.Hostname);
        text = text.Replace("$PWD", ctx.Session.Cwd);
        text = text.Replace("$?", ctx.Session.LastExitCode.ToString());

        return new CommandResult(new[] { text });
    }
}
