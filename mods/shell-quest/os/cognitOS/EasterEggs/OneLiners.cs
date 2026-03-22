using CognitOS.Core;
using CognitOS.Kernel;
using CognitOS.Network;

namespace CognitOS.EasterEggs;

/// <summary>
/// Simple one-liner responses for unrecognized commands that deserve flavor.
/// </summary>
internal sealed class OneLiners : IEasterEgg
{
    public string Trigger => "(multiple)";

    private static readonly Dictionary<string, Func<IUnitOfWork, string?>> Responses = new(StringComparer.OrdinalIgnoreCase)
    {
        ["emacs"] = _ => "emacs: not installed. only vi available on this system.",
        ["vi"] = _ => "vi: insufficient memory",
        ["vim"] = _ => "vim: command not found",
        ["nano"] = _ => "nano: command not found",
        ["rm"] = _ => "rm: permission denied (nice try)",
        ["su"] = uow => uow.Spec.Difficulty == Difficulty.Su
            ? "su: you chose this name, didn't you?"
            : "su: incorrect password",
        ["sudo"] = _ => "sudo: command not found. this is MINIX.",
        ["shutdown"] = _ => "shutdown: must be superuser.",
        ["halt"] = _ => "halt: must be superuser.",
        ["reboot"] = _ => "reboot: must be superuser.",
        ["make"] = _ => "make: no targets. nothing to do.",
        ["gcc"] = _ => "gcc: not installed. try Amsterdam Compiler Kit.",
        ["cc"] = _ => "cc: no input files",
        ["ld"] = _ => "ld: no input files",
        ["exit"] = _ => "logout",
        ["logout"] = _ => "logout",
        ["passwd"] = _ => "passwd: only root may change passwords",
        ["adduser"] = _ => "adduser: permission denied",
        ["useradd"] = _ => "useradd: permission denied",
        ["chmod"] = _ => "chmod: operation not permitted",
        ["chown"] = _ => "chown: must be superuser",
        ["chgrp"] = _ => "chgrp: must be superuser",
        ["init"] = _ => "init: must be run as PID 1",
        ["crontab"] = _ => "crontab: no changes made",
        ["at"] = _ => "at: command scheduling disabled",
        ["nice"] = _ => "nice: permission denied",
        ["renice"] = _ => "renice: permission denied",
        ["sed"] = _ => "sed: not installed",
        ["awk"] = _ => "awk: not installed",
        ["wget"] = _ => "wget: command not found",
        ["curl"] = _ => "curl: command not found",
        ["python"] = _ => "python: command not found",
        ["perl"] = _ => "perl: command not found",
        ["bash"] = _ => "bash: not a standard shell. use /bin/sh",
        ["apt"] = _ => "apt: command not found",
        ["yum"] = _ => "yum: command not found",
        ["git"] = _ => "git: command not found",
        ["ssh"] = _ => "ssh: command not found",
        ["scp"] = _ => "scp: command not found",
        ["alias"] = _ => "alias: not supported in sh",
        ["export"] = _ => "export: read-only environment",
        ["hello"] = _ => null,
    };

    public bool Matches(string command, IReadOnlyList<string> argv)
        => Responses.ContainsKey(command);

    public int Handle(IUnitOfWork uow, string command, string[] argv)
    {
        if (Responses.TryGetValue(command, out var fn))
        {
            var result = fn(uow);
            if (result != null)
                uow.Out.WriteLine(result);
        }
        return 0;
    }
}
