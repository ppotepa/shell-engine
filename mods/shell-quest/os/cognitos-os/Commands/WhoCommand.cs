using CognitosOs.Core;
using CognitosOs.Kernel;

namespace CognitosOs.Commands;

[CognitosOs.Framework.Ioc.Command("who", OsTag = "minix")]
internal sealed class WhoCommand : IKernelCommand
{
    public string Name => "who";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        var now = uow.Clock.Now();
        uow.Out.WriteLine($"linus    tty0     {now:MMM dd HH:mm}");
        uow.Out.WriteLine("ast      tty1     Sep 15 09:41");

        if (!uow.Quest.UploadSuccess)
            uow.Out.WriteLine("         tty2     Jan  1 00:00");

        return 0;
    }
}
