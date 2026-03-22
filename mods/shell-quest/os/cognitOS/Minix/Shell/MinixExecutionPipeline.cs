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

        // Parse into one or more simple commands (separated by ';')
        var commands = ShellTokenizer.Parse(submitted, uow.Session);
        foreach (var simple in commands)
        {
            var result = RunSimple(uow, simple);
            if (result != ApplicationResult.Continue)
                return result;
        }
        return ApplicationResult.Continue;
    }

    private ApplicationResult RunSimple(IUnitOfWork uow, SimpleCommand simple)
    {
        if (simple.Tokens.Length == 0) return ApplicationResult.Continue;

        // Handle pipe: run left side into a StringWriter, feed its output to right side
        if (simple.HasPipe && simple.PipeTo is not null)
        {
            var pipeBuffer = new System.IO.StringWriter();
            var pipeUow = new PipedUnitOfWork(uow, pipeBuffer);
            RunTokens(pipeUow, simple.Tokens, null, false);

            var pipeInput = pipeBuffer.ToString().Trim();
            // Feed piped output as stdin substitute: right side gets it as an implicit arg (cat-style)
            // For commands like grep that accept file or stdin, we write to a temp VFS path and pass it
            var tempPath = "/tmp/.pipe.tmp";
            uow.Disk.WriteFile(tempPath, pipeInput);
            var rightTokens = simple.PipeTo.Tokens.Append(tempPath).ToArray();
            RunTokens(uow, rightTokens, simple.PipeTo.RedirectFile, simple.PipeTo.RedirectAppend);
            uow.Disk.Unlink(tempPath);
            return ApplicationResult.Continue;
        }

        RunTokens(uow, simple.Tokens, simple.RedirectFile, simple.RedirectAppend);
        return ApplicationResult.Continue;
    }

    private void RunTokens(IUnitOfWork uow, string[] parts, string? redirectFile, bool redirectAppend)
    {
        if (parts.Length == 0) return;

        var cmd = parts[0];

        if (parts.Skip(1).Any(a => a == "--help"))
        {
            uow.Session.LastExitCode = 1;
            uow.Out.WriteLine($"{cmd}: illegal option -- -");
            uow.Out.WriteLine($"Try: man {cmd}");
            uow.Out.WriteLine();
            return;
        }

        // Resolve output redirect
        System.IO.TextWriter? redirectWriter = null;
        IUnitOfWork activeUow = uow;

        if (redirectFile is not null)
        {
            var resolvedPath = uow.Session.ResolvePath(redirectFile);
            var existing = redirectAppend ? (uow.Disk.Exists(resolvedPath) ? uow.Disk.RawRead(resolvedPath) ?? "" : "") : "";
            var sw = new RedirectWriter(uow, resolvedPath, existing, redirectAppend);
            redirectWriter = sw;
            activeUow = new PipedUnitOfWork(uow, sw);
        }

        try
        {
            if (_builtins.TryHandle(activeUow, parts, out var builtinResult))
            {
                redirectWriter?.Flush();
                return;
            }

            if (_commandIndex.TryGetValue(cmd, out var command))
            {
                var exitCode = command.Run(activeUow, parts);
                activeUow.Session.LastExitCode = exitCode;
                if (exitCode == 900)
                    _stack.Push(new FtpApplication(_machineState), uow.Session);
            }
            else
            {
                var eggExitCode = _eggs.TryHandle(activeUow, cmd, parts);
                if (eggExitCode.HasValue)
                {
                    activeUow.Session.LastExitCode = eggExitCode.Value;
                }
                else if (_scripts.CanExecute(activeUow, cmd))
                {
                    activeUow.Session.LastExitCode = _scripts.Execute(activeUow, parts);
                }
                else
                {
                    activeUow.Session.LastExitCode = 127;
                    uow.Out.WriteLine(Style.Fg(Style.Error, $"{cmd}: command not found"));
                    uow.Out.WriteLine();
                }
            }
        }
        finally
        {
            redirectWriter?.Flush();
        }
    }

    /// <summary>
    /// Lightweight UoW wrapper that redirects Out to a different writer.
    /// Used for pipe and redirect implementations.
    /// </summary>
    private sealed class PipedUnitOfWork : IUnitOfWork
    {
        private readonly IUnitOfWork _inner;
        public TextWriter Out { get; }
        public TextWriter Err => _inner.Err;
        public CognitOS.Kernel.Disk.IDisk Disk => _inner.Disk;
        public CognitOS.Kernel.Network.INetwork Net => _inner.Net;
        public CognitOS.Kernel.Process.IProcessTable Process => _inner.Process;
        public CognitOS.Kernel.Clock.IClock Clock => _inner.Clock;
        public CognitOS.Kernel.Mail.IMailSpool Mail => _inner.Mail;
        public CognitOS.Kernel.Journal.IJournal Journal => _inner.Journal;
        public CognitOS.Kernel.Session.ISessionManager Sessions => _inner.Sessions;
        public CognitOS.Kernel.Users.IUserDatabase Users => _inner.Users;
        public CognitOS.Kernel.Mount.IMountTable Mounts => _inner.Mounts;
        public UserSession Session => _inner.Session;
        public QuestState Quest => _inner.Quest;
        public MachineSpec Spec => _inner.Spec;
        public CognitOS.Kernel.Resources.ResourceSnapshot Resources => _inner.Resources;
        public int? CommandPid { get => _inner.CommandPid; set => _inner.CommandPid = value; }

        public PipedUnitOfWork(IUnitOfWork inner, TextWriter redirected)
        {
            _inner = inner;
            Out = redirected;
        }

        public void Dispose() => _inner.Dispose();
    }

    /// <summary>
    /// TextWriter that collects output and on Flush writes it to VFS (redirect).
    /// </summary>
    private sealed class RedirectWriter : System.IO.StringWriter
    {
        private readonly IUnitOfWork _uow;
        private readonly string _path;
        private readonly string _prefix;

        public RedirectWriter(IUnitOfWork uow, string path, string prefix, bool append)
        {
            _uow = uow;
            _path = path;
            _prefix = append ? prefix : "";
        }

        public override void Flush()
        {
            base.Flush();
            _uow.Disk.WriteFile(_path, _prefix + ToString());
        }
    }
}
