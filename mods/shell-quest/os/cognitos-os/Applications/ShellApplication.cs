using CognitosOs.Commands;
using CognitosOs.Core;
using CognitosOs.Framework.Execution;
using CognitosOs.State;

namespace CognitosOs.Applications;

/// <summary>
/// The base shell application. Always sits at the bottom of the application stack.
/// Handles command dispatch including builtins, easter eggs, and launching child applications.
/// </summary>
internal sealed class ShellApplication : IKernelApplication
{
    private readonly IExecutionPipeline _pipeline;

    public ShellApplication(
        IExecutionPipeline pipeline)
    {
        _pipeline = pipeline;
    }

    public string PromptPrefix(UserSession session)
    {
        var user = session.User;
        var host = session.Hostname;
        var cwd = session.DisplayCwd();
        return $"{Style.Fg(Style.PromptUser, user)}@{Style.Fg(Style.PromptHost, host)}:{Style.Fg(Style.PromptPath, cwd)} [{session.LastExitCode}]$ ";
    }

    public void OnEnter(CognitosOs.Kernel.IUnitOfWork uow) { }
    public void OnExit(CognitosOs.Kernel.IUnitOfWork uow) { }

    public ApplicationResult HandleInput(CognitosOs.Kernel.IUnitOfWork uow, string input)
        => _pipeline.Execute(uow, input);
}
