namespace CognitOS.Kernel.Network;

using System.Collections.Generic;

/// <summary>
/// Per-socket network packet queue with bandwidth-based scheduling.
///
/// Packets are assigned a `ScheduledAtMs` timestamp based on:
/// - RTT to the remote host
/// - Packet size and available bandwidth
/// - Cumulative transfer time (each packet adds delay for the next)
///
/// The kernel event queue will check this queue each tick and emit
/// packet-ready events when `ScheduledAtMs <= currentTimeMs`.
/// </summary>
internal sealed class NetworkPacketQueue
{
    private readonly Queue<NetworkPacket> _packets = new();
    private readonly string _remoteHost;
    private readonly int _rttMs;
    private readonly int _bandwidthBytesPerMs;
    private ulong _lastScheduledAtMs;

    public NetworkPacketQueue(string remoteHost, int rttMs, int bandwidthBytesPerMs = 3)
    {
        _remoteHost = remoteHost;
        _rttMs = rttMs;
        _bandwidthBytesPerMs = bandwidthBytesPerMs;
        _lastScheduledAtMs = 0;
    }

    /// <summary>
    /// Schedule a response packet for delivery. Assigns a timestamp based on cumulative delay.
    /// </summary>
    public void EnqueueResponse(byte[] data, ulong nowMs)
    {
        ulong rttMs = (ulong)_rttMs;
        ulong transferTimeMs = (ulong)((data.Length + _bandwidthBytesPerMs - 1) / _bandwidthBytesPerMs);
        
        // Packet arrives: RTT to remote + transfer time + RTT back
        ulong scheduledAtMs = nowMs + rttMs + transferTimeMs + rttMs;
        
        if (_packets.Count == 0)
            _lastScheduledAtMs = scheduledAtMs;
        else
            // Cumulative: each packet adds its transfer time to the last
            scheduledAtMs = _lastScheduledAtMs + transferTimeMs;

        _packets.Enqueue(new NetworkPacket(data, scheduledAtMs));
        _lastScheduledAtMs = scheduledAtMs;
    }

    /// <summary>
    /// Get packets ready for delivery at or before the given time, in order.
    /// </summary>
    public List<NetworkPacket> DrainReady(ulong nowMs)
    {
        var ready = new List<NetworkPacket>();
        while (_packets.Count > 0 && _packets.Peek().ScheduledAtMs <= nowMs)
        {
            ready.Add(_packets.Dequeue());
        }
        return ready;
    }

    /// <summary>
    /// Peek at the next packet without removing it (for debugging/diagnostics).
    /// </summary>
    public NetworkPacket? Peek => _packets.Count > 0 ? _packets.Peek() : null;

    /// <summary>
    /// Number of packets waiting to be delivered.
    /// </summary>
    public int PendingCount => _packets.Count;
}

internal readonly record struct NetworkPacket(
    byte[] Data,
    ulong ScheduledAtMs
);
