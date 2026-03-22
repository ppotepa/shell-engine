namespace CognitOS.Kernel.Resources;

/// <summary>
/// Network bandwidth sharing. Single NIC — bandwidth is split between active connections.
/// </summary>
internal sealed class NetworkController
{
    private readonly double _totalBandwidthKBs;
    private const int MaxConnections = 8; // NE2000 / 3Com EtherLink era socket limit

    public int ActiveConnections { get; private set; }
    public long TotalBytesSent { get; private set; }
    public long TotalBytesReceived { get; private set; }

    public NetworkController(double bandwidthKBs)
    {
        _totalBandwidthKBs = bandwidthKBs;
    }

    /// <summary>
    /// Available bandwidth per connection in KB/s.
    /// Bandwidth is evenly split between active connections.
    /// </summary>
    public double AvailableBandwidthKBs() =>
        _totalBandwidthKBs / Math.Max(1, ActiveConnections);

    /// <summary>Check whether a new connection can be opened.</summary>
    public bool CanConnect() => ActiveConnections < MaxConnections;

    /// <summary>Open a connection — claims bandwidth share.</summary>
    public void ReserveBandwidth() => ActiveConnections++;

    /// <summary>Close a connection — releases bandwidth share.</summary>
    public void ReleaseBandwidth() => ActiveConnections = Math.Max(0, ActiveConnections - 1);

    /// <summary>Acquire called by <see cref="CognitOS.Minix.Kernel.MinixSyscallGate"/> Debit.</summary>
    public void Acquire() => ReserveBandwidth();

    /// <summary>Release called by <see cref="CognitOS.Minix.Kernel.MinixSyscallGate"/> Credit.</summary>
    public void Release() => ReleaseBandwidth();

    /// <summary>Record bytes sent (stats only).</summary>
    public void RecordSent(int bytes) => TotalBytesSent += bytes;

    /// <summary>Record bytes received (stats only).</summary>
    public void RecordReceived(int bytes) => TotalBytesReceived += bytes;
}
