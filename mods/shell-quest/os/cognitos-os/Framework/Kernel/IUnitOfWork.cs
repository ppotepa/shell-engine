namespace CognitosOs.Framework.Kernel;

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
