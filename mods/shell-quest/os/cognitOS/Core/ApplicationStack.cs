namespace CognitOS.Core;

using CognitOS.Framework.Kernel;
using CognitOS.State;

/// <summary>
/// Manages the stack of open applications.
/// Input always routes to the topmost app. When an app exits it is popped
/// and the app below regains focus — exactly like a Unix process tree.
/// </summary>
internal sealed class ApplicationStack
{
    private readonly IKernel _kernel;
    private readonly MachineState _machineState;
    private readonly Stack<IKernelApplication> _stack = new();
    private readonly ScreenBuffer _screen;
    private readonly List<(ulong DueAtMs, string Line)> _pendingDelayed = new();

    public ApplicationStack(IKernel kernel, MachineState machineState, ScreenBuffer screen)
    {
        _kernel = kernel;
        _machineState = machineState;
        _screen = screen;
    }

    public bool IsEmpty => _stack.Count == 0;

    public IKernelApplication? Current => _stack.Count > 0 ? _stack.Peek() : null;

    /// <summary>Pushes a new application and calls its OnEnter.</summary>
    public void Push(IKernelApplication app, UserSession session)
    {
        _screen.Append("");
        _stack.Push(app);
        using var uow = CreateScope(session);
        app.OnEnter(uow);
        FlushOutput(uow);
        SchedulePendingOutputs(uow);
    }

    /// <summary>
    /// Routes one line of input to the topmost app.
    /// If the app returns Exit, pops it and calls OnExit.
    /// </summary>
    public void HandleInput(string input, UserSession session)
    {
        if (_stack.Count == 0) return;

        using var uow = CreateScope(session);
        var result = _stack.Peek().HandleInput(uow, input);
        FlushOutput(uow);
        SchedulePendingOutputs(uow);
        if (result == ApplicationResult.Exit)
        {
            using var exitUow = CreateScope(session);
            _stack.Peek().OnExit(exitUow);
            FlushOutput(exitUow);
            SchedulePendingOutputs(exitUow);
            _stack.Pop();
            _screen.Append("");
        }
    }

    /// <summary>
    /// Drain any pending delayed outputs whose due time has arrived.
    /// Called every tick so delayed lines appear sequentially over time.
    /// </summary>
    public void DrainDelayedOutput(ulong nowMs)
    {
        if (_pendingDelayed.Count == 0) return;

        _pendingDelayed.Sort((a, b) => a.DueAtMs.CompareTo(b.DueAtMs));

        int drained = 0;
        foreach (var (dueAt, line) in _pendingDelayed)
        {
            if (dueAt > nowMs) break;
            _screen.Append(line);
            drained++;
        }

        if (drained > 0)
            _pendingDelayed.RemoveRange(0, drained);
    }

    /// <summary>Returns the prompt prefix of the topmost app.</summary>
    public string CurrentPrompt(UserSession session)
        => _stack.Count > 0 ? _stack.Peek().PromptPrefix(session) : "";

    private CognitOS.Kernel.IUnitOfWork CreateScope(UserSession session)
        => (CognitOS.Kernel.IUnitOfWork)_kernel.CreateScope(session, new StringWriter(), _machineState.Quest);

    private void FlushOutput(CognitOS.Kernel.IUnitOfWork uow)
    {
        uow.Out.Flush();
        var text = uow.Out.ToString();
        if (string.IsNullOrEmpty(text))
            return;

        var lines = text.TrimEnd('\r', '\n').Split('\n');
        _screen.Append(lines.Concat(new[] { "" }).ToArray());
    }

    private void SchedulePendingOutputs(CognitOS.Kernel.IUnitOfWork uow)
    {
        var scheduled = uow.DrainScheduledOutputs();
        if (scheduled.Count == 0) return;

        var baseMs = _kernel.Clock.UptimeMs();
        foreach (var (delayMs, line) in scheduled)
            _pendingDelayed.Add((baseMs + delayMs, line));
    }
}
