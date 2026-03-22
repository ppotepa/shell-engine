using CognitosOs.Core;
using CognitosOs.State;

namespace CognitosOs.Network;

/// <summary>
/// Easter egg interface — checked after CommandIndex lookup fails,
/// before emitting "command not found".
/// </summary>
internal interface IEasterEgg
{
    string Trigger { get; }
    bool Matches(string command, IReadOnlyList<string> argv);
    CommandResult Handle(string fullInput, CommandContext ctx);
}

/// <summary>
/// Registry of all easter eggs. Checked by ShellApplication on unknown commands.
/// </summary>
internal sealed class EasterEggRegistry
{
    private readonly List<IEasterEgg> _eggs = new();

    public void Register(IEasterEgg egg) => _eggs.Add(egg);

    public CommandResult? TryHandle(string command, IReadOnlyList<string> argv, CommandContext ctx)
    {
        foreach (var egg in _eggs)
        {
            if (egg.Matches(command, argv))
                return egg.Handle($"{command} {string.Join(' ', argv)}".Trim(), ctx);
        }
        return null;
    }
}
