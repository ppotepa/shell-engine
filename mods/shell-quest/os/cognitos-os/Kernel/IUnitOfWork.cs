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
/// Scoped unit-of-work for a single command or app interaction.
/// Provides access to all kernel subsystems via a clean API.
/// Tracks side effects and cleans up on disposal.
/// </summary>
internal interface IUnitOfWork : IDisposable
{
    /// <summary>Standard output — write command results here.</summary>
    TextWriter Out { get; }

    /// <summary>Standard error.</summary>
    TextWriter Err { get; }

    /// <summary>Disk subsystem (read/write with timing + cache).</summary>
    IDisk Disk { get; }

    /// <summary>Network subsystem (connect/send/receive with bandwidth).</summary>
    INetwork Net { get; }

    /// <summary>Process table (fork/exec/exit with RAM accounting).</summary>
    IProcessTable Process { get; }

    /// <summary>Simulated clock.</summary>
    IClock Clock { get; }

    /// <summary>Mail spool (read/deliver with disk I/O).</summary>
    IMailSpool Mail { get; }

    /// <summary>System journal (/var/log).</summary>
    IJournal Journal { get; }

    /// <summary>Current user session.</summary>
    UserSession Session { get; }

    /// <summary>Quest state for game progression.</summary>
    QuestState Quest { get; }

    /// <summary>Hardware specification (read-only).</summary>
    MachineSpec Spec { get; }

    /// <summary>Resource state snapshot (read-only stats).</summary>
    ResourceSnapshot Resources { get; }
}
