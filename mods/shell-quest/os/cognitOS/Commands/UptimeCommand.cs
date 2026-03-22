using CognitOS.Core;
using CognitOS.Kernel;

namespace CognitOS.Commands;

[CognitOS.Framework.Ioc.Command("uptime", OsTag = "minix")]
internal sealed class UptimeCommand : IKernelCommand
{
    public string Name => "uptime";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        var now = uow.Clock.Now();
        var uptime = TimeSpan.FromMilliseconds(uow.Clock.UptimeMs());
        var days = (int)uptime.TotalDays;
        var hours = uptime.Hours;
        var minutes = uptime.Minutes;

        var uptimeStr = days > 0
            ? $"{days} day{(days != 1 ? "s" : "")}, {hours:D2}:{minutes:D2}"
            : $"{hours:D2}:{minutes:D2}";

        var load1 = 0.30 + Random.Shared.NextDouble() * 0.25;
        var load5 = 0.25 + Random.Shared.NextDouble() * 0.20;
        var load15 = 0.20 + Random.Shared.NextDouble() * 0.15;

        uow.Out.WriteLine($" {now:HH:mm:ss} up {uptimeStr},  3 users,  load average: {load1:F2}, {load5:F2}, {load15:F2}");
        return 0;
    }
}
