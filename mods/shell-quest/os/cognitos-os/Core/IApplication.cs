namespace CognitosOs.Core;

using CognitosOs.Kernel;

internal enum ApplicationResult
{
    Continue,
    Exit,
}

/// <summary>
/// Represents a running application in the application stack.
/// Input from the keyboard always routes to the topmost app.
/// All interactions receive a fresh <see cref="IUnitOfWork"/> scope.
/// </summary>
internal interface IKernelApplication
{
    string PromptPrefix(UserSession session);
    void OnEnter(IUnitOfWork uow);
    void OnExit(IUnitOfWork uow);
    ApplicationResult HandleInput(IUnitOfWork uow, string input);
}
