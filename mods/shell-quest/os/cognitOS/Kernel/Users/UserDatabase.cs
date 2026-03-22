using CognitOS.Kernel.Disk;

namespace CognitOS.Kernel.Users;

/// <summary>
/// Reads /etc/passwd and /etc/group from the VFS.
/// Parses on each call — files are small and this avoids stale caches.
/// </summary>
internal sealed class UserDatabase : IUserDatabase
{
    private readonly IDisk _disk;

    public UserDatabase(IDisk disk)
    {
        _disk = disk;
    }

    public PasswdEntry? GetUser(string login)
        => ParsePasswd().FirstOrDefault(e => e.Login == login);

    public IEnumerable<PasswdEntry> GetAllUsers()
        => ParsePasswd();

    public GroupEntry? GetGroup(int gid)
        => ParseGroup().FirstOrDefault(e => e.Gid == gid);

    public GroupEntry? GetGroupByName(string name)
        => ParseGroup().FirstOrDefault(e => e.Name == name);

    public IEnumerable<string> GetGroupsForUser(string login)
    {
        var user = GetUser(login);
        var groups = ParseGroup();
        foreach (var g in groups)
        {
            // primary group or explicit member
            if ((user is not null && g.Gid == user.Gid) || g.Members.Contains(login))
                yield return g.Name;
        }
    }

    private IEnumerable<PasswdEntry> ParsePasswd()
    {
        var raw = _disk.RawRead("/etc/passwd");
        if (string.IsNullOrEmpty(raw)) yield break;

        foreach (var line in raw.Split('\n', StringSplitOptions.RemoveEmptyEntries))
        {
            if (line.StartsWith('#')) continue;
            var parts = line.Split(':');
            if (parts.Length < 7) continue;
            if (!int.TryParse(parts[2], out var uid)) continue;
            if (!int.TryParse(parts[3], out var gid)) continue;
            yield return new PasswdEntry
            {
                Login = parts[0],
                Uid = uid,
                Gid = gid,
                Gecos = parts[4],
                Home = parts[5],
                Shell = parts[6],
            };
        }
    }

    private IEnumerable<GroupEntry> ParseGroup()
    {
        var raw = _disk.RawRead("/etc/group");
        if (string.IsNullOrEmpty(raw)) yield break;

        foreach (var line in raw.Split('\n', StringSplitOptions.RemoveEmptyEntries))
        {
            if (line.StartsWith('#')) continue;
            var parts = line.Split(':');
            if (parts.Length < 4) continue;
            if (!int.TryParse(parts[2], out var gid)) continue;
            var members = parts[3].Split(',', StringSplitOptions.RemoveEmptyEntries);
            yield return new GroupEntry
            {
                Name = parts[0],
                Gid = gid,
                Members = members,
            };
        }
    }
}
