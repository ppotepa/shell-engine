namespace CognitosOs.Network;

internal enum ServerType
{
    Normal,
    PingOnly,
    Anomaly,
    Loopback,
}

internal interface IExternalServer
{
    string Hostname { get; }
    IReadOnlyList<string> Aliases { get; }
    string IpAddress { get; }
    int BasePingMs { get; }
    ServerType Type { get; }
}

internal sealed class NormalServer : IExternalServer
{
    public string Hostname { get; init; } = "";
    public IReadOnlyList<string> Aliases { get; init; } = Array.Empty<string>();
    public string IpAddress { get; init; } = "";
    public int BasePingMs { get; init; }
    public ServerType Type { get; init; } = ServerType.Normal;
}

internal sealed class AnomalyServer : IExternalServer
{
    public string Hostname { get; init; } = "";
    public IReadOnlyList<string> Aliases { get; init; } = Array.Empty<string>();
    public string IpAddress => "";
    public int BasePingMs => 0;
    public ServerType Type => ServerType.Anomaly;

    /// <summary>
    /// Each anomaly has unique error lines that differ from the others.
    /// </summary>
    public IReadOnlyList<string> ErrorSequence { get; init; } = Array.Empty<string>();
}
