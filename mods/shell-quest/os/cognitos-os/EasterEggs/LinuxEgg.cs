using CognitosOs.Core;
using CognitosOs.Network;

namespace CognitosOs.EasterEggs;

/// <summary>
/// "linux" → "command not found (not yet)"
/// "linux --help" → full quest walkthrough
/// </summary>
internal sealed class LinuxEgg : IEasterEgg
{
    public string Trigger => "linux";

    public bool Matches(string command, IReadOnlyList<string> argv)
        => command.Equals("linux", StringComparison.OrdinalIgnoreCase);

    public CommandResult Handle(string fullInput, CommandContext ctx)
    {
        if (ctx.Argv.Any(a => a is "--help" or "-h"))
        {
            return new CommandResult(new[]
            {
                "linux: command not found (not yet)",
                "",
                "...but since you asked:",
                "",
                "  1. there are files in ~/linux-0.01/",
                "  2. one of them needs to reach nic.funet.fi",
                "  3. ftp is how files travel",
                "  4. compressed archives are not text",
                "  5. the default mode is wrong",
                "",
                "good luck.",
            });
        }

        return new CommandResult(new[] { "linux: command not found (not yet)" });
    }
}
