using CognitosOs.Core;
using CognitosOs.Network;

namespace CognitosOs.EasterEggs;

/// <summary>
/// Stateful: silent twice, "minix: I know." on 3rd call, then silent forever.
/// </summary>
internal sealed class MinixEgg : IEasterEgg
{
    private int _count;
    public string Trigger => "minix";

    public bool Matches(string command, IReadOnlyList<string> argv)
        => command.Equals("minix", StringComparison.OrdinalIgnoreCase) && argv.Count == 0;

    public CommandResult Handle(string fullInput, CommandContext ctx)
    {
        _count++;
        if (_count == 3)
            return new CommandResult(new[] { "minix: I know." });

        return new CommandResult(Array.Empty<string>());
    }
}
