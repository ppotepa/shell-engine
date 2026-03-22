using CognitosOs.Core;
using CognitosOs.Kernel;
using CognitosOs.Network;

namespace CognitosOs.Commands;

[CognitosOs.Framework.Ioc.Command("ping", OsTag = "minix")]
internal sealed class PingCommand : IKernelCommand
{
    private readonly NetworkRegistry _network;

    public PingCommand(NetworkRegistry network) => _network = network;

    public string Name => "ping";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        if (argv.Length < 2)
        {
            uow.Err.WriteLine("usage: ping <host>");
            return 1;
        }

        var host = argv[1];
        var server = _network.Resolve(host);

        if (server is null)
        {
            uow.Err.WriteLine($"ping: unknown host {host}");
            return 1;
        }

        if (server is AnomalyServer anomaly)
            return HandleAnomaly(anomaly, uow);

        return HandleNormal(server, uow);
    }

    private int HandleNormal(IExternalServer server, IUnitOfWork uow)
    {
        if (server.Type == ServerType.Loopback)
        {
            uow.Out.WriteLine($"PING {server.Hostname} ({server.IpAddress}): 56 data bytes");
            for (int i = 0; i < 3; i++)
                uow.Out.WriteLine($"64 bytes from {server.IpAddress}: icmp_seq={i} ttl=64 time=0.01ms");
            uow.Out.WriteLine($"--- {server.Hostname} ping statistics ---");
            uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
            uow.Out.WriteLine("round-trip min/avg/max = 0.01/0.01/0.01 ms");
            return 0;
        }

        var pings = new int[3];
        for (int i = 0; i < 3; i++)
            pings[i] = NetworkRegistry.JitteredPing(server.BasePingMs, uow.Spec);

        var ttl = server.BasePingMs < 50 ? 62 : server.BasePingMs < 150 ? 52 : 44;
        uow.Out.WriteLine($"PING {server.Hostname} ({server.IpAddress}): 56 data bytes");
        for (int i = 0; i < 3; i++)
            uow.Out.WriteLine($"64 bytes from {server.IpAddress}: icmp_seq={i} ttl={ttl} time={pings[i]}ms");
        uow.Out.WriteLine($"--- {server.Hostname} ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine($"round-trip min/avg/max = {pings.Min()}/{pings.Sum() / 3}/{pings.Max()} ms");
        return 0;
    }

    private int HandleAnomaly(AnomalyServer anomaly, IUnitOfWork uow)
    {
        var quest = uow.Quest;

        quest.AnomaliesDiscovered ??= new List<string>();
        if (!quest.AnomaliesDiscovered.Contains(anomaly.Hostname))
            quest.AnomaliesDiscovered.Add(anomaly.Hostname);

        UpdateNetTrace(uow);

        foreach (var line in anomaly.ErrorSequence)
            uow.Out.WriteLine(line);
        return 1;
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

        try { uow.Disk.WriteFile("/var/log/net.trace", string.Join("\n", lines)); }
        catch { /* disk full — silently fail */ }
    }
}
