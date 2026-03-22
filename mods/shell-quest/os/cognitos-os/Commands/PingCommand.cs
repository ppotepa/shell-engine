using CognitosOs.Core;
using CognitosOs.Network;

namespace CognitosOs.Commands;

internal sealed class PingCommand : ICommand
{
    private readonly NetworkRegistry _network;

    public PingCommand(NetworkRegistry network) => _network = network;

    public string Name => "ping";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        if (ctx.Argv.Count < 1)
            return new CommandResult(new[] { "usage: ping <host>" }, 1);

        var host = ctx.Argv[0];
        var server = _network.Resolve(host);

        if (server is null)
            return new CommandResult(new[] { $"ping: unknown host {host}" }, 1);

        if (server is AnomalyServer anomaly)
            return HandleAnomaly(anomaly, ctx);

        return HandleNormal(server, ctx);
    }

    private CommandResult HandleNormal(IExternalServer server, CommandContext ctx)
    {
        var lines = new List<string>();
        var spec = ctx.Os.Spec;

        if (server.Type == ServerType.Loopback)
        {
            lines.Add($"PING {server.Hostname} ({server.IpAddress}): 56 data bytes");
            for (int i = 0; i < 3; i++)
                lines.Add($"64 bytes from {server.IpAddress}: icmp_seq={i} ttl=64 time=0.01ms");
            lines.Add($"--- {server.Hostname} ping statistics ---");
            lines.Add("3 packets transmitted, 3 received, 0% packet loss");
            lines.Add("round-trip min/avg/max = 0.01/0.01/0.01 ms");
            return new CommandResult(lines);
        }

        var pings = new int[3];
        for (int i = 0; i < 3; i++)
            pings[i] = NetworkRegistry.JitteredPing(server.BasePingMs, spec);

        var ttl = server.BasePingMs < 50 ? 62 : server.BasePingMs < 150 ? 52 : 44;
        lines.Add($"PING {server.Hostname} ({server.IpAddress}): 56 data bytes");
        for (int i = 0; i < 3; i++)
            lines.Add($"64 bytes from {server.IpAddress}: icmp_seq={i} ttl={ttl} time={pings[i]}ms");
        lines.Add($"--- {server.Hostname} ping statistics ---");
        lines.Add("3 packets transmitted, 3 received, 0% packet loss");
        lines.Add($"round-trip min/avg/max = {pings.Min()}/{pings.Sum() / 3}/{pings.Max()} ms");
        return new CommandResult(lines);
    }

    private static CommandResult HandleAnomaly(AnomalyServer anomaly, CommandContext ctx)
    {
        var quest = ctx.Os.State.Quest;

        // Track anomaly discovery
        quest.AnomaliesDiscovered ??= new List<string>();
        if (!quest.AnomaliesDiscovered.Contains(anomaly.Hostname))
            quest.AnomaliesDiscovered.Add(anomaly.Hostname);

        // Generate /var/log/net.trace content dynamically
        UpdateNetTrace(ctx);

        return new CommandResult(anomaly.ErrorSequence.ToList(), 1);
    }

    private static void UpdateNetTrace(CommandContext ctx)
    {
        var count = ctx.Os.State.Quest.AnomaliesDiscovered?.Count ?? 0;
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

        if (ctx.Os.FileSystem is State.IMutableFileSystem mfs)
            mfs.TryWrite("var/log/net.trace", string.Join("\n", lines), out _);
    }
}
