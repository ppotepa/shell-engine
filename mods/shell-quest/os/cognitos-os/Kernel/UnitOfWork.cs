namespace CognitosOs.Kernel;

using CognitosOs.Core;
using CognitosOs.Kernel.Clock;
using CognitosOs.Kernel.Disk;
using CognitosOs.Kernel.Journal;
using CognitosOs.Kernel.Mail;
using CognitosOs.Kernel.Network;
using CognitosOs.Kernel.Process;
using CognitosOs.Kernel.Resources;
using CognitosOs.State;

/// <summary>
/// Concrete UoW. Delegates to kernel subsystems.
/// Tracks the command process PID so it can be cleaned up on dispose.
/// </summary>
internal sealed class UnitOfWork : IUnitOfWork
{
    private readonly IKernel _kernel;
    private bool _disposed;

    /// <summary>PID of the forked command process (set by Shell after fork).</summary>
    public int? CommandPid { get; set; }

    public TextWriter Out { get; }
    public TextWriter Err { get; }
    public IDisk Disk => _kernel.Disk;
    public INetwork Net => _kernel.Net;
    public IProcessTable Process => _kernel.Process;
    public IClock Clock => _kernel.Clock;
    public IMailSpool Mail => _kernel.Mail;
    public IJournal Journal => _kernel.Journal;
    public UserSession Session { get; }
    public QuestState Quest { get; }
    public MachineSpec Spec => _kernel.Spec;
    public ResourceSnapshot Resources => _kernel.Resources.Snapshot();

    public UnitOfWork(IKernel kernel, UserSession session, TextWriter output, QuestState quest)
    {
        _kernel = kernel;
        Session = session;
        Out = output;
        Err = output; // In 1991 MINIX, stderr goes to same terminal
        Quest = quest;
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;

        // If a command process was forked for this scope, ensure it's cleaned up
        if (CommandPid.HasValue)
        {
            var proc = Process.Get(CommandPid.Value);
            if (proc is not null)
                Process.Exit(CommandPid.Value);
        }

        Out.Flush();
    }
}
