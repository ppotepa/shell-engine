using CognitOS.Core;
using CognitOS.Kernel;
using CognitOS.State;

namespace CognitOS.Network;

/// <summary>
/// Easter egg interface — checked after CommandIndex lookup fails,
/// before emitting "command not found".
/// </summary>
internal interface IEasterEgg
{
    string Trigger { get; }
    bool Matches(string command, IReadOnlyList<string> argv);

    /// <summary>
    /// Handle the easter egg. Write output to <c>uow.Out</c>.
    /// Return exit code.
    /// </summary>
    int Handle(IUnitOfWork uow, string command, string[] argv);
}

/// <summary>
/// Registry of all easter eggs. Checked by ShellApplication on unknown commands.
/// </summary>
internal sealed class EasterEggRegistry
{
    private readonly List<IEasterEgg> _eggs = new();

    public void Register(IEasterEgg egg) => _eggs.Add(egg);

    /// <summary>
    /// Try to handle an unknown command as an easter egg.
    /// Returns null if no egg matched, or the exit code if one did.
    /// </summary>
    public int? TryHandle(IUnitOfWork uow, string command, string[] argv)
    {
        foreach (var egg in _eggs)
        {
            if (egg.Matches(command, argv))
                return egg.Handle(uow, command, argv);
        }
        return null;
    }
}
