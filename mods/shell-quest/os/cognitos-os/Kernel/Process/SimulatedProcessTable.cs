namespace CognitosOs.Kernel.Process;

using CognitosOs.Framework.Kernel;
using CognitosOs.Kernel.Clock;
using CognitosOs.Kernel.Hardware;
using CognitosOs.Kernel.Resources;
using CognitosOs.State;

/// <summary>
/// Simulated MINIX process table. Fork/Exec go through <see cref="ISyscallGate"/>
/// for unified resource debit + latency injection.
/// </summary>
internal sealed class SimulatedProcessTable : IProcessTable
{
    private readonly List<ProcessEntry> _processes = new();
    private readonly ResourceState _res;
    private readonly HardwareProfile _hw;
    private readonly IClock _clock;
    private readonly ISyscallGate _gate;
    private int _nextPid = 1;

    // Binary size table (KB) — approximate 1991 MINIX binary sizes
    private static readonly Dictionary<string, int> BinarySizes = new(StringComparer.OrdinalIgnoreCase)
    {
        ["kernel"] = 32, ["mm"] = 24, ["fs"] = 48, ["init"] = 8,
        ["update"] = 4, ["cron"] = 8, ["getty"] = 8, ["sh"] = 16,
        ["ls"] = 4, ["cat"] = 2, ["cp"] = 3, ["ps"] = 4,
        ["who"] = 2, ["whoami"] = 1, ["uname"] = 1, ["date"] = 2,
        ["man"] = 8, ["mail"] = 12, ["ftp"] = 16, ["ed"] = 10,
        ["finger"] = 4, ["write"] = 2, ["tar"] = 8, ["compress"] = 6,
        ["clear"] = 1, ["pwd"] = 1, ["cd"] = 1, ["echo"] = 1,
        ["head"] = 2, ["tail"] = 2, ["wc"] = 2, ["grep"] = 4,
        ["kill"] = 1, ["df"] = 2, ["mount"] = 3, ["sync"] = 1,
        ["ping"] = 4, ["help"] = 2, ["history"] = 2,
    };

    public SimulatedProcessTable(ResourceState res, HardwareProfile hw, IClock clock, ISyscallGate gate)
    {
        _res = res;
        _hw = hw;
        _clock = clock;
        _gate = gate;
    }

    public int Fork(string name, int sizeKb, string user, string tty)
    {
        if (_processes.Count >= _hw.Spec.MaxProcesses)
            throw new InvalidOperationException("fork: process table full");

        int pid = _nextPid++;
        var result = _gate.Dispatch(
            SyscallRequest.For(SyscallKind.ProcessFork, sizeKb * 1024L),
            () =>
            {
                _processes.Add(new ProcessEntry
                {
                    Pid = pid,
                    Ppid = 1,
                    Uid = user == "root" ? 0 : (user == "ast" ? 100 : 101),
                    Name = name,
                    User = user,
                    StateCh = 'R',
                    Tty = tty,
                    Sz = sizeKb,
                });
            });

        if (!result.Success)
            throw new InvalidOperationException($"fork: {result.ErrorCode}");

        return pid;
    }

    public void Exec(int pid, string binaryPath)
    {
        string binName = System.IO.Path.GetFileName(binaryPath);
        int binSize = BinarySizes.GetValueOrDefault(binName, 4);

        _gate.Dispatch(
            SyscallRequest.For(SyscallKind.ProcessExec, binSize * 1024L),
            () => { /* binary image loaded — state already committed */ }
        ).ThrowIfFailed();
    }

    public void Exit(int pid)
    {
        var proc = _processes.FindIndex(p => p.Pid == pid);
        if (proc < 0) return;

        var entry = _processes[proc];
        _res.Ram.FreeProcess(entry.Sz);
        _res.Cpu.DecrementRunnable();
        _processes.RemoveAt(proc);
    }

    public void Kill(int pid, int signal)
    {
        var proc = _processes.Find(p => p.Pid == pid);
        if (proc is null) return;

        if (signal == 9)
            Exit(pid);
    }

    public IReadOnlyList<ProcessEntry> List() => _processes.AsReadOnly();

    public int NextPid() => _nextPid;

    public ProcessEntry? Get(int pid) => _processes.Find(p => p.Pid == pid);

    /// <summary>
    /// Add a system process directly (for boot, not via fork delays).
    /// Used during kernel initialization.
    /// </summary>
    public void AddSystemProcess(ProcessEntry entry)
    {
        if (entry.Pid >= _nextPid) _nextPid = entry.Pid + 1;
        _processes.Add(entry);
        _res.Ram.AllocProcess(entry.Sz);
        _res.Cpu.IncrementRunnable();
    }

    /// <summary>Lookup binary size for a command name.</summary>
    public static int GetBinarySize(string name) =>
        BinarySizes.GetValueOrDefault(name, 4);
}
