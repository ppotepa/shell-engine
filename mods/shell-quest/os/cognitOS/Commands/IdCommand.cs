using CognitOS.Core;
using CognitOS.Kernel;

namespace CognitOS.Commands;

[CognitOS.Framework.Ioc.Command("id", OsTag = "minix")]
internal sealed class IdCommand : IKernelCommand
{
    public string Name => "id";
    public IReadOnlyList<string> Aliases => new[] { "groups" };

    public int Run(IUnitOfWork uow, string[] argv)
    {
        var login = uow.Session.User;
        var entry = uow.Users.GetUser(login);

        if (argv[0] == "groups")
        {
            var groupNames = uow.Users.GetGroupsForUser(login);
            uow.Out.WriteLine(string.Join(" ", groupNames));
            return 0;
        }

        if (entry is null)
        {
            uow.Out.WriteLine($"uid=?(?) gid=(?) groups=(?)");
            return 1;
        }

        var primaryGroup = uow.Users.GetGroup(entry.Gid);
        var groups = uow.Users.GetGroupsForUser(login).ToList();
        var gidStr = primaryGroup is not null ? $"{primaryGroup.Gid}({primaryGroup.Name})" : $"{entry.Gid}";
        var groupsStr = string.Join(",", groups.Select(g =>
        {
            var ge = uow.Users.GetGroupByName(g);
            return ge is not null ? $"{ge.Gid}({ge.Name})" : g;
        }));

        uow.Out.WriteLine($"uid={entry.Uid}({entry.Login}) gid={gidStr} groups={groupsStr}");
        return 0;
    }
}
