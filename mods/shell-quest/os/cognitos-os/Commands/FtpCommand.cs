using CognitosOs.Core;
using CognitosOs.State;

namespace CognitosOs.Commands;

/// <summary>
/// Enters the FTP client session. Switches <see cref="SessionMode"/> to
/// <see cref="SessionMode.FtpSession"/> so <see cref="AppHost"/> routes
/// input to the FTP subsystem instead of the shell command index.
/// </summary>
internal sealed class FtpCommand : ICommand
{
    public string Name => "ftp";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        // ftp <host> — connect immediately
        if (ctx.Argv.Count > 0)
        {
            ctx.Os.State.Quest.FtpRemoteHost = ctx.Argv[0];
        }

        ctx.Os.State.Mode = SessionMode.FtpSession;

        // Return instructions; AppHost will handle the rest
        return new CommandResult(Array.Empty<string>());
    }
}
