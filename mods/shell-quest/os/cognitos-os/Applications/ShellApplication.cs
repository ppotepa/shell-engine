using CognitosOs.Commands;
using CognitosOs.Core;
using CognitosOs.Kernel;
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
    private readonly IReadOnlyDictionary<string, IKernelCommand> _commandIndex;
    private readonly ScreenBuffer _screen;
    private readonly ApplicationStack _stack;
    private readonly EasterEggRegistry _eggs;
    private readonly HistoryCommand _historyCmd;
    private readonly Func<UserSession, IUnitOfWork> _createUow;

    public ShellApplication(
        IOperatingSystem os,
        IReadOnlyDictionary<string, IKernelCommand> commandIndex,
        ScreenBuffer screen, ApplicationStack stack,
        EasterEggRegistry eggs, HistoryCommand historyCmd,
        Func<UserSession, IUnitOfWork> createUow)
    {
        _os = os;
        _commandIndex = commandIndex;
        _screen = screen;
        _stack = stack;
        _eggs = eggs;
        _historyCmd = historyCmd;
        _createUow = createUow;
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

        _historyCmd.CommandLog.Add(submitted);

        var parts = submitted.Split(' ', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);
        var cmd = parts[0];

        // Strict-1991: no GNU-style --help. MINIX used man pages.
        if (parts.Skip(1).Any(a => a == "--help"))
        {
            session.LastExitCode = 1;
            _screen.Append($"{cmd}: illegal option -- -", $"Try: man {cmd}", "");
            return ApplicationResult.Continue;
        }

        if (!_commandIndex.TryGetValue(cmd, out var command))
        {
            // Try easter eggs before "command not found"
            using var eggUow = _createUow(session);
            var exitCode = _eggs.TryHandle(eggUow, cmd, parts);
            if (exitCode.HasValue)
            {
                session.LastExitCode = exitCode.Value;
                FlushOutput(eggUow);
                return ApplicationResult.Continue;
            }

            session.LastExitCode = 127;
            _screen.Append(Style.Fg(Style.Error, $"{cmd}: command not found"), "");
            return ApplicationResult.Continue;
        }

        using var uow = _createUow(session);
        var result = command.Run(uow, parts);
        session.LastExitCode = result;

        switch (result)
        {
            case 901: // clear screen
                _screen.ClearViewport();
                break;
            case 900: // launch FTP app
                FlushOutput(uow);
                _stack.Push(new FtpApplication(_os, _screen), session);
                break;
            default:
                FlushOutput(uow);
                break;
        }

        return ApplicationResult.Continue;
    }

    private void FlushOutput(IUnitOfWork uow)
    {
        uow.Out.Flush();
        var text = uow.Out.ToString()!;
        if (text.Length > 0)
        {
            var lines = text.TrimEnd('\r', '\n').Split('\n');
            _screen.Append(lines.Concat(new[] { "" }).ToArray());
        }
        else
        {
            _screen.Append("");
        }
    }
}
