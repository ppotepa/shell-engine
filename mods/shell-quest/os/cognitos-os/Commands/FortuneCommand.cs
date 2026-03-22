using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class FortuneCommand : ICommand
{
    public string Name => "fortune";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    private static readonly string[] Fortunes =
    {
        "\"Real programmers don't use Pascal.\" -- Unknown",
        "\"The number of bugs in any program is at least one more.\" -- Lubarsky",
        "RFC 1149: A Standard for the Transmission of IP Datagrams on Avian Carriers.",
        "\"I'd rather write programs to write programs than write programs.\" -- Dick Sites",
        "\"Unix is user-friendly. It's just picky about who its friends are.\" -- Anonymous",
        "\"There are only two kinds of languages: the ones people complain about and the ones nobody uses.\" -- Stroustrup",
        "\"In theory, there is no difference between theory and practice. In practice, there is.\" -- Yogi Berra",
        "\"Those who don't understand UNIX are condemned to reinvent it, poorly.\" -- Henry Spencer",
        "\"Simplicity is prerequisite for reliability.\" -- Dijkstra",
        "\"xK#9fZ!m@2vL&w*0...Q\" -- /dev/random",
    };

    private static readonly string SpookyFortune =
        "\"The best programs are the ones that haven't been written yet.\" -- ????";

    public CommandResult Execute(CommandContext ctx)
    {
        var anomalyCount = ctx.Os.State.Quest.AnomaliesDiscovered?.Count ?? 0;

        // ~10% chance spooky after anomalies
        if (anomalyCount >= 2 && Random.Shared.Next(10) == 0)
            return new CommandResult(new[] { SpookyFortune });

        var pick = Fortunes[Random.Shared.Next(Fortunes.Length)];
        return new CommandResult(new[] { pick });
    }
}
