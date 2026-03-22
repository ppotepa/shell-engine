namespace CognitOS.Kernel.Session;

internal sealed class SessionManager : ISessionManager
{
    private readonly List<TtySession> _sessions = new();

    public IReadOnlyList<TtySession> GetSessions() => _sessions;

    public void RegisterSession(TtySession session)
    {
        _sessions.RemoveAll(s => s.Tty == session.Tty);
        _sessions.Add(session);
        _sessions.Sort((a, b) => string.Compare(a.Tty, b.Tty, StringComparison.Ordinal));
    }

    public TtySession? GetSession(string tty)
        => _sessions.Find(s => s.Tty == tty);
}
