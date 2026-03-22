namespace CognitOS.Kernel.Disk;

/// <summary>
/// Per-file metadata (permissions, owner, timestamps, link count).
/// Mirrors a real MINIX 1.1 disk inode — stored separately from file content.
/// </summary>
internal sealed record InodeRecord(
    string Mode,
    string Owner,
    string Group,
    int Nlinks,
    DateTime Mtime,
    DateTime Ctime);

/// <summary>
/// In-memory inode table. Keyed by VFS-normalized path (leading slash stripped,
/// trailing slash stripped, same representation as ZipVirtualFileSystem._files).
///
/// Seeded from the game epoch on construction. Updated by disk operations so
/// that Stat() always returns authoritative metadata — no path-prefix guessing.
/// </summary>
internal sealed class InodeTable
{
    private readonly Dictionary<string, InodeRecord> _inodes =
        new(StringComparer.Ordinal);

    private static readonly DateTime Epoch =
        new(1991, 9, 17, 21, 0, 0, DateTimeKind.Utc);

    private static InodeRecord Dir(string owner, string group, string mode = "drwxr-xr-x")
        => new(mode, owner, group, 2, Epoch, Epoch);

    private static InodeRecord Reg(string owner, string group, string mode = "-rw-r--r--")
        => new(mode, owner, group, 1, Epoch, Epoch);

    public InodeTable() => Seed();

    private void Seed()
    {
        // ── VFS root = /usr/torvalds ──────────────────────────────────────────
        S("",                              Dir("torvalds", "staff",    "drwx--x--x"));

        // ── /etc ─────────────────────────────────────────────────────────────
        S("etc",                           Dir("root", "operator"));
        S("etc/passwd",                    Reg("root", "operator"));
        S("etc/hostname",                  Reg("root", "operator"));
        S("etc/hosts",                     Reg("root", "operator"));
        S("etc/services",                  Reg("root", "operator"));
        S("etc/resolv.conf",               Reg("root", "operator"));
        S("etc/rc",                        Reg("root", "operator",    "-rwxr-xr-x"));
        S("etc/motd",                      Reg("root", "operator"));
        S("etc/group",                     Reg("root", "operator"));

        // ── User tree under /usr/torvalds ─────────────────────────────────────
        S("linux-0.01",                    Dir("torvalds", "staff"));
        S("linux-0.01/RELNOTES-0.01",      Reg("torvalds", "staff",   "-rw-rw-r--"));
        S("linux-0.01/linux-0.01.tar.Z",   Reg("torvalds", "staff",   "-rw-rw-r--"));
        S("linux-0.01/bash.Z",             Reg("torvalds", "staff",   "-rw-rw-r--"));
        S("linux-0.01/update.Z",           Reg("torvalds", "staff",   "-rw-rw-r--"));
        S("linux-0.01/README",             Reg("torvalds", "staff",   "-rw-rw-r--"));
        S("mail",                          Dir("torvalds", "staff",   "drwx------"));
        S("mail/welcome.txt",              Reg("torvalds", "staff",   "-rw-------"));
        S("mail/ast.txt",                  Reg("torvalds", "staff",   "-rw-------"));
        S("notes",                         Dir("torvalds", "staff"));
        S("notes/starter.txt",             Reg("torvalds", "staff",   "-rw-rw-r--"));
        S(".sh_history",                   Reg("torvalds", "staff",   "-rw-------"));
        S(".profile",                      Reg("torvalds", "staff",   "-rw-rw-r--"));
        S(".plan",                         Reg("torvalds", "staff",   "-rw-rw-r--"));

        // ── System directories ────────────────────────────────────────────────
        S("bin",                           Dir("root", "operator"));
        S("usr",                           Dir("root", "operator"));
        S("usr/bin",                       Dir("root", "operator"));
        S("usr/lib",                       Dir("root", "operator"));
        S("usr/man",                       Dir("root", "operator"));
        S("usr/src",                       Dir("root", "operator"));
        S("usr/src/minix",                 Dir("root", "operator",    "drwx------"));
        S("tmp",                           Dir("root", "other",       "drwxrwxrwt"));
        S("proc",                          Dir("root", "operator",    "dr-xr-xr-x"));
        S("dev",                           Dir("root", "operator"));

        // ── /usr/ast ──────────────────────────────────────────────────────────
        S("usr/ast",                       Dir("ast", "staff",        "drwx--x--x"));
        S("usr/ast/README",                Reg("ast", "staff"));
        S("usr/ast/.plan",                 Reg("ast", "staff"));
        S("usr/ast/minix-2.0-notes.txt",   Reg("ast", "staff"));

        // ── /usr/adm ──────────────────────────────────────────────────────────
        S("usr/adm",                       Dir("root", "operator",    "drwxr-x---"));
        S("usr/adm/messages",              Reg("root", "operator",    "-rw-r-----"));
        S("usr/adm/cron",                  Reg("root", "operator",    "-rw-r-----"));
        S("usr/adm/wtmp",                  Reg("root", "operator",    "-rw-r-----"));

        // ── /tmp files ────────────────────────────────────────────────────────
        S("tmp/thesis-FINAL-v3-REAL.bak",  Reg("torvalds", "staff",   "-rw-rw-r--"));
        S("tmp/core",                      Reg("torvalds", "staff",   "-rw-------"));
        S("tmp/nroff-err.log",             Reg("torvalds", "staff",   "-rw-rw-r--"));
        S("tmp/.Xauthority",               Reg("torvalds", "staff",   "-rw-------"));
        S("tmp/.lock-ast",                 Reg("ast",      "staff",   "-rw-------"));

        // ── /proc ─────────────────────────────────────────────────────────────
        S("proc/version",                  Reg("root", "operator",    "-r--r--r--"));

        // ── /dev — character (c) and block (b) devices ────────────────────────
        S("dev/null",    new InodeRecord("crw-rw-rw-", "root",     "operator", 1, Epoch, Epoch));
        S("dev/console", new InodeRecord("crw--w----", "root",     "operator", 1, Epoch, Epoch));
        S("dev/tty0",    new InodeRecord("crw--w----", "torvalds", "operator", 1, Epoch, Epoch));
        S("dev/tty1",    new InodeRecord("crw--w----", "ast",      "operator", 1, Epoch, Epoch));
        S("dev/tty2",    new InodeRecord("crw--w----", "root",     "operator", 1, Epoch, Epoch));
        S("dev/hd0",     new InodeRecord("brw-rw----", "root",     "operator", 1, Epoch, Epoch));
        S("dev/hd1",     new InodeRecord("brw-rw----", "root",     "operator", 1, Epoch, Epoch));
        S("dev/hd2",     new InodeRecord("brw-rw----", "root",     "operator", 1, Epoch, Epoch));
        S("dev/modem",   new InodeRecord("crw-rw----", "root",     "operator", 1, Epoch, Epoch));
        S("dev/mem",     new InodeRecord("crw-r-----", "root",     "operator", 1, Epoch, Epoch));
    }

    private void S(string path, InodeRecord record) => _inodes[path] = record;

    /// <summary>Get inode metadata for a VFS-normalized path.</summary>
    public InodeRecord? Get(string normalizedPath)
        => _inodes.GetValueOrDefault(normalizedPath);

    /// <summary>
    /// Update mtime on write. Creates a default inode if the path is new
    /// (e.g. user created a file we haven't seeded).
    /// </summary>
    public void Touch(string normalizedPath, DateTime mtime)
    {
        if (_inodes.TryGetValue(normalizedPath, out var existing))
            _inodes[normalizedPath] = existing with { Mtime = mtime };
        else
            _inodes[normalizedPath] = new InodeRecord("-rw-rw-r--", "torvalds", "staff", 1, mtime, mtime);
    }

    public void CreateDir(string normalizedPath, DateTime now)
        => _inodes[normalizedPath] = new InodeRecord("drwxr-xr-x", "torvalds", "staff", 2, now, now);

    public void Remove(string normalizedPath)
        => _inodes.Remove(normalizedPath);

    /// <summary>Change the mode string for a path (chmod).</summary>
    public void Chmod(string normalizedPath, string mode)
    {
        if (_inodes.TryGetValue(normalizedPath, out var existing))
            _inodes[normalizedPath] = existing with { Mode = mode };
    }
}
