namespace CognitosOs.Kernel.Network;

using CognitosOs.Kernel.Hardware;
using CognitosOs.Kernel.Resources;
using CognitosOs.Network;

/// <summary>
/// Simulated network operations with realistic timing.
/// DNS resolution reads /etc/hosts from disk. Connections incur TCP handshake delay.
/// Bandwidth is shared between active connections via <see cref="NetworkController"/>.
/// </summary>
internal sealed class SimulatedNetwork : INetwork
{
    private readonly NetworkRegistry _registry;
    private readonly ResourceState _res;
    private readonly HardwareProfile _hw;
    private readonly Disk.IDisk _disk;
    private readonly Random _rng = new();

    // Track socket → host for bandwidth/RTT lookup
    private readonly Dictionary<int, string> _sockets = new();

    public SimulatedNetwork(NetworkRegistry registry, ResourceState res, HardwareProfile hw, Disk.IDisk disk)
    {
        _registry = registry;
        _res = res;
        _hw = hw;
        _disk = disk;
    }

    public string? Resolve(string hostname)
    {
        // Read /etc/hosts — real disk I/O!
        string? hosts = _disk.RawRead("/etc/hosts");
        if (hosts is not null)
        {
            foreach (var line in hosts.Split('\n'))
            {
                var trimmed = line.Trim();
                if (trimmed.Length == 0 || trimmed[0] == '#') continue;
                var parts = trimmed.Split(new[] { ' ', '\t' }, StringSplitOptions.RemoveEmptyEntries);
                if (parts.Length >= 2)
                {
                    for (int i = 1; i < parts.Length; i++)
                    {
                        if (string.Equals(parts[i], hostname, StringComparison.OrdinalIgnoreCase))
                            return parts[0];
                    }
                }
            }
        }

        // Not in hosts — check if registry knows this host at all
        if (_registry.IsKnown(hostname))
        {
            // Simulate DNS query (no real DNS, but delay as if querying nameserver)
            _hw.BlockFor(_hw.NetBasePingMs * 2);
            return _registry.ResolveIp(hostname);
        }

        return null;
    }

    public int Connect(string host, int port)
    {
        string? ip = Resolve(host);
        if (ip is null)
            throw new IOException($"connect: {host}: Host not found");

        if (!_registry.IsKnown(host))
            throw new IOException($"connect: {host}: Connection refused");

        // Allocate socket FD
        int fd = _res.Fd.Alloc();
        _res.NetCtrl.ReserveBandwidth();

        // TCP 3-way handshake
        _hw.BlockFor(_hw.NetBasePingMs * 3 + _res.Cpu.OverheadMs());

        _sockets[fd] = host;
        return fd;
    }

    public void Send(int socketFd, byte[] data)
    {
        double bw = _res.NetCtrl.AvailableBandwidthKBs();
        double sizeKb = data.Length / 1024.0;
        double transferMs = sizeKb / bw * 1000.0;

        _hw.BlockFor(transferMs + _res.Cpu.OverheadMs());
        _res.NetCtrl.RecordSent(data.Length);
    }

    public byte[] Receive(int socketFd, int expectedBytes)
    {
        double bw = _res.NetCtrl.AvailableBandwidthKBs();
        double sizeKb = expectedBytes / 1024.0;
        double transferMs = sizeKb / bw * 1000.0;

        // RTT for request + transfer time for response
        _hw.BlockFor(_hw.NetBasePingMs + transferMs + _res.Cpu.OverheadMs());
        _res.NetCtrl.RecordReceived(expectedBytes);

        return new byte[expectedBytes]; // placeholder — actual data comes from server simulation
    }

    public PingResult Ping(string host)
    {
        string? ip = Resolve(host);
        if (ip is null)
            return new PingResult(host, null, 0, false, $"ping: {host}: Host not found");

        // 2×RTT + random jitter (±20%)
        double jitter = _hw.NetBasePingMs * 0.2 * (_rng.NextDouble() * 2 - 1);
        double rtt = _hw.NetBasePingMs * 2 + jitter;

        _hw.BlockFor(rtt + _res.Cpu.OverheadMs());

        return new PingResult(host, ip, rtt, true, null);
    }

    public void Close(int socketFd)
    {
        if (_sockets.Remove(socketFd))
        {
            _res.Fd.Close(socketFd);
            _res.NetCtrl.ReleaseBandwidth();
        }
    }

    public bool IsKnownHost(string hostname) => _registry.IsKnown(hostname);
}
