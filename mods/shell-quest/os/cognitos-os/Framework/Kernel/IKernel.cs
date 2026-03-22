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
/// The kernel — composes all OS subsystems and creates per-command UoW scopes.
/// Implementations are OS-specific (MinixKernel, LinuxKernel).
/// </summary>
internal interface IKernel
{
    IDisk Disk { get; }
    INetwork Net { get; }
    IProcessTable Process { get; }
    IClock Clock { get; }
    IMailSpool Mail { get; }
    IJournal Journal { get; }
    MachineSpec Spec { get; }
    ResourceState Resources { get; }

    IUnitOfWork CreateScope(UserSession session, TextWriter output, QuestState quest);
    void Tick(ulong dtMs);
}
