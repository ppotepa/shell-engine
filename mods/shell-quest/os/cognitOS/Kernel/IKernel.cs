namespace CognitOS.Kernel;

using CognitOS.Core;
using CognitOS.Kernel.Clock;
using CognitOS.Kernel.Disk;
using CognitOS.Kernel.Journal;
using CognitOS.Kernel.Mail;
using CognitOS.Kernel.Network;
using CognitOS.Kernel.Process;
using CognitOS.Kernel.Resources;
using CognitOS.Kernel.Services;
using CognitOS.Kernel.Hardware;
using CognitOS.Kernel.Session;
using CognitOS.Kernel.Users;
using CognitOS.Kernel.Mount;
using CognitOS.State;

/// <summary>
/// Central kernel — owns all subsystems and resource state.
/// Created once per game session. Provides <see cref="CreateScope"/> to
/// hand out <see cref="IUnitOfWork"/> instances for individual commands/apps.
/// </summary>
internal interface IKernel
{
    IDisk Disk { get; }
    INetwork Net { get; }
    IProcessTable Process { get; }
    IClock Clock { get; }
    IMailSpool Mail { get; }
    IJournal Journal { get; }
    IServiceManager Services { get; }
    ISessionManager Sessions { get; }
    IUserDatabase Users { get; }
    IMountTable Mounts { get; }
    ResourceState Resources { get; }
    HardwareProfile Hardware { get; }
    MachineSpec Spec { get; }

    /// <summary>Create a scoped UoW for a single command/app interaction.</summary>
    IUnitOfWork CreateScope(UserSession session, TextWriter output, QuestState quest);

    /// <summary>Advance kernel tick: clock, services, resource recalc.</summary>
    void Tick(ulong dtMs);
}
