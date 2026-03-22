using CognitOS.Core;
using CognitOS.Kernel;
using CognitOS.Network;

namespace CognitOS.EasterEggs;

/// <summary>
/// Stateful: silent twice, "minix: I know." on 3rd call, then silent forever.
/// </summary>
internal sealed class MinixEgg : IEasterEgg
{
    private int _count;
    public string Trigger => "minix";

    public bool Matches(string command, IReadOnlyList<string> argv)
        => command.Equals("minix", StringComparison.OrdinalIgnoreCase) && argv.Count == 0;

    public int Handle(IUnitOfWork uow, string command, string[] argv)
    {
        _count++;
        if (_count == 3)
            uow.Out.WriteLine("minix: I know.");
        return 0;
    }
}
