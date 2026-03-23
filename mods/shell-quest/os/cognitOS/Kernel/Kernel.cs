namespace CognitOS.Kernel;

using CognitOS.Core;
using CognitOS.Framework.Kernel;
using CognitOS.Kernel.Clock;
using CognitOS.Kernel.Disk;
using CognitOS.Kernel.Events;
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
using CognitOS.Kernel.Modem;
using CognitOS.Minix.Kernel;
using CognitOS.State;
using FrameworkKernel = CognitOS.Framework.Kernel;

/// <summary>
/// Kernel implementation. Composes all subsystems from a <see cref="MachineSpec"/>
/// and provides <see cref="IUnitOfWork"/> scopes for command execution.
/// Implements both <see cref="IKernel"/> (internal) and <see cref="FrameworkKernel.IKernel"/>
/// (framework-facing) so it can be registered in the IoC container as either.
/// </summary>
internal sealed class Kernel : IKernel, FrameworkKernel.IKernel
{
    public IDisk Disk { get; }
    public INetwork Net { get; }
    public IProcessTable Process { get; }
    public IClock Clock { get; }
    public IMailSpool Mail { get; }
    public IJournal Journal { get; }
    public IServiceManager Services { get; }
    public ISessionManager Sessions { get; }
    public IUserDatabase Users { get; }
    public IMountTable Mounts { get; }
    public IModem Modem { get; }
    public ResourceState Resources { get; }
    public HardwareProfile Hardware { get; }
    public MachineSpec Spec { get; }
    public KernelEventQueue Events { get; }

    public Kernel(MachineSpec spec, IMutableFileSystem vfs, CognitOS.Network.RemoteHostIndex hostIndex)
    {
        Spec = spec;

        // Layer 1: Hardware timings
        Hardware = HardwareProfile.FromSpec(spec);

        // Layer 2: Resource pools
        Resources = new ResourceState(spec, Hardware);

        // Layer 3: Clock
        Clock = new SimulatedClock(new DateTime(1991, 9, 17, 21, 12, 0));
        Events = new KernelEventQueue();

        // Layer 3.5: Syscall gate — single choke point for resource checks + latency
        ISyscallGate gate = new MinixSyscallGate(Resources, Hardware, Clock);

        // Layer 4: Subsystems (each wraps storage with timing via gate)
        Disk = new SimulatedDisk(vfs, Resources, Hardware, gate, Clock);
        var processTable = new SimulatedProcessTable(Resources, Hardware, Clock, gate);
        Process = processTable;
        Net = new SimulatedNetwork(hostIndex, Resources, Hardware, Disk, gate);
        Journal = new SimulatedJournal(Disk, Clock);
        Mail = new SimulatedMailSpool(Disk, Clock, "torvalds");

        // Layer 5: Service manager
        Services = new SimulatedServiceManager(Process, Disk, Clock, Mail, Journal);

        // Layer 6: Higher-level OS subsystems
        Modem = new SimulatedModem(Hardware);
        Sessions = new SessionManager();
        Users = new UserDatabase(Disk);
        var mountTable = new MountTable();
        var rootKb = spec.DiskKb / 2;
        mountTable.AddMount(new MountEntry { Device = "/dev/hd1", MountPoint = "/",    FsType = "minix", Options = "rw", SizeKb = rootKb });
        mountTable.AddMount(new MountEntry { Device = "/dev/hd2", MountPoint = "/usr", FsType = "minix", Options = "rw", SizeKb = spec.DiskKb - rootKb });
        Mounts = mountTable;

        // Boot sessions and processes
        BootSystemProcesses(processTable);
        BootSessions((SessionManager)Sessions, new DateTime(1991, 9, 17, 21, 12, 0));
    }

    public IUnitOfWork CreateScope(UserSession session, TextWriter output, QuestState quest)
    {
        return new UnitOfWork(this, session, output, quest);
    }

    /// <summary>Explicit implementation for Framework.Kernel.IKernel.</summary>
    FrameworkKernel.IUnitOfWork FrameworkKernel.IKernel.CreateScope(
        UserSession session, TextWriter output, QuestState quest)
        => CreateScope(session, output, quest);

    public ulong NowMs => Clock.UptimeMs();

    public void Schedule(KernelEventKind kind, ulong delayMs, Action action, string? tag = null)
        => Events.ScheduleAfter(NowMs, delayMs, kind, action, tag);

    public void Tick(ulong dtMs)
    {
        Clock.Advance(dtMs);
        Services.Tick(dtMs);
        Resources.DiskCtrl.UpdateSpindleState(NowMs);
        foreach (var ev in Events.DrainReady(NowMs))
            ev.Action();
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
        pt.AddSystemProcess(new ProcessEntry { Pid = 7, Ppid = 6, Uid = 101, Name = "sh", User = "torvalds", StateCh = 'S', Tty = "tty0", Sz = 16 });

        // Start cron and update services
        Services.Start("cron");
        Services.Start("update");
    }

    private static void BootSessions(SessionManager sm, DateTime epoch)
    {
        // torvalds logged in at epoch on tty0 (the player's session)
        sm.RegisterSession(new TtySession { User = "torvalds", Tty = "tty0", LoginTime = epoch });
        // ast has been logged in on tty1 since Sep 15
        sm.RegisterSession(new TtySession { User = "ast",      Tty = "tty1", LoginTime = new DateTime(1991, 9, 15, 9, 41, 0) });
        // anomaly: unnamed session from epoch zero
        sm.RegisterSession(new TtySession { User = "",         Tty = "tty2", LoginTime = new DateTime(1970, 1, 1, 0, 0, 0), IsAnomaly = true });
    }
}
