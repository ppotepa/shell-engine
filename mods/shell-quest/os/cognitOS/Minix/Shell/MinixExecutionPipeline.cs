namespace CognitOS.Minix.Shell;

using CognitOS.Applications;
using CognitOS.Commands;
using CognitOS.Core;
using CognitOS.Framework.Execution;
using CognitOS.Kernel;
using CognitOS.Network;
using CognitOS.State;

internal sealed class MinixExecutionPipeline : IExecutionPipeline
{
    private readonly MachineState _machineState;
    private readonly ApplicationStack _stack;
    private readonly IShellBuiltins _builtins;
    private readonly IScriptInterpreter _scripts;
    private readonly IReadOnlyDictionary<string, IKernelCommand> _commandIndex;
    private readonly EasterEggRegistry _eggs;
    private readonly HistoryCommand _historyCmd;

    public MinixExecutionPipeline(
        MachineState machineState,
        ApplicationStack stack,
        IShellBuiltins builtins,
        IScriptInterpreter scripts,
        IReadOnlyDictionary<string, IKernelCommand> commandIndex,
        EasterEggRegistry eggs,
        HistoryCommand historyCmd)
    {
        _machineState = machineState;
        _stack = stack;
        _builtins = builtins;
        _scripts = scripts;
        _commandIndex = commandIndex;
        _eggs = eggs;
        _historyCmd = historyCmd;
    }

    public ApplicationResult Execute(IUnitOfWork uow, string input)
    {
        var submitted = input.Trim();
        if (string.IsNullOrWhiteSpace(submitted))
            return ApplicationResult.Continue;

        _historyCmd.CommandLog.Add(submitted);
        var parts = submitted.Split(' ', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);
        var cmd = parts[0];

        if (parts.Skip(1).Any(a => a == "--help"))
        {
            uow.Session.LastExitCode = 1;
            uow.Out.WriteLine($"{cmd}: illegal option -- -");
            uow.Out.WriteLine($"Try: man {cmd}");
            uow.Out.WriteLine();
            return ApplicationResult.Continue;
        }

        if (_builtins.TryHandle(uow, parts, out var builtinResult))
            return builtinResult;

        if (_commandIndex.TryGetValue(cmd, out var command))
        {
            var exitCode = command.Run(uow, parts);
            uow.Session.LastExitCode = exitCode;
            if (exitCode == 900)
                _stack.Push(new FtpApplication(_machineState), uow.Session);
            return ApplicationResult.Continue;
        }

        var eggExitCode = _eggs.TryHandle(uow, cmd, parts);
        if (eggExitCode.HasValue)
        {
            uow.Session.LastExitCode = eggExitCode.Value;
            return ApplicationResult.Continue;
        }

        if (_scripts.CanExecute(uow, cmd))
        {
            uow.Session.LastExitCode = _scripts.Execute(uow, parts);
            return ApplicationResult.Continue;
        }

        uow.Session.LastExitCode = 127;
        uow.Out.WriteLine(Style.Fg(Style.Error, $"{cmd}: command not found"));
        uow.Out.WriteLine();
        return ApplicationResult.Continue;
    }
}
