namespace CognitOS.Framework.Kernel;

using CognitOS.Core;
using CognitOS.Kernel.Clock;
using CognitOS.Kernel.Disk;
using CognitOS.Kernel.Journal;
using CognitOS.Kernel.Mail;
using CognitOS.Kernel.Network;
using CognitOS.Kernel.Process;
using CognitOS.Kernel.Resources;
using CognitOS.State;

/// <summary>
/// Per-command unit of work. Commands write output to Out/Err and
/// access OS subsystems through typed interfaces.
/// Dispose cleans up any forked process.
/// </summary>
internal interface IUnitOfWork : IDisposable
{
    TextWriter Out { get; }
    TextWriter Err { get; }
    IDisk Disk { get; }
    INetwork Net { get; }
    IProcessTable Process { get; }
    IClock Clock { get; }
    IMailSpool Mail { get; }
    IJournal Journal { get; }
    UserSession Session { get; }
    QuestState Quest { get; }
    MachineSpec Spec { get; }
    ResourceSnapshot Resources { get; }
    int? CommandPid { get; set; }
}
