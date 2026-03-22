namespace CognitosOs.Kernel.Clock;

/// <summary>
/// Simulated system clock starting at the game epoch (1991-09-17).
/// Advances via <see cref="Advance"/> called from Kernel.Tick.
/// </summary>
internal interface IClock
{
    /// <summary>Current simulated time.</summary>
    DateTime Now();

    /// <summary>Milliseconds since boot.</summary>
    ulong UptimeMs();

    /// <summary>The epoch — when the machine was "first powered on".</summary>
    DateTime Epoch { get; }

    /// <summary>Advance clock by <paramref name="dtMs"/> milliseconds.</summary>
    void Advance(ulong dtMs);
}
