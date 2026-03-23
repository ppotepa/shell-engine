namespace CognitOS.Kernel.Events;

/// <summary>
/// Time-ordered kernel event queue used to move the sidecar toward non-blocking simulation.
/// Events are scheduled against simulated kernel time and drained from <c>Kernel.Tick</c>.
/// </summary>
internal sealed class KernelEventQueue
{
    private readonly PriorityQueue<ScheduledKernelEvent, (ulong DueAtMs, ulong Sequence)> _queue = new();
    private ulong _nextSequence;

    public int Count => _queue.Count;

    public void ScheduleAt(ulong dueAtMs, Action action, string? tag = null)
        => ScheduleAt(dueAtMs, KernelEventKind.Timer, action, tag);

    public void ScheduleAt(ulong dueAtMs, KernelEventKind kind, Action action, string? tag = null)
    {
        ArgumentNullException.ThrowIfNull(action);
        var sequence = _nextSequence++;
        var scheduled = new ScheduledKernelEvent(dueAtMs, sequence, kind, action, tag);
        _queue.Enqueue(scheduled, (dueAtMs, sequence));
    }

    public void ScheduleAfter(ulong nowMs, ulong delayMs, Action action, string? tag = null)
        => ScheduleAfter(nowMs, delayMs, KernelEventKind.Timer, action, tag);

    public void ScheduleAfter(
        ulong nowMs,
        ulong delayMs,
        KernelEventKind kind,
        Action action,
        string? tag = null)
    {
        var dueAtMs = ulong.MaxValue - nowMs < delayMs
            ? ulong.MaxValue
            : nowMs + delayMs;
        ScheduleAt(dueAtMs, kind, action, tag);
    }

    public List<ScheduledKernelEvent> DrainReady(ulong nowMs)
    {
        var ready = new List<ScheduledKernelEvent>();
        while (_queue.Count > 0 && _queue.TryPeek(out var ev, out var priority) && priority.DueAtMs <= nowMs)
        {
            _queue.Dequeue();
            ready.Add(ev);
        }

        return ready;
    }

    public void Clear() => _queue.Clear();
}

internal enum KernelEventKind
{
    Timer,
    Output,
    Completion,
    Disk,
    Network,
    Modem,
}

internal sealed record ScheduledKernelEvent(
    ulong DueAtMs,
    ulong Sequence,
    KernelEventKind Kind,
    Action Action,
    string? Tag);
