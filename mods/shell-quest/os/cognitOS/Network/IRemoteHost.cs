namespace CognitOS.Network;

internal enum HostAccess { Normal, PingOnly, Loopback }

/// <summary>
/// Single source of truth for every host the game knows about —
/// real 1991 servers and temporal anomalies alike.
/// </summary>
internal interface IRemoteHost
{
    string Hostname { get; }
    IReadOnlyList<string> Aliases { get; }
    string IpAddress { get; }
    int BasePingMs { get; }
    HostAccess Access { get; }
}
