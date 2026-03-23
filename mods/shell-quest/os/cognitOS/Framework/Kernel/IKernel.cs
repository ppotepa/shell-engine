namespace CognitOS.Framework.Kernel;

using CognitOS.Core;
using CognitOS.Kernel.Clock;
using CognitOS.Kernel.Disk;
using CognitOS.Kernel.Journal;
using CognitOS.Kernel.Mail;
using CognitOS.Kernel.Network;
using CognitOS.Kernel.Process;
using CognitOS.Kernel.Resources;
using CognitOS.Kernel.Users;
using CognitOS.State;

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
    IUserDatabase Users { get; }
    MachineSpec Spec { get; }
    ResourceState Resources { get; }

    IUnitOfWork CreateScope(UserSession session, TextWriter output, QuestState quest);
    void Tick(ulong dtMs);
}
