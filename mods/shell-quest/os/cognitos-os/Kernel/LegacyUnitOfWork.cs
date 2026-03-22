namespace CognitosOs.Kernel;

using CognitosOs.Core;
using CognitosOs.Kernel.Clock;
using CognitosOs.Kernel.Disk;
using CognitosOs.Kernel.Journal;
using CognitosOs.Kernel.Mail;
using CognitosOs.Kernel.Network;
using CognitosOs.Kernel.Process;
using CognitosOs.Kernel.Resources;
using CognitosOs.State;

/// <summary>
/// Bridge UoW that wraps the legacy <see cref="IOperatingSystem"/> subsystems
/// so that IKernelCommand can be used before the full Kernel is wired up.
/// Output is buffered in StringWriter — caller reads via Out.ToString().
/// </summary>
internal sealed class LegacyUnitOfWork : IUnitOfWork
{
    private readonly IOperatingSystem _os;
    private readonly LegacyDisk _disk;
    private readonly LegacyProcessTable _proc;
    private readonly LegacyClock _clock;

    public LegacyUnitOfWork(IOperatingSystem os, UserSession session)
    {
        _os = os;
        Session = session;
        Out = new StringWriter();
        Err = new StringWriter();
        _disk = new LegacyDisk(os.FileSystem);
        _proc = new LegacyProcessTable(os);
        _clock = new LegacyClock(os);
    }

    public TextWriter Out { get; }
    public TextWriter Err { get; }
    public IDisk Disk => _disk;
    public INetwork Net => throw new NotSupportedException("Network not available in legacy mode");
    public IProcessTable Process => _proc;
    public IClock Clock => _clock;
    public IMailSpool Mail => throw new NotSupportedException("Mail not available in legacy mode");
    public IJournal Journal => throw new NotSupportedException("Journal not available in legacy mode");
    public UserSession Session { get; }
    public QuestState Quest => _os.State.Quest;
    public MachineSpec Spec => _os.Spec;

    public ResourceSnapshot Resources => new(
        TotalRamKb: Spec.RamKb,
        KernelKb: 109,
        ProcessKb: _os.ProcessSnapshot().Sum(p => p.Sz),
        CacheKb: 0,
        FreeRamKb: Spec.RamKb - 109 - _os.ProcessSnapshot().Sum(p => p.Sz),
        DiskTotalKb: Spec.DiskKb,
        DiskFreeKb: Spec.DiskFreeKb,
        DiskUsedKb: Spec.DiskKb - Spec.DiskFreeKb,
        OpenFds: 3,
        MaxFds: Spec.MaxOpenFiles,
        CacheHits: 0,
        CacheMisses: 0,
        CacheEntries: 0,
        CpuLoadFactor: _os.UsageSnapshot().CpuPercent / 100.0,
        RunnableProcesses: _os.ProcessSnapshot().Count,
        ActiveNetConnections: 0
    );

    public void Dispose()
    {
        Out.Flush();
        Err.Flush();
    }
}

/// <summary>Bridge IDisk that wraps legacy IVirtualFileSystem.</summary>
internal sealed class LegacyDisk : IDisk
{
    private readonly IVirtualFileSystem _fs;

    public LegacyDisk(IVirtualFileSystem fs) => _fs = fs;

    public string ReadFile(string path)
    {
        var vfs = _fs.ToVfsPath(path);
        if (!_fs.TryCat(vfs, out var content))
            throw new FileNotFoundException(path);
        return content;
    }

    public void WriteFile(string path, string content)
    {
        var vfs = _fs.ToVfsPath(path);
        if (_fs is IMutableFileSystem mfs)
        {
            if (!mfs.TryWrite(vfs, content, out var error))
                throw new IOException(error ?? "Write failed");
        }
        else
        {
            throw new IOException("Read-only file system");
        }
    }

    public void AppendFile(string path, string content)
    {
        var existing = RawRead(path) ?? "";
        WriteFile(path, existing + content);
    }

    public IReadOnlyList<string> ReadDir(string path)
    {
        var vfs = _fs.ToVfsPath(path);
        if (!_fs.DirectoryExists(vfs))
            throw new DirectoryNotFoundException(path);
        return _fs.Ls(vfs).ToList();
    }

    public FileStat Stat(string path)
    {
        var vfs = _fs.ToVfsPath(path);
        return _fs.GetStat(vfs) ?? throw new FileNotFoundException(path);
    }

    public void Mkdir(string path)
    {
        if (_fs is IMutableFileSystem mfs)
            mfs.TryMkdir(_fs.ToVfsPath(path), out _);
    }

    public void Unlink(string path)
    {
        if (_fs is IMutableFileSystem mfs)
            mfs.TryDelete(_fs.ToVfsPath(path));
    }

    public bool Exists(string path)
    {
        var vfs = _fs.ToVfsPath(path);
        return _fs.TryCat(vfs, out _) || _fs.DirectoryExists(vfs);
    }

    public string? RawRead(string path)
    {
        var vfs = _fs.ToVfsPath(path);
        return _fs.TryCat(vfs, out var content) ? content : null;
    }

    public IReadOnlyList<string>? RawReadDir(string path)
    {
        var vfs = _fs.ToVfsPath(path);
        if (!_fs.DirectoryExists(vfs)) return null;
        return _fs.Ls(vfs).ToList();
    }
}

/// <summary>Bridge IProcessTable that wraps legacy IOperatingSystem.</summary>
internal sealed class LegacyProcessTable : IProcessTable
{
    private readonly IOperatingSystem _os;

    public LegacyProcessTable(IOperatingSystem os) => _os = os;

    public IReadOnlyList<ProcessEntry> List() => _os.ProcessSnapshot();
    public ProcessEntry? Get(int pid) => _os.ProcessSnapshot().FirstOrDefault(p => p.Pid == pid);
    public int NextPid() => _os.ProcessSnapshot().Max(p => p.Pid) + 1;

    public int Fork(string name, int sizeKb, string user, string tty) => NextPid();
    public void Exec(int pid, string binaryPath) { }
    public void Exit(int pid) { }
    public void Kill(int pid, int signal) { }
}

/// <summary>Bridge IClock that wraps legacy IOperatingSystem.</summary>
internal sealed class LegacyClock : IClock
{
    private readonly IOperatingSystem _os;
    private static readonly DateTime EpochValue = new(1991, 9, 17, 21, 12, 0, DateTimeKind.Utc);

    public LegacyClock(IOperatingSystem os) => _os = os;

    public DateTime Now() => _os.SimulatedNow();
    public ulong UptimeMs() => _os.State.UptimeMs;
    public DateTime Epoch => EpochValue;
    public void Advance(ulong dtMs) { }
}
