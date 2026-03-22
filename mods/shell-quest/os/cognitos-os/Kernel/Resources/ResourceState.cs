namespace CognitosOs.Kernel.Resources;

using CognitosOs.Core;
using CognitosOs.Kernel.Hardware;

/// <summary>
/// Central resource accounting. Holds all resource pools and recalculates
/// derived values (cache max, CPU load) each tick.
/// </summary>
internal sealed class ResourceState
{
    public RamAllocator Ram { get; }
    public BufferCache Cache { get; }
    public DiskController DiskCtrl { get; }
    public CpuScheduler Cpu { get; }
    public FdTable Fd { get; }
    public NetworkController NetCtrl { get; }

    // Kernel fixed memory breakdown (KB)
    private const int KernelBaseKb = 64;
    private const int MmKb = 16;
    private const int FsKb = 24;
    private const int TtyKb = 5;
    public static int KernelFixedKb => KernelBaseKb + MmKb + FsKb + TtyKb; // 109 KB

    public ResourceState(MachineSpec spec, HardwareProfile hw)
    {
        Ram = new RamAllocator(
            totalRamKb: spec.RamKb,
            kernelKb: KernelFixedKb,
            diskTotalKb: spec.DiskKb,
            diskFreeKb: spec.DiskFreeKb
        );

        int initialCacheMax = Math.Min(Ram.FreeKb / 2, 2048);
        Cache = new BufferCache(initialCacheMax);

        DiskCtrl = new DiskController(hw);
        Cpu = new CpuScheduler();
        Fd = new FdTable(spec.MaxOpenFiles);
        NetCtrl = new NetworkController(hw.NetBandwidthKBs);
    }

    /// <summary>
    /// Called once per tick. Updates derived resource values.
    /// </summary>
    public void Recalc()
    {
        // Sync cache used KB into RAM allocator
        Ram.CacheUsedKb = Cache.UsedKb;

        // Cache max = min(50% of free RAM after processes, hard cap 2048)
        int availableForCache = Math.Max(0, Ram.TotalKb - Ram.KernelKb - Ram.ProcessKb);
        int newCacheMax = Math.Min(availableForCache / 2, 2048);

        if (newCacheMax < Cache.MaxKb)
            Cache.ShrinkTo(newCacheMax);
        else
            Cache.MaxKb = newCacheMax;

        Cpu.RecalcLoadFactor();
    }

    /// <summary>Snapshot for display (ps, df, free, diagnostics).</summary>
    public ResourceSnapshot Snapshot() => new(
        TotalRamKb: Ram.TotalKb,
        KernelKb: Ram.KernelKb,
        ProcessKb: Ram.ProcessKb,
        CacheKb: Cache.UsedKb,
        FreeRamKb: Ram.FreeKb,
        DiskTotalKb: Ram.DiskTotalKb,
        DiskFreeKb: Ram.DiskFreeKb,
        DiskUsedKb: Ram.DiskTotalKb - Ram.DiskFreeKb,
        OpenFds: Fd.OpenCount,
        MaxFds: Fd.MaxFds,
        CacheHits: Cache.Hits,
        CacheMisses: Cache.Misses,
        CacheEntries: Cache.EntryCount,
        CpuLoadFactor: Cpu.LoadFactor,
        RunnableProcesses: Cpu.RunnableCount,
        ActiveNetConnections: NetCtrl.ActiveConnections
    );
}

internal readonly record struct ResourceSnapshot(
    int TotalRamKb,
    int KernelKb,
    int ProcessKb,
    int CacheKb,
    int FreeRamKb,
    int DiskTotalKb,
    int DiskFreeKb,
    int DiskUsedKb,
    int OpenFds,
    int MaxFds,
    int CacheHits,
    int CacheMisses,
    int CacheEntries,
    double CpuLoadFactor,
    int RunnableProcesses,
    int ActiveNetConnections
);
