using CognitosOs.Core;

namespace CognitosOs.Commands;

/// <summary>
/// Enters the FTP client application. Signals the shell to push a
/// FtpApplication onto the application stack via CommandResult.LaunchApp.
/// </summary>
internal sealed class FtpCommand : ICommand
{
    public string Name => "ftp";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        if (ctx.Argv.Count > 0)
            ctx.Os.State.Quest.FtpRemoteHost = ctx.Argv[0];

        return new CommandResult(Array.Empty<string>(), LaunchApp: "ftp");
    }
}
