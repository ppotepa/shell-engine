namespace CognitosOs.Kernel.Resources;

/// <summary>
/// Tracks RAM allocation: kernel fixed + process working sets + buffer cache.
/// FreeKb = TotalKb - KernelKb - ProcessKb - CacheUsedKb.
/// </summary>
internal sealed class RamAllocator
{
    public int TotalKb { get; }
    public int KernelKb { get; }
    public int ProcessKb { get; private set; }

    /// <summary>Set externally by <see cref="BufferCache"/> via <see cref="ResourceState.Recalc"/>.</summary>
    public int CacheUsedKb { get; set; }

    public int FreeKb => Math.Max(0, TotalKb - KernelKb - ProcessKb - CacheUsedKb);

    // Disk tracking (separate from RAM but kept here for single accounting point)
    public int DiskTotalKb { get; }
    public int DiskFreeKb { get; private set; }

    public RamAllocator(int totalRamKb, int kernelKb, int diskTotalKb, int diskFreeKb)
    {
        TotalKb = totalRamKb;
        KernelKb = kernelKb;
        DiskTotalKb = diskTotalKb;
        DiskFreeKb = diskFreeKb;
    }

    /// <summary>
    /// Check whether <paramref name="kb"/> of RAM can be allocated for a process.
    /// Considers potential cache eviction: if free + cache > kb, allocation is possible.
    /// </summary>
    public bool CanAllocProcess(int kb) =>
        (FreeKb + CacheUsedKb) >= kb;

    /// <summary>Allocate process memory. Caller must check <see cref="CanAllocProcess"/> first.</summary>
    public void AllocProcess(int kb) => ProcessKb += kb;

    /// <summary>Free process memory on exit.</summary>
    public void FreeProcess(int kb) => ProcessKb = Math.Max(0, ProcessKb - kb);

    /// <summary>Check if disk has enough free space.</summary>
    public bool CheckDiskFree(int kb) => DiskFreeKb >= kb;

    /// <summary>Consume disk space after a write.</summary>
    public void ConsumeDisk(int kb) => DiskFreeKb = Math.Max(0, DiskFreeKb - kb);

    /// <summary>Release disk space after a delete.</summary>
    public void ReleaseDisk(int kb) => DiskFreeKb = Math.Min(DiskTotalKb, DiskFreeKb + kb);
}
