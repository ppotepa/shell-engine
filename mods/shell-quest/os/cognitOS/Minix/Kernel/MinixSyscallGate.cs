namespace CognitOS.Minix.Kernel;

using CognitOS.Framework.Kernel;
using CognitOS.Kernel.Clock;
using CognitOS.Kernel.Hardware;
using CognitOS.Kernel.Resources;

internal sealed class MinixSyscallGate : ISyscallGate
{
    private readonly ResourceState _res;
    private readonly HardwareProfile _hw;
    private readonly IClock _clock;

    public MinixSyscallGate(ResourceState res, HardwareProfile hw, IClock clock)
    {
        _res = res;
        _hw = hw;
        _clock = clock;
    }

    public SyscallResult Dispatch(SyscallRequest req, Action execute)
    {
        var check = CanSatisfy(req);
        if (check is not null) return SyscallResult.Fail(check);

        Debit(req);
        double latencyMs = LatencyFor(req);
        _hw.BlockFor(latencyMs);

        try
        {
            execute();
        }
        finally
        {
            Credit(req);
        }

        return SyscallResult.Ok((int)latencyMs);
    }

    public SyscallResult Dispatch<T>(SyscallRequest req, Func<T> execute, out T result)
    {
        var check = CanSatisfy(req);
        if (check is not null)
        {
            result = default!;
            return SyscallResult.Fail(check);
        }

        Debit(req);
        double latencyMs = LatencyFor(req);
        _hw.BlockFor(latencyMs);

        T value = default!;
        try
        {
            value = execute();
        }
        finally
        {
            Credit(req);
        }

        result = value;
        return SyscallResult.Ok((int)latencyMs);
    }

    // --- Resource check ---

    private string? CanSatisfy(SyscallRequest req) => req.Kind switch
    {
        SyscallKind.DiskWrite or SyscallKind.DiskAppend =>
            !_res.Ram.CheckDiskFree((int)Math.Max(1, req.SizeBytes / 1024))
                ? "ENOSPC" : null,

        SyscallKind.ProcessFork or SyscallKind.ProcessExec =>
            !_res.Ram.CanAllocProcess((int)Math.Max(4, req.SizeBytes / 1024))
                ? "ENOMEM" : null,

        SyscallKind.MemAlloc =>
            !_res.Ram.CanAllocProcess((int)Math.Max(1, req.SizeBytes / 1024))
                ? "ENOMEM" : null,

        SyscallKind.NetConnect =>
            !_res.NetCtrl.CanConnect()
                ? "ECONNREFUSED" : null,

        SyscallKind.DiskRead or SyscallKind.DiskListDir or SyscallKind.DiskStat =>
            null, // reads never blocked by space

        _ => null,
    };

    // --- Debit (before execute) ---

    private void Debit(SyscallRequest req)
    {
        switch (req.Kind)
        {
            case SyscallKind.DiskRead:
            case SyscallKind.DiskWrite:
            case SyscallKind.DiskAppend:
                _res.DiskCtrl.ReserveForAccounting();
                break;

            case SyscallKind.NetConnect:
            case SyscallKind.NetSend:
            case SyscallKind.NetRecv:
                _res.NetCtrl.Acquire();
                break;

            case SyscallKind.ProcessFork:
            case SyscallKind.ProcessExec:
                _res.Cpu.IncrementRunnable();
                _res.Ram.AllocProcess((int)Math.Max(4, req.SizeBytes / 1024));
                break;
        }
    }

    // --- Credit (after execute) ---

    private void Credit(SyscallRequest req)
    {
        switch (req.Kind)
        {
            case SyscallKind.DiskRead:
            case SyscallKind.DiskWrite:
            case SyscallKind.DiskAppend:
                _res.DiskCtrl.Release();
                break;

            case SyscallKind.NetConnect:
            case SyscallKind.NetSend:
            case SyscallKind.NetRecv:
                _res.NetCtrl.Release();
                break;

            case SyscallKind.ProcessExit:
            case SyscallKind.ProcessKill:
                _res.Cpu.DecrementRunnable();
                break;
        }
    }

    // --- Latency table ---

    private double LatencyFor(SyscallRequest req)
    {
        double sizeKb = Math.Max(1, req.SizeBytes / 1024.0);
        ulong nowMs = _clock.UptimeMs();
        
        return req.Kind switch
        {
            SyscallKind.DiskRead      => _res.DiskCtrl.Acquire(nowMs) + _hw.DiskTransferMs(sizeKb) + _res.DiskCtrl.ContentionMs() + _res.Cpu.OverheadMs(),
            SyscallKind.DiskWrite     => _res.DiskCtrl.Acquire(nowMs) + _hw.DiskTransferMs(sizeKb) + _res.DiskCtrl.ContentionMs() + _res.Cpu.OverheadMs(),
            SyscallKind.DiskAppend    => _res.DiskCtrl.Acquire(nowMs) + _hw.DiskTransferMs(sizeKb) + _res.DiskCtrl.ContentionMs() + _res.Cpu.OverheadMs(),
            SyscallKind.DiskStat      => _res.DiskCtrl.Acquire(nowMs) + _hw.DiskDirEntryMs + _res.Cpu.OverheadMs(),
            SyscallKind.DiskListDir   => _res.DiskCtrl.Acquire(nowMs) + _hw.DiskDirEntryMs * 2 + _res.Cpu.OverheadMs(),
            SyscallKind.DiskUnlink    => _res.DiskCtrl.Acquire(nowMs) + _res.Cpu.OverheadMs(),
            SyscallKind.DiskMkdir     => _res.DiskCtrl.Acquire(nowMs) + _res.Cpu.OverheadMs(),

            SyscallKind.NetConnect    => _hw.NetBasePingMs,
            SyscallKind.NetSend       => _hw.NetTransferMs(sizeKb),
            SyscallKind.NetRecv       => _hw.NetBasePingMs + _hw.NetTransferMs(sizeKb),
            SyscallKind.NetResolve    => _hw.NetBasePingMs * 2,
            SyscallKind.NetClose      => _res.Cpu.OverheadMs(),

            SyscallKind.ProcessFork   => _hw.ForkMs + _res.Cpu.OverheadMs(),
            SyscallKind.ProcessExec   => _hw.ExecLoadMs(sizeKb) + _res.Cpu.OverheadMs(),
            SyscallKind.ProcessExit   => _res.Cpu.OverheadMs(),
            SyscallKind.ProcessKill   => _res.Cpu.OverheadMs(),
            SyscallKind.ProcessList   => _res.Cpu.OverheadMs(),

            SyscallKind.MemAlloc      => _res.Cpu.OverheadMs(),
            SyscallKind.MemFree       => 0,

            SyscallKind.ClockRead     => 0,

            SyscallKind.MailRead      => _res.DiskCtrl.Acquire(nowMs) + _res.Cpu.OverheadMs(),
            SyscallKind.MailDeliver   => _res.DiskCtrl.Acquire(nowMs) + _res.Cpu.OverheadMs(),

            SyscallKind.JournalAppend => _hw.DiskDirEntryMs,

            _ => 0,
        };
    }
}
