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
        if (result == ApplicationResult.Exit)
        {
            using var exitUow = CreateScope(session);
            _stack.Peek().OnExit(exitUow);
            FlushOutput(exitUow);
            _stack.Pop();
            _screen.Append("");
        }
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
}
