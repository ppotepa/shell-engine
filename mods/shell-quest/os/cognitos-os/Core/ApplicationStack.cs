namespace CognitosOs.Core;

/// <summary>
/// Manages the stack of open applications.
/// Input always routes to the topmost app. When an app exits it is popped
/// and the app below regains focus — exactly like a Unix process tree.
/// </summary>
internal sealed class ApplicationStack
{
    private readonly Stack<IApplication> _stack = new();

    public bool IsEmpty => _stack.Count == 0;

    public IApplication? Current => _stack.Count > 0 ? _stack.Peek() : null;

    /// <summary>Pushes a new application and calls its OnEnter.</summary>
    public void Push(IApplication app, UserSession session)
    {
        _stack.Push(app);
        app.OnEnter(session);
    }

    /// <summary>
    /// Routes one line of input to the topmost app.
    /// If the app returns Exit, pops it and calls OnExit.
    /// </summary>
    public void HandleInput(string input, UserSession session)
    {
        if (_stack.Count == 0) return;

        var result = _stack.Peek().HandleInput(input, session);
        if (result == ApplicationResult.Exit)
        {
            _stack.Peek().OnExit(session);
            _stack.Pop();
        }
    }

    /// <summary>Returns the prompt prefix of the topmost app.</summary>
    public string CurrentPrompt(UserSession session)
        => _stack.Count > 0 ? _stack.Peek().PromptPrefix(session) : "";
}
