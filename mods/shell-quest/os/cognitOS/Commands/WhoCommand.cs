using CognitOS.Core;
using CognitOS.Kernel;

namespace CognitOS.Commands;

[CognitOS.Framework.Ioc.Command("who", OsTag = "minix")]
internal sealed class WhoCommand : IKernelCommand
{
    public string Name => "who";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        foreach (var s in uow.Sessions.GetSessions())
        {
            // anomaly tty2 disappears after upload succeeds
            if (s.IsAnomaly && uow.Quest.UploadSuccess)
                continue;

            var user = string.IsNullOrEmpty(s.User) ? "" : s.User;
            uow.Out.WriteLine($"{user,-8} {s.Tty,-8} {s.LoginTime:MMM dd HH:mm}");
        }
        return 0;
    }
}
