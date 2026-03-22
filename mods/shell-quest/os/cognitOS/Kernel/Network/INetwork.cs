namespace CognitOS.Kernel.Network;

/// <summary>
/// Network subsystem. Wraps server registry with RTT, bandwidth, DNS resolution delays.
/// </summary>
internal interface INetwork
{
    /// <summary>Resolve hostname to IP. Reads /etc/hosts (disk delay), falls back to DNS query.</summary>
    string? Resolve(string hostname);

    /// <summary>Open a connection. Delays: resolve + 3×RTT handshake. Returns socket FD.</summary>
    int Connect(string host, int port);

    /// <summary>Send data over socket. Delay = bytes / available bandwidth.</summary>
    void Send(int socketFd, byte[] data);

    /// <summary>Receive data from socket. Delay = RTT + response bytes / bandwidth.</summary>
    byte[] Receive(int socketFd, int expectedBytes);

    /// <summary>ICMP ping. Delay = resolve + 2×RTT + jitter.</summary>
    PingResult Ping(string host);

    /// <summary>Close a network socket.</summary>
    void Close(int socketFd);

    /// <summary>Check if a host is reachable (registry lookup, no delay).</summary>
    bool IsKnownHost(string hostname);
}

internal readonly record struct PingResult(
    string Host,
    string? Ip,
    double RttMs,
    bool Success,
    string? Error
);
