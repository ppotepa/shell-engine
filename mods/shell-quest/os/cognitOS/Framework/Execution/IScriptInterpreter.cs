namespace CognitOS.Framework.Execution;

using CognitOS.Kernel;

/// <summary>
/// Executes shell scripts found on the virtual file system.
/// The MINIX prologue uses a deliberately tiny subset of sh semantics.
/// </summary>
internal interface IScriptInterpreter
{
    bool CanExecute(IUnitOfWork uow, string commandName);
    int Execute(IUnitOfWork uow, string[] argv);
}
