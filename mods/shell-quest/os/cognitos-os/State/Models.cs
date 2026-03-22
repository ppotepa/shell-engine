using CognitosOs.Core;

namespace CognitosOs.State;

internal enum SessionMode
{
    Booting,
    LoginUser,
    LoginPassword,
    Shell,
}

internal sealed class MachineState
{
    public string? UserName { get; set; }
    public string? Password { get; set; }
    public DateTime? LastLogin { get; set; }
    public ulong UptimeMs { get; set; }
    public SessionMode Mode { get; set; } = SessionMode.Booting;
    public string PendingLoginUser { get; set; } = "";
    public List<ProcessEntry> Processes { get; set; } = new();
    public List<ServiceEntry> Services { get; set; } = new();
    public List<MailMessage> MailMessages { get; set; } = new();
    public int UnreadMailCount { get; set; } = 1;

    /// <summary>Hardware spec derived from difficulty. Set once at hello.</summary>
    public MachineSpec Spec { get; set; } = MachineSpec.FromDifficulty(Difficulty.ICanExitVim);

    /// <summary>Prologue quest tracking.</summary>
    public QuestState Quest { get; set; } = new();

    public bool HasAccount => !string.IsNullOrWhiteSpace(UserName) && !string.IsNullOrWhiteSpace(Password);
}

/// <summary>
/// Tracks prologue quest progress. Extensible for future quests.
/// </summary>
internal sealed class QuestState
{
    public string FtpTransferMode { get; set; } = "ascii";
    public bool UploadAttempted { get; set; }
    public bool BackupMade { get; set; }
    public bool UploadSuccess { get; set; }
    public string? FtpRemoteHost { get; set; }
    public bool FtpConnected { get; set; }

    /// <summary>Hostnames of temporal anomalies the player has pinged.</summary>
    public List<string>? AnomaliesDiscovered { get; set; }
}

internal sealed class ServiceEntry
{
    public string Name { get; set; } = "";
    public string Status { get; set; } = "active";
    public DateTime LastTickUtc { get; set; } = DateTime.UtcNow;
}

internal sealed class MailMessage
{
    public string FileName { get; set; } = "";
    public string Content { get; set; } = "";
    public bool IsRead { get; set; }
}

internal sealed class StateManifest
{
    public int SchemaVersion { get; set; } = 1;
    public string OperatingSystem { get; set; } = "minix";
    public DateTime CreatedUtc { get; set; } = DateTime.UtcNow;
    public DateTime UpdatedUtc { get; set; } = DateTime.UtcNow;
}

internal sealed class UserProfile
{
    public string UserName { get; set; } = "linus";
    public string Password { get; set; } = "";
    public DateTime? LastLogin { get; set; }
    public string HomeDirectory { get; set; } = "/usr/linus";
    public string Shell { get; set; } = "/bin/sh";
}

internal sealed class ClockState
{
    public ulong UptimeMs { get; set; }
}

internal sealed record FileStat(
    string Permissions,
    int Links,
    string Owner,
    string Group,
    int Size,
    DateTime Modified);

internal sealed class ProcessEntry
{
    public int Pid { get; set; }
    public int Ppid { get; set; }
    public int Uid { get; set; }
    public string Name { get; set; } = "";
    public string User { get; set; } = "root";
    public char StateCh { get; set; } = 'S';   // S=sleep R=run W=wait Z=zombie
    public string Tty { get; set; } = "?";
    public int Sz { get; set; }                 // size in clicks
    public double CpuPercent { get; set; }
    public double MemoryPercent { get; set; }

    // Legacy compat
    public string State
    {
        get => StateCh switch { 'R' => "running", 'W' => "waiting", 'Z' => "zombie", _ => "sleeping" };
        set => StateCh = value switch { "running" => 'R', "waiting" => 'W', "zombie" => 'Z', _ => 'S' };
    }
}
