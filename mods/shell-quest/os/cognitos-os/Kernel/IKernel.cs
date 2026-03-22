namespace CognitosOs.Kernel;

using CognitosOs.Core;
using CognitosOs.Kernel.Clock;
using CognitosOs.Kernel.Disk;
using CognitosOs.Kernel.Journal;
using CognitosOs.Kernel.Mail;
using CognitosOs.Kernel.Network;
using CognitosOs.Kernel.Process;
using CognitosOs.Kernel.Resources;
using CognitosOs.Kernel.Services;
using CognitosOs.Kernel.Hardware;
using CognitosOs.State;

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
    ResourceState Resources { get; }
    HardwareProfile Hardware { get; }
    MachineSpec Spec { get; }

    /// <summary>Create a scoped UoW for a single command/app interaction.</summary>
    IUnitOfWork CreateScope(UserSession session, TextWriter output, QuestState quest);

    /// <summary>Advance kernel tick: clock, services, resource recalc.</summary>
    void Tick(ulong dtMs);
}
