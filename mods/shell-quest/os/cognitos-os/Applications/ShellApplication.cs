using CognitosOs.Core;
using CognitosOs.State;

namespace CognitosOs.Applications;

/// <summary>
/// The base shell application. Always sits at the bottom of the application stack.
/// Handles command dispatch including builtins and launching child applications.
/// </summary>
internal sealed class ShellApplication : IApplication
{
    private readonly IOperatingSystem _os;
    private readonly ScreenBuffer _screen;
    private readonly ApplicationStack _stack;

    public ShellApplication(IOperatingSystem os, ScreenBuffer screen, ApplicationStack stack)
    {
        _os = os;
        _screen = screen;
        _stack = stack;
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

        var parts = submitted.Split(' ', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);
        var cmd = parts[0];
        var argv = (IReadOnlyList<string>)parts.Skip(1).ToArray();

        if (!_os.CommandIndex.TryGetValue(cmd, out var command))
        {
            session.LastExitCode = 127;
            _screen.Append(Style.Fg(Style.Error, $"{cmd}: command not found"), "");
            return ApplicationResult.Continue;
        }

        var ctx = new CommandContext(_os, session, cmd, argv);
        var result = command.Execute(ctx);
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
