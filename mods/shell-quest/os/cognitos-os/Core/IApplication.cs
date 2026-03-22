namespace CognitosOs.Core;

using CognitosOs.Kernel;

internal enum ApplicationResult
{
    Continue,
    Exit,
}

/// <summary>
/// Represents a running application in the application stack.
/// Input from the keyboard always routes to the topmost application.
/// </summary>
internal interface IApplication
{
    /// <summary>The prompt prefix shown when this app is topmost on the stack.</summary>
    string PromptPrefix(UserSession session);

    /// <summary>Called once when this app is pushed onto the stack.</summary>
    void OnEnter(UserSession session);

    /// <summary>Called once just before this app is popped from the stack.</summary>
    void OnExit(UserSession session);

    /// <summary>
    /// Processes one line of user input.
    /// Returns <see cref="ApplicationResult.Exit"/> when the app is done
    /// and should be popped, returning focus to the app below.
    /// </summary>
    ApplicationResult HandleInput(string input, UserSession session);
}

/// <summary>
/// New-style application interface that receives <see cref="IUnitOfWork"/>.
/// Replaces <see cref="IApplication"/> incrementally.
/// </summary>
internal interface IKernelApplication
{
    string PromptPrefix(UserSession session);
    void OnEnter(IUnitOfWork uow);
    void OnExit(IUnitOfWork uow);
    ApplicationResult HandleInput(IUnitOfWork uow, string input);
}
