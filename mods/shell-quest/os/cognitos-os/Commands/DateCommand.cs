using CognitosOs.Core;
using CognitosOs.Kernel;

namespace CognitosOs.Commands;

internal sealed class DateCommand : IKernelCommand
{
    public string Name => "date";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        var now = uow.Clock.Now();
        var anomalyCount = uow.Quest.AnomaliesDiscovered?.Count ?? 0;

        if (anomalyCount >= 3 && Random.Shared.Next(20) == 0)
        {
            var realNow = DateTime.UtcNow;
            uow.Out.WriteLine(now.ToString("ddd MMM dd HH:mm:ss 'EET' yyyy"));
            uow.Out.WriteLine(realNow.ToString("ddd MMM dd HH:mm:ss 'EET' yyyy"));
            uow.Out.WriteLine(now.ToString("ddd MMM dd HH:mm:ss 'EET' yyyy"));
            return 0;
        }

        uow.Out.WriteLine(now.ToString("ddd MMM dd HH:mm:ss 'EET' yyyy"));
        return 0;
    }
}
