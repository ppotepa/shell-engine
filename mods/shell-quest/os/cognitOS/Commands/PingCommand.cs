using CognitOS.Core;
using CognitOS.Kernel;
using CognitOS.Network;
using System.Linq;

namespace CognitOS.Commands;

[CognitOS.Framework.Ioc.Command("ping", OsTag = "minix")]
internal sealed class PingCommand : IKernelCommand
{
    private readonly RemoteHostIndex _index;

    public PingCommand(RemoteHostIndex index) => _index = index;

    public string Name => "ping";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        if (argv.Length < 2)
        {
            uow.Err.WriteLine("usage: ping <host>");
            return 1;
        }

        var hostname = argv[1];
        var host = _index.Resolve(hostname);

        if (host is null)
        {
            uow.Err.WriteLine($"ping: unknown host {hostname}");
            return 1;
        }

        if (host is IEasterEgg egg)
        {
            TrackAnomaly(hostname, uow);
            egg.Execute(uow);
            return 1;
        }

        return HandleNormal(host, uow);
    }

    private int HandleNormal(IRemoteHost host, IUnitOfWork uow)
    {
        if (host.Access == HostAccess.Loopback)
        {
            uow.ScheduleOutput($"PING {host.Hostname} ({host.IpAddress}): 56 data bytes", 0);
            for (int i = 0; i < 3; i++)
                uow.ScheduleOutput($"64 bytes from {host.IpAddress}: icmp_seq={i} ttl=64 time=0.01ms", 250);
            uow.ScheduleOutput($"--- {host.Hostname} ping statistics ---", 150);
            uow.ScheduleOutput("3 packets transmitted, 3 received, 0% packet loss", 0);
            uow.ScheduleOutput("round-trip min/avg/max = 0.01/0.01/0.01 ms", 0);
            return 0;
        }

        var pings = new int[3];
        for (int i = 0; i < 3; i++)
            pings[i] = RemoteHostIndex.JitteredPing(host.BasePingMs, uow.Spec);

        var ttl = host.BasePingMs < 50 ? 62 : host.BasePingMs < 150 ? 52 : 44;

        uow.ScheduleOutput($"PING {host.Hostname} ({host.IpAddress}): 56 data bytes", 0);
        for (int i = 0; i < 3; i++)
            uow.ScheduleOutput($"64 bytes from {host.IpAddress}: icmp_seq={i} ttl={ttl} time={pings[i]}ms", 250);
        uow.ScheduleOutput($"--- {host.Hostname} ping statistics ---", 150);
        uow.ScheduleOutput("3 packets transmitted, 3 received, 0% packet loss", 0);
        uow.ScheduleOutput($"round-trip min/avg/max = {pings.Min()}/{pings.Sum() / 3}/{pings.Max()} ms", 0);
        return 0;
    }

    private void TrackAnomaly(string hostname, IUnitOfWork uow)
    {
        var quest = uow.Quest;
        quest.AnomaliesDiscovered ??= new List<string>();
        if (!quest.AnomaliesDiscovered.Contains(hostname))
            quest.AnomaliesDiscovered.Add(hostname);

        UpdateNetTrace(uow);
    }

    private static void UpdateNetTrace(IUnitOfWork uow)
    {
        var count = uow.Quest.AnomaliesDiscovered?.Count ?? 0;
        if (count == 0) return;

        var lines = new List<string>();
        if (count == 1)
        {
            lines.Add("[warn] unresolvable host returned partial route data");
            lines.Add("[warn] destination network not yet allocated by IANA");
        }
        else if (count == 2)
        {
            lines.Add($"[warn] {count} unresolvable hosts returned partial route data");
            lines.Add("[warn] destination networks not yet allocated by IANA");
            lines.Add("[warn] route fragments suggest future allocation");
        }
        else
        {
            lines.Add($"[warn] {count} unresolvable hosts returned partial route data");
            lines.Add("[warn] destination networks not yet allocated by IANA");
            lines.Add("[warn] temporal inconsistency detected in routing tables");
            lines.Add("[    ] ...this shouldn't happen.");
        }

        try { uow.Disk.WriteFile("/usr/adm/net.trace", string.Join("\n", lines)); }
        catch { /* disk full — silently fail */ }
    }
}
