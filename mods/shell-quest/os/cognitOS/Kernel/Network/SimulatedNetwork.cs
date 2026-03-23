namespace CognitOS.Kernel.Network;

using CognitOS.Framework.Kernel;
using CognitOS.Kernel.Hardware;
using CognitOS.Kernel.Resources;
using CognitOS.Network;

/// <summary>
/// Simulated network operations with realistic timing.
/// Connect/Send/Recv go through <see cref="ISyscallGate"/> for unified latency + resource management.
/// 
/// Per-socket packet queues model bandwidth limitations and RTT so network operations
/// feel realistic — packets arrive over time instead of instantly.
/// </summary>
internal sealed class SimulatedNetwork : INetwork
{
    private readonly RemoteHostIndex _registry;
    private readonly ResourceState _res;
    private readonly HardwareProfile _hw;
    private readonly Disk.IDisk _disk;
    private readonly ISyscallGate _gate;
    private readonly Random _rng = new();

    private readonly Dictionary<int, string> _sockets = new();
    private readonly Dictionary<int, NetworkPacketQueue> _packetQueues = new();

    public SimulatedNetwork(RemoteHostIndex registry, ResourceState res, HardwareProfile hw, Disk.IDisk disk, ISyscallGate gate)
    {
        _registry = registry;
        _res = res;
        _hw = hw;
        _disk = disk;
        _gate = gate;
    }

    public string? Resolve(string hostname)
    {
        // Read /etc/hosts — real disk I/O via disk subsystem
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

        if (_registry.IsKnown(hostname))
        {
            // Simulate DNS query via gate
            _gate.Dispatch(
                SyscallRequest.For(SyscallKind.NetResolve),
                () => { }
            ).ThrowIfFailed();
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

        int fd = _res.Fd.Alloc();

        var result = _gate.Dispatch(
            SyscallRequest.For(SyscallKind.NetConnect),
            () =>
            {
                _sockets[fd] = host;
                // Create packet queue for this socket: RTT ~50ms, bandwidth ~3 bytes/ms (24 Kbps)
                _packetQueues[fd] = new NetworkPacketQueue(host, rttMs: 50, bandwidthBytesPerMs: 3);
            });

        if (!result.Success)
        {
            _res.Fd.Close(fd);
            throw new IOException($"connect: {host}: {result.ErrorCode}");
        }

        return fd;
    }

    public void Send(int socketFd, byte[] data)
    {
        _gate.Dispatch(
            SyscallRequest.For(SyscallKind.NetSend, data.Length),
            () => _res.NetCtrl.RecordSent(data.Length)
        ).ThrowIfFailed();
    }

    public byte[] Receive(int socketFd, int expectedBytes)
    {
        _gate.Dispatch(
            SyscallRequest.For(SyscallKind.NetRecv, expectedBytes),
            () => _res.NetCtrl.RecordReceived(expectedBytes)
        ).ThrowIfFailed();

        return new byte[expectedBytes];
    }

    public PingResult Ping(string host)
    {
        string? ip = Resolve(host);
        if (ip is null)
            return new PingResult(host, null, 0, false, $"ping: {host}: Host not found");

        double jitter = _hw.NetBasePingMs * 0.2 * (_rng.NextDouble() * 2 - 1);
        double rtt = _hw.NetBasePingMs * 2 + jitter;

        // Note: This method is not used by commands (they use EasterEggOutput.SimulatePing).
        // The latency here is informational only and does not block.
        // Real ping output is scheduled via UnitOfWork.ScheduleOutput().

        return new PingResult(host, ip, rtt, true, null);
    }

    public void Close(int socketFd)
    {
        if (_sockets.Remove(socketFd))
        {
            _res.Fd.Close(socketFd);
            _res.NetCtrl.ReleaseBandwidth();
            _packetQueues.Remove(socketFd);
        }
    }

    public bool IsKnownHost(string hostname) => _registry.IsKnown(hostname);

    /// <summary>
    /// Get all packet queues for the kernel event loop to drain.
    /// </summary>
    internal IEnumerable<(int fd, NetworkPacketQueue queue)> GetPacketQueues()
        => _packetQueues.Select(kvp => (kvp.Key, kvp.Value));
}

