using CognitOS.Core;
using CognitOS.Kernel;

namespace CognitOS.Commands;

[CognitOS.Framework.Ioc.Command("ps", OsTag = "minix")]
internal sealed class PsCommand : IKernelCommand
{
    public string Name => "ps";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        bool showAll = false, showNoTty = false, longFmt = false;
        foreach (var arg in argv.Skip(1))
        {
            if (arg.StartsWith('-') && arg.Length > 1)
            {
                foreach (var c in arg[1..])
                {
                    switch (c)
                    {
                        case 'a': showAll = true; break;
                        case 'x': showNoTty = true; break;
                        case 'l': longFmt = true; break;
                        default:
                            uow.Err.WriteLine($"ps: illegal option -- {c}");
                            uow.Err.WriteLine("Try: man ps");
                            return 1;
                    }
                }
            }
        }

        var user = uow.Session.User;
        var procs = uow.Process.List().OrderBy(p => p.Pid).ToList();

        if (!showAll && !showNoTty)
            procs = procs.Where(p => p.User == user && p.Tty != "?").ToList();
        else if (showAll && !showNoTty)
            procs = procs.Where(p => p.Tty != "?").ToList();

        if (longFmt)
        {
            uow.Out.WriteLine("  F S   UID   PID  PPID  PGRP    SZ TTY      TIME CMD");
            foreach (var p in procs)
                uow.Out.WriteLine(string.Format("{0,3} {1} {2,5} {3,5} {4,5} {5,5} {6,5} {7,-4} {8,9} {9}",
                    1, p.StateCh, p.Uid, p.Pid, p.Ppid, p.Pid, p.Sz,
                    p.Tty, FormatTime(p.Pid), p.Name));
        }
        else
        {
            uow.Out.WriteLine("  PID TTY      TIME CMD");
            foreach (var p in procs)
                uow.Out.WriteLine(string.Format("{0,5} {1,-4} {2,9} {3}",
                    p.Pid, p.Tty, FormatTime(p.Pid), p.Name));
        }

        return 0;
    }

    private static string FormatTime(int pid)
    {
        // Deterministic CPU time based on PID — small processes accumulate little
        var secs = pid * 3 % 60;
        var mins = pid / 4 % 60;
        return $"{mins}:{secs:D2}";
    }
}
