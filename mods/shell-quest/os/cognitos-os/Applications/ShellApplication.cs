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
internal sealed class ShellApplication : IKernelApplication
{
    private readonly MachineState _machineState;
    private readonly IReadOnlyDictionary<string, IKernelCommand> _commandIndex;
    private readonly ApplicationStack _stack;
    private readonly EasterEggRegistry _eggs;
    private readonly HistoryCommand _historyCmd;

    public ShellApplication(
        MachineState machineState,
        IReadOnlyDictionary<string, IKernelCommand> commandIndex,
        ApplicationStack stack,
        EasterEggRegistry eggs, HistoryCommand historyCmd)
    {
        _machineState = machineState;
        _commandIndex = commandIndex;
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

    public void OnEnter(CognitosOs.Kernel.IUnitOfWork uow) { }
    public void OnExit(CognitosOs.Kernel.IUnitOfWork uow) { }

    public ApplicationResult HandleInput(CognitosOs.Kernel.IUnitOfWork uow, string input)
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
            uow.Session.LastExitCode = 1;
            uow.Out.WriteLine($"{cmd}: illegal option -- -");
            uow.Out.WriteLine($"Try: man {cmd}");
            uow.Out.WriteLine();
            return ApplicationResult.Continue;
        }

        if (!_commandIndex.TryGetValue(cmd, out var command))
        {
            var exitCode = _eggs.TryHandle(uow, cmd, parts);
            if (exitCode.HasValue)
            {
                uow.Session.LastExitCode = exitCode.Value;
                return ApplicationResult.Continue;
            }

            uow.Session.LastExitCode = 127;
            uow.Out.WriteLine(Style.Fg(Style.Error, $"{cmd}: command not found"));
            uow.Out.WriteLine();
            return ApplicationResult.Continue;
        }

        var result = command.Run(uow, parts);
        uow.Session.LastExitCode = result;

        switch (result)
        {
            case 901: // clear screen
                break;
            case 900: // launch FTP app
                _stack.Push(new FtpApplication(_machineState), uow.Session);
                break;
        }

        return ApplicationResult.Continue;
    }
}
