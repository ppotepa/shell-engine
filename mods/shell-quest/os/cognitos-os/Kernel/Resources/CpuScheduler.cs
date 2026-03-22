namespace CognitosOs.Kernel.Resources;

/// <summary>
/// Tracks runnable process count and computes per-operation CPU overhead.
/// Single-core i386 — no parallelism, more processes = more context switches = slower.
/// </summary>
internal sealed class CpuScheduler
{
    public int RunnableCount { get; private set; }
    public double LoadFactor { get; private set; }

    /// <summary>Increment when a process becomes runnable (fork, wake).</summary>
    public void IncrementRunnable()
    {
        RunnableCount++;
        RecalcLoadFactor();
    }

    /// <summary>Decrement when a process exits or blocks.</summary>
    public void DecrementRunnable()
    {
        RunnableCount = Math.Max(0, RunnableCount - 1);
        RecalcLoadFactor();
    }

    /// <summary>
    /// Extra milliseconds per operation due to CPU contention.
    /// More runnable processes = more scheduling overhead.
    /// </summary>
    public double OverheadMs() => RunnableCount switch
    {
        <= 4 => 0,
        <= 6 => 0.5,
        <= 8 => 1.5,
        <= 12 => 3.0,
        _ => 5.0,
    };

    public void RecalcLoadFactor()
    {
        // Normalized: 4 runnable = load 1.0 (normal baseline for this era)
        LoadFactor = RunnableCount / 4.0;
    }
}
