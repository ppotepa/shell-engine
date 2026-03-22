namespace CognitosOs.Framework.Execution;

using CognitosOs.Core;
using CognitosOs.Kernel;

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
