namespace CognitOS.Kernel.Hardware;

using CognitOS.Core;

/// <summary>
/// Derived timing constants from <see cref="MachineSpec"/>.
/// Single source of truth for all simulated hardware delays.
/// All values in milliseconds or KB/s.
/// </summary>
internal class HardwareProfile
{
    // Disk — Seagate ST-157A era (3.5" MFM/IDE)
    public double DiskSeekMs { get; }
    public double DiskRotationMs { get; }
    public double DiskAccessMs { get; }
    public double DiskTransferKBs { get; }
    public double DiskDirEntryMs { get; }

    // Network — NE2000 / 3Com EtherLink III
    public double NetBandwidthKBs { get; }
    public double NetBasePingMs { get; }

    // Disk — spindle state transitions
    /// <summary>Spin-up from fully stopped (disk idle >30s): ~300ms for a 1991 5400-RPM drive.</summary>
    public double DiskSpinUpMs { get; }
    /// <summary>Partial spin-up (disk idle 2-30s, platter still coasting): ~80ms.</summary>
    public double DiskCoastMs { get; }
    /// <summary>After this many ms idle the spindle stops completely.</summary>
    public double DiskIdleStopMs { get; }
    /// <summary>After this many ms idle the spindle is coasting (slower than full speed).</summary>
    public double DiskCoastThresholdMs { get; }

    // CPU — fork/exec/context-switch
    public double ForkMs { get; }
    public double ContextSwitchMs { get; }

    // Raw spec reference
    public MachineSpec Spec { get; }

    private HardwareProfile(MachineSpec spec)
    {
        Spec = spec;
        double m = spec.OperationSpeedMultiplier;

        DiskSeekMs       = 15.0 * m;
        DiskRotationMs   = 8.3 * m;
        DiskAccessMs     = DiskSeekMs + DiskRotationMs;
        DiskTransferKBs  = 750.0 / m;
        DiskDirEntryMs   = DiskAccessMs / 4.0;
        DiskSpinUpMs     = 300.0 * m;        // full stop to full speed
        DiskCoastMs      = 80.0 * m;         // coasting to full speed
        DiskIdleStopMs   = 30000.0 * m;      // 30s idle stops spindle
        DiskCoastThresholdMs = 2000.0 * m;   // 2s idle: transition to coast

        NetBandwidthKBs  = spec.ModemBaud / 8000.0;  // baud → KB/s
        NetBasePingMs    = 120.0 * m;

        ForkMs           = 80.0 * m;
        ContextSwitchMs  = 1.0 * m;
    }

    /// <summary>
    /// Compute time to load a binary of <paramref name="sizeKb"/> from disk.
    /// </summary>
    public double ExecLoadMs(double sizeKb) =>
        DiskAccessMs + (sizeKb / DiskTransferKBs * 1000.0);

    /// <summary>
    /// Compute time to transfer <paramref name="sizeKb"/> over disk.
    /// </summary>
    public double DiskTransferMs(double sizeKb) =>
        sizeKb / DiskTransferKBs * 1000.0;

    /// <summary>
    /// Compute time to transfer <paramref name="sizeKb"/> over network.
    /// </summary>
    public double NetTransferMs(double sizeKb) =>
        sizeKb / NetBandwidthKBs * 1000.0;

    /// <summary>
    /// Block the current thread for the specified duration.
    /// This is the sole delay source — every subsystem calls this.
    /// </summary>
    public virtual void BlockFor(double ms)
    {
        int wait = (int)Math.Max(ms, 0);
        if (wait > 0)
            Thread.Sleep(wait);
    }

    /// <summary>
    /// Production profile — real delays derived from spec.
    /// </summary>
    public static HardwareProfile FromSpec(MachineSpec spec) => new(spec);

    /// <summary>
    /// Test/instant profile — zero delays, for unit tests.
    /// </summary>
    public static HardwareProfile Instant(MachineSpec spec) => new InstantProfile(spec);

    private sealed class InstantProfile : HardwareProfile
    {
        internal InstantProfile(MachineSpec spec) : base(spec) { }
        public override void BlockFor(double _) { }
    }
}
