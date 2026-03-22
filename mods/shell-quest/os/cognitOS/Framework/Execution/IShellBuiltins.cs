namespace CognitOS.Framework.Execution;

using CognitOS.Core;
using CognitOS.Kernel;

/// <summary>
/// OS-specific shell builtins handled before external command lookup.
/// Examples: cd, pwd, exit, export, set.
/// </summary>
internal interface IShellBuiltins
{
    /// <summary>
    /// Try to execute a shell builtin.
    /// Returns true when the builtin handled the input.
    /// </summary>
    bool TryHandle(IUnitOfWork uow, string[] argv, out ApplicationResult result);
}
