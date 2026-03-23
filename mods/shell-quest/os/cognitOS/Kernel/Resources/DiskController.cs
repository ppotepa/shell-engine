namespace CognitOS.Kernel.Resources;

using CognitOS.Kernel.Hardware;

/// <summary>
/// Single-spindle disk controller. Simulates head contention and spindle state:
/// - Stopped: after 30s idle
/// - Coasting: 2-30s idle (slower acceleration)
/// - Running: recently accessed
/// </summary>
internal sealed class DiskController
{
    private readonly HardwareProfile _hw;
    private readonly Random _rng = new();

    public int ActiveIoCount { get; private set; }
    public int TotalOps { get; private set; }
    public int ContentionEvents { get; private set; }
    
    // Spindle state tracking
    private ulong _lastAccessMs;
    private enum SpindleState { Stopped, Coasting, Running }
    private SpindleState _state = SpindleState.Running;

    public DiskController(HardwareProfile hw)
    {
        _hw = hw;
        _lastAccessMs = 0;
    }

    /// <summary>
    /// Update spindle state based on time since last access.
    /// Called by kernel tick to recalculate idle transitions.
    /// </summary>
    public void UpdateSpindleState(ulong nowMs)
    {
        ulong idleMs = nowMs - _lastAccessMs;
        
        if (idleMs > (ulong)_hw.DiskIdleStopMs)
            _state = SpindleState.Stopped;
        else if (idleMs > (ulong)_hw.DiskCoastThresholdMs)
            _state = SpindleState.Coasting;
        else
            _state = SpindleState.Running;
    }

    /// <summary>
    /// Acquire the disk for an I/O operation.
    /// If spindle was idle, add spin-up cost to first access.
    /// If other operations are in-flight, there's head contention.
    /// </summary>
    public double Acquire(ulong nowMs)
    {
        _lastAccessMs = nowMs;
        ActiveIoCount++;
        TotalOps++;

        double spinUpCost = _state switch
        {
            SpindleState.Stopped => _hw.DiskSpinUpMs,
            SpindleState.Coasting => _hw.DiskCoastMs,
            SpindleState.Running => 0,
            _ => 0,
        };

        _state = SpindleState.Running;

        if (ActiveIoCount <= 1) return spinUpCost;

        // Probability of contention grows with concurrent I/O
        double busyChance = Math.Min(0.8, (ActiveIoCount - 1) * 0.15);
        if (_rng.NextDouble() < busyChance)
        {
            ContentionEvents++;
            return spinUpCost + _hw.DiskSeekMs * 0.7; // head was elsewhere
        }

        return spinUpCost;
    }

    /// <summary>
    /// Acquire for resource accounting only (no spindle state update).
    /// Used by Debit() before executing the command.
    /// </summary>
    public void ReserveForAccounting()
    {
        ActiveIoCount++;
        TotalOps++;
    }

    /// <summary>Release the disk after I/O completes.</summary>
    public void Release()
    {
        ActiveIoCount = Math.Max(0, ActiveIoCount - 1);
    }

    /// <summary>
    /// Full disk access cost including potential spin-up and contention.
    /// </summary>
    public double AccessCost(ulong nowMs)
    {
        double spinUp = Acquire(nowMs);
        return spinUp + _hw.DiskAccessMs;
    }

    /// <summary>
    /// Mark that a service started background I/O (for contention calculation).
    /// Called by ServiceManager during Tick.
    /// </summary>
    public void ServiceIoBegin(ulong nowMs) => Acquire(nowMs);

    /// <summary>Mark service background I/O complete.</summary>
    public void ServiceIoEnd() => Release();

    /// <summary>
    /// Returns spin-up + seek penalty if other I/O is in-flight, without mutating state.
    /// Called by <see cref="CognitOS.Minix.Kernel.MinixSyscallGate"/> for latency calculation.
    /// </summary>
    public double ContentionMs()
    {
        double spinUpCost = _state switch
        {
            SpindleState.Stopped => _hw.DiskSpinUpMs * 0.3,    // predict potential spin-up
            SpindleState.Coasting => _hw.DiskCoastMs * 0.1,
            SpindleState.Running => 0,
            _ => 0,
        };
        return spinUpCost + (ActiveIoCount > 0 ? _hw.DiskAccessMs * 0.1 : 0);
    }
}
