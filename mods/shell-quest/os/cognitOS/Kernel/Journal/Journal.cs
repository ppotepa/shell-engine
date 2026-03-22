namespace CognitOS.Kernel.Journal;

using CognitOS.Kernel.Clock;
using CognitOS.Kernel.Disk;

/// <summary>
/// System journal. Appends to /var/log/messages via <see cref="IDisk"/> — real disk delays.
/// </summary>
internal interface IJournal
{
    /// <summary>Append a log entry. Incurs disk write delay.</summary>
    void Append(string level, string message);

    /// <summary>Read a log file. Incurs disk read delay.</summary>
    string Read(string logFile);

    /// <summary>Read the last N entries from /usr/adm/messages (in-memory, no extra disk cost).</summary>
    IReadOnlyList<string> Recent(int count);
}

/// <summary>
/// Simulated journal backed by disk writes.
/// Keeps an in-memory ring buffer for fast <see cref="Recent"/> access.
/// </summary>
internal sealed class SimulatedJournal : IJournal
{
    private readonly IDisk _disk;
    private readonly IClock _clock;
    private readonly List<string> _entries = new();
    private const string LogPath = "/usr/adm/messages";

    public SimulatedJournal(IDisk disk, IClock clock)
    {
        _disk = disk;
        _clock = clock;
    }

    public void Append(string level, string message)
    {
        var now = _clock.Now();
        string entry = $"{now:MMM dd HH:mm:ss} kruuna {level}: {message}";
        _entries.Add(entry);

        // Append to disk log — real disk I/O delay
        try { _disk.AppendFile(LogPath, entry + "\n"); }
        catch { /* ENOSPC — log entry still in memory */ }
    }

    public string Read(string logFile)
    {
        try { return _disk.ReadFile(logFile); }
        catch { return ""; }
    }

    public IReadOnlyList<string> Recent(int count)
    {
        int start = Math.Max(0, _entries.Count - count);
        return _entries.GetRange(start, _entries.Count - start).AsReadOnly();
    }
}
