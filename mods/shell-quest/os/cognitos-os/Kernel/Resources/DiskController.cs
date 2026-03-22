namespace CognitosOs.Kernel.Resources;

using CognitosOs.Kernel.Hardware;

/// <summary>
/// Single-spindle disk controller. Simulates head contention:
/// if services are doing I/O, user commands may incur extra seek cost.
/// Single-threaded (i386 era) — contention is probabilistic, not lock-based.
/// </summary>
internal sealed class DiskController
{
    private readonly HardwareProfile _hw;
    private readonly Random _rng = new();

    public int ActiveIoCount { get; private set; }
    public int TotalOps { get; private set; }
    public int ContentionEvents { get; private set; }

    public DiskController(HardwareProfile hw)
    {
        _hw = hw;
    }

    /// <summary>
    /// Acquire the disk for an I/O operation.
    /// If other operations are in-flight (service background I/O),
    /// there's a chance the head is elsewhere → extra seek penalty.
    /// Returns the extra delay in ms (0 if no contention).
    /// </summary>
    public double Acquire()
    {
        ActiveIoCount++;
        TotalOps++;

        if (ActiveIoCount <= 1) return 0;

        // Probability of contention grows with concurrent I/O
        double busyChance = Math.Min(0.8, (ActiveIoCount - 1) * 0.15);
        if (_rng.NextDouble() < busyChance)
        {
            ContentionEvents++;
            return _hw.DiskSeekMs * 0.7; // head was elsewhere
        }

        return 0;
    }

    /// <summary>Release the disk after I/O completes.</summary>
    public void Release()
    {
        ActiveIoCount = Math.Max(0, ActiveIoCount - 1);
    }

    /// <summary>
    /// Full disk access cost including potential contention.
    /// Combines: acquire penalty + seek + rotation.
    /// </summary>
    public double AccessCost()
    {
        double contention = Acquire();
        return contention + _hw.DiskAccessMs;
    }

    /// <summary>
    /// Mark that a service started background I/O (for contention calculation).
    /// Called by ServiceManager during Tick.
    /// </summary>
    public void ServiceIoBegin() => ActiveIoCount++;

    /// <summary>Mark service background I/O complete.</summary>
    public void ServiceIoEnd() => ActiveIoCount = Math.Max(0, ActiveIoCount - 1);

    /// <summary>
    /// Returns extra seek penalty in ms if other I/O is in-flight, without mutating state.
    /// Called by <see cref="CognitosOs.Minix.Kernel.MinixSyscallGate"/> for latency calculation.
    /// </summary>
    public double ContentionMs() => ActiveIoCount > 0 ? _hw.DiskAccessMs * 0.1 : 0;
}
