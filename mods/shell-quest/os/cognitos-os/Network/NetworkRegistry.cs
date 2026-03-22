using CognitosOs.Core;

namespace CognitosOs.Network;

internal sealed class NetworkRegistry
{
    private readonly Dictionary<string, IExternalServer> _hosts = new(StringComparer.OrdinalIgnoreCase);

    public NetworkRegistry()
    {
        Register(new NormalServer
        {
            Hostname = "nic.funet.fi",
            Aliases = new[] { "ftp.funet.fi" },
            IpAddress = "128.214.6.100",
            BasePingMs = 47,
        });
        Register(new NormalServer
        {
            Hostname = "cs.vu.nl",
            IpAddress = "130.37.24.3",
            BasePingMs = 112,
        });
        Register(new NormalServer
        {
            Hostname = "sun.com",
            IpAddress = "192.9.9.1",
            BasePingMs = 203,
            Type = ServerType.PingOnly,
        });
        Register(new NormalServer
        {
            Hostname = "helsinki.fi",
            IpAddress = "128.214.1.1",
            BasePingMs = 12,
            Type = ServerType.PingOnly,
        });
        Register(new NormalServer
        {
            Hostname = "ftp.uu.net",
            Aliases = new[] { "uunet.uu.net" },
            IpAddress = "192.48.96.9",
            BasePingMs = 189,
        });
        Register(new NormalServer
        {
            Hostname = "mit.edu",
            IpAddress = "18.72.2.1",
            BasePingMs = 231,
            Type = ServerType.PingOnly,
        });
        Register(new NormalServer
        {
            Hostname = "localhost",
            Aliases = new[] { "kruuna" },
            IpAddress = "127.0.0.1",
            BasePingMs = 0,
            Type = ServerType.Loopback,
        });

        // --- Temporal anomalies ---
        Register(new AnomalyServer
        {
            Hostname = "google.com",
            ErrorSequence = new[]
            {
                "PING google.com ... resolving",
                "net: forward lookup failed",
                "net: retrying via alternate root",
                "... no route to host",
                "ping: transmit failed (unreachable) [0xFE]",
                Style.Fg(Style.Warn, "note: unexpected partial route trace logged to /var/log/net.trace"),
            },
        });
        Register(new AnomalyServer
        {
            Hostname = "github.com",
            ErrorSequence = new[]
            {
                "PING github.com ... resolving",
                "net: name resolution returned inconclusive",
                "net: authority record points to unallocated block",
                "... request timed out",
                "ping: host not found, but 3 hops responded (unexpected)",
                Style.Fg(Style.Warn, "note: see /var/log/net.trace"),
            },
        });
        Register(new AnomalyServer
        {
            Hostname = "wikipedia.org",
            ErrorSequence = new[]
            {
                "PING wikipedia.org ... resolving",
                "net: forward lookup: NXDOMAIN",
                "net: anomaly: received partial ICMP echo from unregistered AS",
                "... connection interrupted",
                "ping: unknown network error [0xFF]",
                Style.Fg(Style.Warn, "note: logged to /var/log/net.trace"),
            },
        });
    }

    public IExternalServer? Resolve(string hostname)
    {
        _hosts.TryGetValue(hostname, out var server);
        return server;
    }

    public IReadOnlyDictionary<string, IExternalServer> AllHosts => _hosts;

    /// <summary>
    /// Simulate a ping with jitter scaled by NIC speed.
    /// </summary>
    public static int JitteredPing(int baseMs, MachineSpec spec)
    {
        if (baseMs <= 0) return 0;
        var factor = 1200.0 / spec.NicSpeedKbps;
        var scaled = (int)(baseMs * factor);
        var jitter = Random.Shared.Next(-(scaled / 7), scaled / 7 + 1);
        return Math.Max(1, scaled + jitter);
    }

    private void Register(IExternalServer server)
    {
        _hosts[server.Hostname] = server;
        foreach (var alias in server.Aliases)
            _hosts[alias] = server;
    }
}
