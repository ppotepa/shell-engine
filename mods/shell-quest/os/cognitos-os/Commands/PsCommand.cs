using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class PsCommand : ICommand
{
    public string Name => "ps";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        bool showAll = false, showNoTty = false, longFmt = false;
        foreach (var arg in ctx.Argv)
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
                            return new CommandResult(new[]
                            {
                                $"ps: illegal option -- {c}",
                                "Try: man ps"
                            }, 1);
                    }
                }
            }
        }

        var user = ctx.Session.User;
        var procs = ctx.Os.ProcessSnapshot().OrderBy(p => p.Pid).ToList();

        // Filter: default = only user's processes with a tty
        if (!showAll && !showNoTty)
            procs = procs.Where(p => p.User == user && p.Tty != "?").ToList();
        else if (showAll && !showNoTty)
            procs = procs.Where(p => p.Tty != "?").ToList();
        // showNoTty includes everything; showAll+showNoTty = everything

        var lines = new List<string>();

        if (longFmt)
        {
            lines.Add("  F S   UID   PID  PPID  PGRP    SZ TTY      TIME CMD");
            foreach (var p in procs)
            {
                lines.Add(string.Format("{0,3} {1} {2,5} {3,5} {4,5} {5,5} {6,5} {7,-4} {8,9} {9}",
                    1, p.StateCh, p.Uid, p.Pid, p.Ppid, p.Pid, p.Sz,
                    p.Tty, FormatTime(p.Pid), p.Name));
            }
        }
        else
        {
            lines.Add("  PID TTY      TIME CMD");
            foreach (var p in procs)
            {
                lines.Add(string.Format("{0,5} {1,-4} {2,9} {3}",
                    p.Pid, p.Tty, FormatTime(p.Pid), p.Name));
            }
        }

        return new CommandResult(lines);
    }

    private static string FormatTime(int pid)
    {
        // Deterministic fake CPU time based on pid
        var mins = pid % 10;
        var secs = (pid * 7) % 60;
        return $"0:{mins:D2}:{secs:D2}";
    }
}
