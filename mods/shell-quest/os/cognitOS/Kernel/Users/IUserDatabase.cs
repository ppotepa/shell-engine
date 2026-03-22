namespace CognitOS.Kernel.Users;

/// <summary>
/// A parsed entry from /etc/passwd.
/// </summary>
internal sealed class PasswdEntry
{
    public string Login { get; init; } = "";
    public int Uid { get; init; }
    public int Gid { get; init; }
    public string Gecos { get; init; } = "";
    public string Home { get; init; } = "";
    public string Shell { get; init; } = "/bin/sh";
}

/// <summary>
/// A parsed entry from /etc/group.
/// </summary>
internal sealed class GroupEntry
{
    public string Name { get; init; } = "";
    public int Gid { get; init; }
    public IReadOnlyList<string> Members { get; init; } = Array.Empty<string>();
}

/// <summary>
/// Parses /etc/passwd and /etc/group from VFS on demand.
/// Single source of truth for user/group identity — no hardcoding in commands.
/// </summary>
internal interface IUserDatabase
{
    PasswdEntry? GetUser(string login);
    IEnumerable<PasswdEntry> GetAllUsers();
    GroupEntry? GetGroup(int gid);
    GroupEntry? GetGroupByName(string name);
    IEnumerable<string> GetGroupsForUser(string login);
}
