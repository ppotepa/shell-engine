using CognitosOs.Commands;
using CognitosOs.Core;
using CognitosOs.Network;
using CognitosOs.State;

namespace CognitosOs.Applications;

/// <summary>
/// The base shell application. Always sits at the bottom of the application stack.
/// Handles command dispatch including builtins, easter eggs, and launching child applications.
/// </summary>
internal sealed class ShellApplication : IApplication
{
    private readonly IOperatingSystem _os;
    private readonly ScreenBuffer _screen;
    private readonly ApplicationStack _stack;
    private readonly EasterEggRegistry _eggs;
    private readonly HistoryCommand _historyCmd;

    public ShellApplication(
        IOperatingSystem os, ScreenBuffer screen, ApplicationStack stack,
        EasterEggRegistry eggs, HistoryCommand historyCmd)
    {
        _os = os;
        _screen = screen;
        _stack = stack;
        _eggs = eggs;
        _historyCmd = historyCmd;
    }

    public string PromptPrefix(UserSession session)
    {
        var user = session.User;
        var host = session.Hostname;
        var cwd = session.DisplayCwd();
        return $"{Style.Fg(Style.PromptUser, user)}@{Style.Fg(Style.PromptHost, host)}:{Style.Fg(Style.PromptPath, cwd)} [{session.LastExitCode}]$ ";
    }

    public void OnEnter(UserSession session) { }
    public void OnExit(UserSession session) { }

    public ApplicationResult HandleInput(string input, UserSession session)
    {
        var submitted = input.Trim();
        if (string.IsNullOrWhiteSpace(submitted))
            return ApplicationResult.Continue;

        // Track command history
        _historyCmd.CommandLog.Add(submitted);

        var parts = submitted.Split(' ', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);
        var cmd = parts[0];
        var argv = (IReadOnlyList<string>)parts.Skip(1).ToArray();

        if (!_os.CommandIndex.TryGetValue(cmd, out var command))
        {
            // Try easter eggs before "command not found"
            var ctx = new CommandContext(_os, session, cmd, argv);
            var eggResult = _eggs.TryHandle(cmd, argv, ctx);
            if (eggResult != null)
            {
                session.LastExitCode = eggResult.ExitCode;
                if (eggResult.Lines.Count > 0)
                    _screen.Append(eggResult.Lines.Concat(new[] { "" }).ToArray());
                else
                    _screen.Append("");
                return ApplicationResult.Continue;
            }

            session.LastExitCode = 127;
            _screen.Append(Style.Fg(Style.Error, $"{cmd}: command not found"), "");
            return ApplicationResult.Continue;
        }

        var cmdCtx = new CommandContext(_os, session, cmd, argv);
        var result = command.Execute(cmdCtx);
        session.LastExitCode = result.ExitCode;

        if (result.ClearScreen)
            _screen.ClearViewport();
        else
            _screen.Append(result.Lines.Count == 0
                ? new[] { "" }
                : result.Lines.Concat(new[] { "" }).ToArray());

        if (result.LaunchApp == "ftp")
            _stack.Push(new FtpApplication(_os, _screen), session);

        return ApplicationResult.Continue;
    }
}
