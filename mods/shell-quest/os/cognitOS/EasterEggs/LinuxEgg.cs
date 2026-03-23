using CognitOS.Core;
using CognitOS.Kernel;
using CognitOS.Network;

namespace CognitOS.EasterEggs;

/// <summary>
/// "linux" → "command not found (not yet)"
/// "linux --help" → full quest walkthrough
/// </summary>
internal sealed class LinuxEgg : IShellEasterEgg
{
    public string Trigger => "linux";

    public bool Matches(string command, IReadOnlyList<string> argv)
        => command.Equals("linux", StringComparison.OrdinalIgnoreCase);

    public int Handle(IUnitOfWork uow, string command, string[] argv)
    {
        if (argv.Any(a => a is "--help" or "-h"))
        {
            uow.Out.WriteLine("linux: command not found (not yet)");
            uow.Out.WriteLine("");
            uow.Out.WriteLine("...but since you asked:");
            uow.Out.WriteLine("");
            uow.Out.WriteLine("  1. there are files in ~/linux-0.01/");
            uow.Out.WriteLine("  2. one of them needs to reach nic.funet.fi");
            uow.Out.WriteLine("  3. ftp is how files travel");
            uow.Out.WriteLine("  4. compressed archives are not text");
            uow.Out.WriteLine("  5. the default mode is wrong");
            uow.Out.WriteLine("");
            uow.Out.WriteLine("good luck.");
            return 0;
        }

        uow.Out.WriteLine("linux: command not found (not yet)");
        return 0;
    }
}
