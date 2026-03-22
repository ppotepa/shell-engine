using CognitOS.Core;
using CognitOS.Kernel;

namespace CognitOS.Commands;

[CognitOS.Framework.Ioc.Command("finger", OsTag = "minix")]
internal sealed class FingerCommand : IKernelCommand
{
    public string Name => "finger";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        if (argv.Length < 2)
        {
            uow.Err.WriteLine("usage: finger <user>");
            return 1;
        }

        var target = argv[1].ToLowerInvariant();

        if (target is "torvalds")
        {
            var plan = uow.Disk.RawRead("/usr/torvalds/.plan") ?? "No plan.";
            uow.Out.WriteLine("Login: torvalds                         Name: Linus B. Torvalds");
            uow.Out.WriteLine("Directory: /usr/torvalds                Shell: /bin/sh");
            uow.Out.WriteLine($"On since {uow.Clock.Now():MMM dd HH:mm} on tty0");
            uow.Out.WriteLine($"Plan:\n{plan}");
            return 0;
        }

        if (target is "ast" or "tanenbaum")
        {
            var plan = uow.Disk.RawRead("/usr/ast/.plan") ?? "No plan.";
            uow.Out.WriteLine("Login: ast                              Name: Andy S. Tanenbaum");
            uow.Out.WriteLine("Directory: /usr/ast                     Shell: /bin/sh");
            uow.Out.WriteLine("On since Sep 15 09:41 on tty1");
            uow.Out.WriteLine($"Plan:\n{plan}");
            return 0;
        }

        if (target is "root")
        {
            uow.Out.WriteLine("Login: root                             Name: Charlie Root");
            uow.Out.WriteLine("Directory: /root                        Shell: /bin/sh");
            uow.Out.WriteLine("Never logged in.");
            return 0;
        }

        uow.Err.WriteLine($"finger: {target}: no such user.");
        return 1;
    }
}
