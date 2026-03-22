namespace CognitosOs.Kernel.Clock;

/// <summary>
/// In-memory clock that starts at <see cref="Epoch"/> and accumulates elapsed time.
/// Thread-safe: Advance and Now can be called from tick and command threads.
/// </summary>
internal sealed class SimulatedClock : IClock
{
    private ulong _elapsedMs;

    public DateTime Epoch { get; }

    public SimulatedClock(DateTime epoch)
    {
        Epoch = epoch;
    }

    public DateTime Now() => Epoch.AddMilliseconds(_elapsedMs);

    public ulong UptimeMs() => _elapsedMs;

    public void Advance(ulong dtMs) => _elapsedMs += dtMs;
}
