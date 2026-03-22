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
/// Kernel implementation. Composes all subsystems from a <see cref="MachineSpec"/>
/// and provides <see cref="IUnitOfWork"/> scopes for command execution.
/// </summary>
internal sealed class Kernel : IKernel
{
    public IDisk Disk { get; }
    public INetwork Net { get; }
    public IProcessTable Process { get; }
    public IClock Clock { get; }
    public IMailSpool Mail { get; }
    public IJournal Journal { get; }
    public IServiceManager Services { get; }
    public ResourceState Resources { get; }
    public HardwareProfile Hardware { get; }
    public MachineSpec Spec { get; }

    public Kernel(MachineSpec spec, IMutableFileSystem vfs, CognitosOs.Network.NetworkRegistry netReg)
    {
        Spec = spec;

        // Layer 1: Hardware timings
        Hardware = HardwareProfile.FromSpec(spec);

        // Layer 2: Resource pools
        Resources = new ResourceState(spec, Hardware);

        // Layer 3: Clock
        Clock = new SimulatedClock(new DateTime(1991, 9, 17, 21, 12, 0));

        // Layer 4: Subsystems (each wraps storage with timing via Resources + Hardware)
        Disk = new SimulatedDisk(vfs, Resources, Hardware);
        var processTable = new SimulatedProcessTable(Resources, Hardware, Clock);
        Process = processTable;
        Net = new SimulatedNetwork(netReg, Resources, Hardware, Disk);
        Journal = new SimulatedJournal(Disk, Clock);
        Mail = new SimulatedMailSpool(Disk, Clock, "linus");

        // Layer 5: Service manager
        Services = new SimulatedServiceManager(Process, Disk, Clock, Mail, Journal);

        // Boot system processes (no fork delay — loaded by bootloader)
        BootSystemProcesses(processTable);
    }

    public IUnitOfWork CreateScope(UserSession session, TextWriter output, QuestState quest)
    {
        return new UnitOfWork(this, session, output, quest);
    }

    public void Tick(ulong dtMs)
    {
        Clock.Advance(dtMs);
        Services.Tick(dtMs);
        Resources.Recalc();
    }

    private void BootSystemProcesses(SimulatedProcessTable pt)
    {
        pt.AddSystemProcess(new ProcessEntry { Pid = 0, Ppid = 0, Uid = 0, Name = "kernel", User = "root", StateCh = 'S', Tty = "?", Sz = 32 });
        pt.AddSystemProcess(new ProcessEntry { Pid = 1, Ppid = 0, Uid = 0, Name = "init", User = "root", StateCh = 'S', Tty = "?", Sz = 8 });
        pt.AddSystemProcess(new ProcessEntry { Pid = 2, Ppid = 0, Uid = 0, Name = "mm", User = "root", StateCh = 'S', Tty = "?", Sz = 24 });
        pt.AddSystemProcess(new ProcessEntry { Pid = 3, Ppid = 0, Uid = 0, Name = "fs", User = "root", StateCh = 'S', Tty = "?", Sz = 48 });
        pt.AddSystemProcess(new ProcessEntry { Pid = 4, Ppid = 1, Uid = 0, Name = "update", User = "root", StateCh = 'S', Tty = "?", Sz = 4 });
        pt.AddSystemProcess(new ProcessEntry { Pid = 5, Ppid = 1, Uid = 0, Name = "cron", User = "root", StateCh = 'S', Tty = "?", Sz = 8 });
        pt.AddSystemProcess(new ProcessEntry { Pid = 6, Ppid = 1, Uid = 0, Name = "getty", User = "root", StateCh = 'S', Tty = "tty0", Sz = 8 });
        pt.AddSystemProcess(new ProcessEntry { Pid = 7, Ppid = 6, Uid = 101, Name = "sh", User = "linus", StateCh = 'S', Tty = "tty0", Sz = 16 });

        // Start cron and update services
        Services.Start("cron");
        Services.Start("update");
    }
}
