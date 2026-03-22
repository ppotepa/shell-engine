using CognitosOs.Commands;
using CognitosOs.Core;
using CognitosOs.Framework.Kernel;
using CognitosOs.Network;
using CognitosOs.State;

namespace CognitosOs.Applications;

/// <summary>
/// The base shell application. Always sits at the bottom of the application stack.
/// Handles command dispatch including builtins, easter eggs, and launching child applications.
/// </summary>
internal sealed class ShellApplication : IApplication
{
    private readonly CognitosOs.Framework.Kernel.IKernel _kernel;
    private readonly MachineState _machineState;
    private readonly IReadOnlyDictionary<string, IKernelCommand> _commandIndex;
    private readonly ScreenBuffer _screen;
    private readonly ApplicationStack _stack;
    private readonly EasterEggRegistry _eggs;
    private readonly HistoryCommand _historyCmd;

    public ShellApplication(
        CognitosOs.Framework.Kernel.IKernel kernel,
        MachineState machineState,
        IReadOnlyDictionary<string, IKernelCommand> commandIndex,
        ScreenBuffer screen, ApplicationStack stack,
        EasterEggRegistry eggs, HistoryCommand historyCmd)
    {
        _kernel = kernel;
        _machineState = machineState;
        _commandIndex = commandIndex;
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
            var writer = new StringWriter();
            using var eggUow = (CognitosOs.Kernel.IUnitOfWork)_kernel.CreateScope(session, writer, _machineState.Quest);
            var exitCode = _eggs.TryHandle(eggUow, cmd, parts);
            if (exitCode.HasValue)
            {
                session.LastExitCode = exitCode.Value;
                FlushOutput(writer);
                return ApplicationResult.Continue;
            }

            session.LastExitCode = 127;
            _screen.Append(Style.Fg(Style.Error, $"{cmd}: command not found"), "");
            return ApplicationResult.Continue;
        }

        var cmdWriter = new StringWriter();
        using var uow = (CognitosOs.Kernel.IUnitOfWork)_kernel.CreateScope(session, cmdWriter, _machineState.Quest);
        var result = command.Run(uow, parts);
        session.LastExitCode = result;

        switch (result)
        {
            case 901: // clear screen
                _screen.ClearViewport();
                break;
            case 900: // launch FTP app
                FlushOutput(cmdWriter);
                _stack.Push(new FtpApplication(_kernel, _machineState, _screen), session);
                break;
            default:
                FlushOutput(cmdWriter);
                break;
        }

        return ApplicationResult.Continue;
    }

    private void FlushOutput(StringWriter writer)
    {
        writer.Flush();
        var text = writer.ToString();
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
