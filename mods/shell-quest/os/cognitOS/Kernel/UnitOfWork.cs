namespace CognitOS.Kernel;

using CognitOS.Core;
using CognitOS.Framework.Transport;
using CognitOS.Kernel.Clock;
using CognitOS.Kernel.Disk;
using CognitOS.Kernel.Journal;
using CognitOS.Kernel.Mail;
using CognitOS.Kernel.Modem;
using CognitOS.Kernel.Mount;
using CognitOS.Kernel.Network;
using CognitOS.Kernel.Process;
using CognitOS.Kernel.Resources;
using CognitOS.Kernel.Session;
using CognitOS.Kernel.Users;
using CognitOS.State;

/// <summary>
/// Concrete UoW. Delegates to kernel subsystems.
/// Tracks the command process PID so it can be cleaned up on dispose.
/// </summary>
internal sealed class UnitOfWork : IUnitOfWork
{
    private readonly IKernel _kernel;
    private bool _disposed;
    private ulong _pendingDelayMs;

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
    public ISessionManager Sessions => _kernel.Sessions;
    public IUserDatabase Users => _kernel.Users;
    public IMountTable Mounts => _kernel.Mounts;
    public IModem Modem => _kernel.Modem;
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

    /// <summary>
    /// Emit a line after <paramref name="delayMs"/> simulated milliseconds.
    /// Delays are cumulative so a sequence of calls produces staggered output.
    /// </summary>
    public void ScheduleOutput(string line, ulong delayMs)
    {
        _pendingDelayMs += delayMs;

        var sink = ResolveSink(Out);
        if (sink is not null)
            Protocol.EmitLine(sink, line, _pendingDelayMs > 0 ? _pendingDelayMs : null);
        else
            Out.WriteLine(line);
    }

    private static IOutputSink? ResolveSink(TextWriter writer)
    {
        if (writer is GameTextWriter gameWriter)
            return gameWriter.Sink;
        var field = writer.GetType().GetField(
            "_sink",
            System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Instance);
        return field?.GetValue(writer) as IOutputSink;
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
