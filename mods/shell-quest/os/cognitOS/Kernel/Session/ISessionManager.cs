namespace CognitOS.Kernel.Session;

/// <summary>
/// A single logged-in TTY session. Immutable after creation.
/// </summary>
internal sealed class TtySession
{
    public string User { get; init; } = "";
    public string Tty { get; init; } = "";
    public DateTime LoginTime { get; init; }
    public bool IsAnomaly { get; init; }
}

/// <summary>
/// Tracks all active login sessions across TTYs.
/// Commands (who, finger, netstat) query this — no hardcoding.
/// </summary>
internal interface ISessionManager
{
    IReadOnlyList<TtySession> GetSessions();
    void RegisterSession(TtySession session);
    TtySession? GetSession(string tty);
}
