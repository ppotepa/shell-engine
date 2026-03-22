namespace CognitosOs.Kernel.Mail;

using CognitosOs.Kernel.Clock;
using CognitosOs.Kernel.Disk;

/// <summary>
/// Mail spool backed by disk. Reading/delivering mail incurs disk I/O delays.
/// Uses <see cref="State.MailMessage"/> from existing state model.
/// </summary>
internal interface IMailSpool
{
    IReadOnlyList<MailEntry> List();
    MailEntry? Read(int index);
    void Deliver(string from, string to, string subject, string body);
    int UnreadCount();
    void MarkRead(int index);
}

internal sealed class MailEntry
{
    public string From { get; init; } = "";
    public string To { get; init; } = "";
    public string Subject { get; init; } = "";
    public string Body { get; init; } = "";
    public DateTime Date { get; init; }
    public bool IsRead { get; set; }
}

/// <summary>
/// Simulated mail spool. Messages stored in-memory but read/deliver go through IDisk
/// to incur realistic timing.
/// </summary>
internal sealed class SimulatedMailSpool : IMailSpool
{
    private readonly List<MailEntry> _messages = new();
    private readonly IDisk _disk;
    private readonly IClock _clock;
    private readonly string _spoolDir;

    public SimulatedMailSpool(IDisk disk, IClock clock, string user, IEnumerable<MailEntry>? initial = null)
    {
        _disk = disk;
        _clock = clock;
        _spoolDir = $"/var/spool/mail/{user}";

        if (initial is not null)
            _messages.AddRange(initial);
    }

    public IReadOnlyList<MailEntry> List() => _messages.AsReadOnly();

    public MailEntry? Read(int index)
    {
        if (index < 0 || index >= _messages.Count) return null;

        // Simulate reading mail file from spool directory
        string path = $"{_spoolDir}/{index}";
        try { _disk.ReadFile(path); } catch { /* spool file may not exist on disk, timing still applies */ }

        return _messages[index];
    }

    public void Deliver(string from, string to, string subject, string body)
    {
        var msg = new MailEntry
        {
            From = from,
            To = to,
            Subject = subject,
            Body = body,
            Date = _clock.Now(),
        };
        _messages.Add(msg);

        // Simulate writing to spool
        string path = $"{_spoolDir}/{_messages.Count - 1}";
        string content = $"From: {msg.From}\nTo: {msg.To}\nSubject: {msg.Subject}\nDate: {msg.Date}\n\n{msg.Body}";
        try { _disk.WriteFile(path, content); } catch { /* ENOSPC — mail still in memory for gameplay */ }
    }

    public int UnreadCount() => _messages.Count(m => !m.IsRead);

    public void MarkRead(int index)
    {
        if (index >= 0 && index < _messages.Count)
            _messages[index].IsRead = true;
    }
}
