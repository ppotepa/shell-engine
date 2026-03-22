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
            // finger with no args: list all logged-in users from SessionManager
            foreach (var s in uow.Sessions.GetSessions())
            {
                if (s.IsAnomaly && uow.Quest.UploadSuccess) continue;
                var name = string.IsNullOrEmpty(s.User) ? "(unknown)" : s.User;
                var entry = uow.Users.GetUser(s.User ?? "");
                var gecos = entry?.Gecos ?? "";
                uow.Out.WriteLine($"{name,-12} {gecos,-28} {s.Tty,-6} {s.LoginTime:MMM dd HH:mm}");
            }
            return 0;
        }

        var target = argv[1].ToLowerInvariant();

        // Look up user in /etc/passwd via UserDatabase
        var user = uow.Users.GetUser(target);
        if (user is null)
        {
            uow.Err.WriteLine($"finger: {target}: no such user.");
            return 1;
        }

        // Check if they have an active session
        var session = uow.Sessions.GetSessions().FirstOrDefault(s => s.User == target);

        uow.Out.WriteLine($"Login: {user.Login,-36} Name: {user.Gecos}");
        uow.Out.WriteLine($"Directory: {user.Home,-28} Shell: {user.Shell}");

        if (session is not null)
            uow.Out.WriteLine($"On since {session.LoginTime:MMM dd HH:mm} on {session.Tty}");
        else
            uow.Out.WriteLine("Never logged in.");

        // Read .plan from VFS
        var plan = uow.Disk.RawRead($"{user.Home}/.plan");
        if (!string.IsNullOrEmpty(plan))
            uow.Out.WriteLine($"Plan:\n{plan}");
        else
            uow.Out.WriteLine("No plan.");

        return 0;
    }
}
