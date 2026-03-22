namespace CognitosOs.Kernel.Services;

using CognitosOs.Kernel.Clock;
using CognitosOs.Kernel.Disk;
using CognitosOs.Kernel.Journal;
using CognitosOs.Kernel.Mail;
using CognitosOs.Kernel.Process;

/// <summary>
/// Cooperative service manager. Each <see cref="Tick"/> call allows services
/// to perform at most one operation each — no threads, single-core i386 style.
/// </summary>
internal interface IServiceManager
{
    void Start(string name);
    void Stop(string name);
    ServiceEntry? Status(string name);
    IReadOnlyList<ServiceEntry> List();

    /// <summary>
    /// Advance service state machines. Called once per kernel tick.
    /// Each service may perform ≤1 I/O operation.
    /// </summary>
    void Tick(ulong elapsedMs);
}

internal sealed class ServiceEntry
{
    public string Name { get; init; } = "";
    public bool Running { get; set; }
    public int? Pid { get; set; }
    public ulong LastActionMs { get; set; }
    public string LastAction { get; set; } = "";
}

/// <summary>
/// Simulated MINIX system services. Cooperative: each Tick, active services
/// check their schedule and perform one operation through real subsystems.
/// </summary>
internal sealed class SimulatedServiceManager : IServiceManager
{
    private readonly IProcessTable _proc;
    private readonly IDisk _disk;
    private readonly IClock _clock;
    private readonly IMailSpool _mail;
    private readonly IJournal _journal;
    private readonly Dictionary<string, ServiceEntry> _services = new();
    private readonly Dictionary<string, ulong> _nextSchedule = new();

    // Service intervals (ms)
    private const ulong CronIntervalMs = 60_000;
    private const ulong UpdateIntervalMs = 30_000;
    private const ulong MaildIntervalMs = 120_000;

    public SimulatedServiceManager(
        IProcessTable proc, IDisk disk, IClock clock,
        IMailSpool mail, IJournal journal)
    {
        _proc = proc;
        _disk = disk;
        _clock = clock;
        _mail = mail;
        _journal = journal;

        // Default services
        RegisterService("cron");
        RegisterService("update");
    }

    private void RegisterService(string name)
    {
        _services[name] = new ServiceEntry { Name = name, Running = false };
        _nextSchedule[name] = 0;
    }

    public void Start(string name)
    {
        if (!_services.TryGetValue(name, out var svc))
        {
            RegisterService(name);
            svc = _services[name];
        }

        if (svc.Running) return;

        try
        {
            int sz = SimulatedProcessTable.GetBinarySize(name);
            svc.Pid = _proc.Fork(name, sz, "root", "?");
            svc.Running = true;
            _journal.Append("init", $"Starting {name}");
        }
        catch
        {
            // Not enough resources to start service
        }
    }

    public void Stop(string name)
    {
        if (!_services.TryGetValue(name, out var svc) || !svc.Running) return;

        if (svc.Pid.HasValue)
        {
            _proc.Kill(svc.Pid.Value, 9);
            svc.Pid = null;
        }

        svc.Running = false;
        _journal.Append("init", $"Stopped {name}");
    }

    public ServiceEntry? Status(string name) =>
        _services.GetValueOrDefault(name);

    public IReadOnlyList<ServiceEntry> List() =>
        _services.Values.ToList().AsReadOnly();

    public void Tick(ulong elapsedMs)
    {
        foreach (var (name, svc) in _services)
        {
            if (!svc.Running) continue;

            ulong now = _clock.UptimeMs();
            if (!_nextSchedule.TryGetValue(name, out ulong nextAt) || now < nextAt)
                continue;

            // Execute one scheduled operation
            switch (name)
            {
                case "cron":
                    TickCron(svc, now);
                    _nextSchedule[name] = now + CronIntervalMs;
                    break;

                case "update":
                    TickUpdate(svc, now);
                    _nextSchedule[name] = now + UpdateIntervalMs;
                    break;

                case "maild":
                    TickMaild(svc, now);
                    _nextSchedule[name] = now + MaildIntervalMs;
                    break;
            }
        }
    }

    private void TickCron(ServiceEntry svc, ulong now)
    {
        _journal.Append("cron", "/usr/lib/atrun");
        svc.LastActionMs = now;
        svc.LastAction = "atrun";
    }

    private void TickUpdate(ServiceEntry svc, ulong now)
    {
        // sync — flush buffers to disk
        try { _disk.WriteFile("/var/log/.sync", ""); }
        catch { /* ignore ENOSPC for sync marker */ }
        svc.LastActionMs = now;
        svc.LastAction = "sync";
    }

    private void TickMaild(ServiceEntry svc, ulong now)
    {
        // Check mail queue — no new mail to deliver in normal operation
        svc.LastActionMs = now;
        svc.LastAction = "check queue";
    }
}
