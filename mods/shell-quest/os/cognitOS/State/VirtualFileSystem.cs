using System.IO.Compression;

namespace CognitOS.State;

internal interface IVirtualFileSystem
{
    IEnumerable<string> Ls(string? path);
    bool TryCat(string target, out string content);
    bool DirectoryExists(string path);
    FileStat? GetStat(string path);

    /// <summary>
    /// Converts an absolute path (e.g. /usr/linus/linux-0.01) to a VFS-relative
    /// key (e.g. linux-0.01). Paths outside /usr/linus pass through stripped of
    /// the leading slash.
    /// </summary>
    string ToVfsPath(string absolutePath);
}

/// <summary>
/// Extends <see cref="IVirtualFileSystem"/> with write operations.
/// </summary>
internal interface IMutableFileSystem : IVirtualFileSystem
{
    bool TryCopy(string source, string dest, out string error);
    bool TryWrite(string path, string content, out string error);
    bool TryMkdir(string path, out string error);
    bool TryDelete(string path);
}

internal sealed class ZipVirtualFileSystem : IMutableFileSystem
{
    private const string HomeAbsolute = "/usr/torvalds";

    private readonly string _statePath;
    private readonly Dictionary<string, string> _files = new(StringComparer.Ordinal);
    private readonly HashSet<string> _directories = new(StringComparer.Ordinal);

    public ZipVirtualFileSystem(string statePath)
    {
        _statePath = statePath;
        ReloadFromStateArchive();
    }

    public void ReloadFromStateArchive()
    {
        _files.Clear();
        _directories.Clear();
        _directories.Add("");

        if (File.Exists(_statePath))
        {
            try
            {
                using var archive = ZipFile.OpenRead(_statePath);
                foreach (var entry in archive.Entries)
                {
                    if (!entry.FullName.StartsWith("users/torvalds/home/", StringComparison.Ordinal))
                        continue;

                    var relative = entry.FullName["users/torvalds/home/".Length..].Trim('/');
                    if (relative.Length == 0) continue;

                    if (entry.FullName.EndsWith("/", StringComparison.Ordinal))
                        RegisterDirectory(relative);
                    else
                        RegisterFile(relative, entry);
                }
            }
            catch
            {
                _files.Clear();
                _directories.Clear();
                _directories.Add("");
            }
        }

        // Always re-seed epoch files after loading — they are not persisted
        // to the ZIP so they would otherwise vanish after the first Persist/Reload cycle.
        SeedEpochFiles();
    }

    public string ToVfsPath(string absolutePath)
    {
        var trimmed = absolutePath.TrimEnd('/');
        if (trimmed == HomeAbsolute) return "";
        if (trimmed.StartsWith(HomeAbsolute + "/"))
            return trimmed[(HomeAbsolute.Length + 1)..];
        return trimmed.TrimStart('/');
    }

    public bool DirectoryExists(string path)
        => _directories.Contains(Normalize(path));

    public IEnumerable<string> Ls(string? path)
    {
        var normalized = Normalize(path);
        if (!_directories.Contains(normalized))
            return Array.Empty<string>();

        var items = new List<string>();
        foreach (var dir in _directories)
        {
            if (dir.Length == 0 || !IsDirectChildOf(dir, normalized)) continue;
            items.Add($"{SegmentName(dir)}/");
        }
        foreach (var file in _files.Keys)
        {
            if (IsDirectChildOf(file, normalized))
                items.Add(SegmentName(file));
        }

        items.Sort(StringComparer.Ordinal);
        return items;
    }

    public bool TryCat(string target, out string content)
        => _files.TryGetValue(Normalize(target), out content!);

    public FileStat? GetStat(string path)
    {
        var normalized = Normalize(path);
        bool isDir = _directories.Contains(normalized);
        bool isFile = _files.ContainsKey(normalized);
        if (!isDir && !isFile) return null;

        var epoch = new DateTime(1991, 9, 17, 21, 0, 0);

        // Ownership by path prefix
        string owner = "torvalds", group = "other";
        if (normalized.StartsWith("etc") || normalized.StartsWith("var") ||
            normalized.StartsWith("proc") || normalized.StartsWith("dev") ||
            normalized.StartsWith("bin") || normalized.StartsWith("usr/bin") ||
            normalized.StartsWith("usr/lib") || normalized.StartsWith("usr/man") ||
            normalized.StartsWith("usr/src"))
        {
            owner = "root";
            group = "operator";
        }
        else if (normalized.StartsWith("usr/ast"))
        {
            owner = "ast";
        }
        else if (normalized.StartsWith("tmp"))
        {
            // tmp files keep various owners — default linus
            group = "other";
            if (normalized.Contains("ast")) owner = "ast";
        }

        string perms;
        int links;
        int size;
        if (isDir)
        {
            perms = "drwxr-xr-x";
            links = 2;
            size = 512;
        }
        else
        {
            perms = owner == "root" ? "-rw-r--r--" : "-rw-rw-r--";
            if (normalized.StartsWith("etc/rc") || normalized.EndsWith(".sh"))
                perms = "-rwxr-xr-x";
            links = 1;
            size = _files[normalized].Length;
        }

        return new FileStat(perms, links, owner, group, size, epoch);
    }

    public bool TryCopy(string source, string dest, out string error)
    {
        var src = Normalize(source);
        if (!_files.TryGetValue(src, out var content))
        {
            error = $"{source}: No such file or directory";
            return false;
        }
        var dst = Normalize(dest);
        _files[dst] = content;
        RegisterParentDirectories(dst);
        error = "";
        return true;
    }

    public bool TryWrite(string path, string content, out string error)
    {
        var normalized = Normalize(path);
        _files[normalized] = content;
        RegisterParentDirectories(normalized);
        error = "";
        return true;
    }

    public bool TryMkdir(string path, out string error)
    {
        var normalized = Normalize(path);
        if (normalized.Length == 0) { error = "invalid path"; return false; }
        RegisterDirectory(normalized);
        error = "";
        return true;
    }

    public bool TryDelete(string path)
    {
        var normalized = Normalize(path);
        return _files.Remove(normalized);
    }

    /// <summary>
    /// Seeds the prologue epoch files into the in-memory filesystem.
    /// These represent Linus's working tree in September 1991.
    /// Called after every ReloadFromStateArchive so they are never lost.
    /// </summary>
    public void SeedEpochFiles()
    {
        TryMkdir("linux-0.01", out _);

        TryWrite("linux-0.01/RELNOTES-0.01",
@"RELEASE NOTES for Linux v0.01
==============================

This is a free MINIX clone. It is NOT portable (uses 386 task
switching etc), and it probably never will support anything
other than AT-hard disks, as that's all I have :-(.

It's mostly in C, but most people wouldn't call what I write C.
It uses every conceivable feature of the 386 I could find, as
it was a project to teach me about the 386.

linus", out _);

        TryWrite("linux-0.01/linux-0.01.tar.Z",
            "[COMPRESSED ARCHIVE — 73091 bytes — tar.Z format]", out _);

        TryWrite("linux-0.01/bash.Z",
            "[COMPRESSED — Bourne Again Shell binary for MINIX]", out _);

        TryWrite("linux-0.01/update.Z",
            "[COMPRESSED — update daemon binary]", out _);

        TryWrite("linux-0.01/README",
@"Linux version 0.01 — September 1991

This directory contains the source for Linux v0.01.
To build, you need MINIX-386 with gcc installed.

Files:
  linux-0.01.tar.Z   kernel source archive
  bash.Z             shell binary
  update.Z           update daemon
  RELNOTES-0.01      release notes

To upload to nic.funet.fi:
  ftp nic.funet.fi
  cd /pub/OS/Linux
  binary
  put linux-0.01.tar.Z", out _);

        // --- Mail ---
        TryMkdir("mail", out _);

        TryWrite("mail/welcome.txt",
@"From: Operator <op@kruuna>
Subject: Welcome

you made it in. good.
read the notes when you get a chance.", out _);

        TryWrite("mail/ast.txt",
@"From: ast@cs.vu.nl (Andy Tanenbaum)
Date: Tue, 16 Sep 1991 14:22:00 +0200
Subject: re: uploading to funet

Linus,

If you're uploading to funet, remember: compressed
archives must transfer in binary mode. ASCII will
corrupt them. I've seen students make this mistake
every semester.

    -- ast", out _);

        // --- Notes ---
        TryMkdir("notes", out _);

        TryWrite("notes/starter.txt",
@"- type ls to look around
- type cat mail/welcome.txt to read your mail
- try man to read manual pages
- type ps to see running processes", out _);

        // --- History (hint: previous user tried ascii) ---
        TryWrite(".sh_history",
@"ls
cd linux-0.01
ls -la
cat RELNOTES-0.01
ftp nic.funet.fi
ascii
put linux-0.01.tar.Z
quit", out _);

        TryWrite(".profile",
@"# ~/.profile — sourced by /bin/sh on login
export PATH=/bin:/usr/bin
export TERM=minix
export EDITOR=ed", out _);

        TryWrite(".plan",
            "working on something. will post to comp.os.minix when ready.", out _);

        // --- /etc ---
        TryMkdir("etc", out _);

        TryWrite("etc/passwd",
@"root:x:0:0:Charlie Root:/root:/bin/sh
daemon:x:1:1:System Daemon:/usr/sbin:/bin/false
bin:x:2:2:Binary:/bin:/bin/false
ast:x:100:10:Andy S. Tanenbaum:/usr/ast:/bin/sh
torvalds:x:101:10:Linus B. Torvalds:/usr/torvalds:/bin/sh
nobody:x:65534:65534:Nobody:/nonexistent:/bin/false", out _);

        TryWrite("etc/hostname", "kruuna", out _);

        TryWrite("etc/hosts",
@"127.0.0.1       localhost kruuna
128.214.6.100   nic.funet.fi ftp.funet.fi
130.37.24.3     cs.vu.nl
128.214.1.1     helsinki.fi", out _);

        TryWrite("etc/services",
@"ftp        21/tcp
telnet     23/tcp
smtp       25/tcp
http       80/tcp
finger     79/tcp", out _);

        TryWrite("etc/resolv.conf", "nameserver 128.214.1.1", out _);

        TryWrite("etc/rc",
@"#!/bin/sh
# /etc/rc — system initialization
/usr/bin/update &
/usr/bin/cron &
/etc/getty tty0 &", out _);

        TryWrite("etc/motd",
@"MINIX 1.1  kruuna.helsinki.fi
University of Helsinki, Department of Computer Science
", out _);

        TryWrite("etc/group",
@"wheel:x:0:root
daemon:x:1:daemon
bin:x:2:bin
staff:x:10:ast,torvalds
operator:x:5:torvalds
nobody:x:65534:", out _);

        // --- /usr/ast ---
        TryMkdir("usr/ast", out _);

        TryWrite("usr/ast/README",
@"MINIX is for teaching, not production.
If you want to write a real OS, start from scratch.
Good luck.
    -- ast, 1991", out _);

        TryWrite("usr/ast/.plan",
@"Working on MINIX 2.0 filesystem improvements.
Teaching OS course next semester.
Conference paper due October.", out _);

        TryWrite("usr/ast/minix-2.0-notes.txt",
@"MINIX 2.0 design notes (draft)
==============================
- new virtual filesystem layer
- improved memory manager
- POSIX.1 compliance (partial)
- target: spring 1992

TODO:
- benchmark vs SunOS 4.1
- test on 286 (dropped?)", out _);

        // --- /tmp ---
        TryMkdir("tmp", out _);

        TryWrite("tmp/thesis-FINAL-v3-REAL.bak",
            "[binary file -- cannot display]", out _);

        TryWrite("tmp/core",
            "[core dump -- process 'cc1' signal 11 (segmentation fault)]", out _);

        TryWrite("tmp/nroff-err.log",
@"nroff: warning: can't find font 'HR'
nroff: warning: can't break line 47
nroff: 2 warnings, 0 errors (but it looks wrong anyway)", out _);

        TryWrite("tmp/.Xauthority",
            "[binary file -- cannot display]", out _);

        TryWrite("tmp/.lock-ast", "", out _);

        // --- /usr/adm (MINIX log directory, not /var/log) ---
        TryMkdir("usr/adm", out _);

        TryWrite("usr/adm/messages",
@"Sep 17 21:00:01 kruuna kernel: MINIX 1.1 boot
Sep 17 21:00:01 kruuna kernel: memory: 4096K total, 109K kernel, 3987K free
Sep 17 21:00:02 kruuna init: system startup
Sep 17 21:00:02 kruuna cron: started
Sep 17 21:00:03 kruuna getty: tty0 ready
Sep 17 21:12:00 kruuna login: torvalds logged in on tty0
Sep 17 21:12:00 kruuna login: session opened for ast on tty1
Sep 17 21:12:00 kruuna login: session opened for (unknown) on tty2", out _);

        TryWrite("usr/adm/cron",
@"Sep 17 21:00:02 cron: started
Sep 17 21:02:00 cron: /usr/lib/atrun
Sep 17 21:04:00 cron: /usr/lib/atrun", out _);

        TryWrite("usr/adm/wtmp",
            "[binary file -- login accounting records]", out _);

        // --- /proc ---
        TryMkdir("proc", out _);

        TryWrite("proc/version",
            "Minix version 1.1 (Sep 17 1991) gcc 1.37.1", out _);

        // --- /dev (virtual) ---
        TryMkdir("dev", out _);

        TryWrite("dev/null",    "", out _);
        TryWrite("dev/console", "[device -- cannot read directly]", out _);
        TryWrite("dev/tty0",    "[tty device -- torvalds, active]", out _);
        TryWrite("dev/tty1",    "[tty device -- ast, idle]", out _);
        TryWrite("dev/tty2",    "[tty device -- (unknown), login Jan 1 00:00]", out _);
        TryWrite("dev/hd0",     "[block device -- winchester disk]", out _);
        TryWrite("dev/hd1",     "[block device -- root partition]", out _);
        TryWrite("dev/hd2",     "[block device -- /usr partition]", out _);
        TryWrite("dev/modem",   "[char device -- RS-232 serial]", out _);
        TryWrite("dev/mem",     "[block device -- physical memory]", out _);

        // --- /usr/bin, /usr/lib, /usr/man, /usr/src ---
        TryMkdir("usr/bin", out _);
        TryMkdir("usr/lib", out _);
        TryMkdir("usr/man", out _);
        TryMkdir("usr/src", out _);
        TryMkdir("usr/src/minix", out _);

        // /usr/src/minix is not readable (permission denied handled by cat)
        TryMkdir("bin", out _);
    }

    private void RegisterParentDirectories(string normalizedPath)
    {
        var parent = normalizedPath;
        while (true)
        {
            var slash = parent.LastIndexOf('/');
            if (slash < 0) break;
            parent = parent[..slash];
            _directories.Add(parent);
        }
    }

    private void RegisterFile(string relativePath, ZipArchiveEntry entry)
    {
        var normalized = Normalize(relativePath);
        RegisterParentDirectories(normalized);
        using var stream = entry.Open();
        using var reader = new StreamReader(stream);
        _files[normalized] = reader.ReadToEnd();
    }

    private void RegisterDirectory(string relativePath)
    {
        var normalized = Normalize(relativePath);
        if (normalized.Length == 0) return;

        var parts = normalized.Split('/', StringSplitOptions.RemoveEmptyEntries);
        var current = "";
        foreach (var part in parts)
        {
            current = current.Length == 0 ? part : $"{current}/{part}";
            _directories.Add(current);
        }
    }

    private static string Normalize(string? raw)
    {
        if (string.IsNullOrWhiteSpace(raw) || raw is "." or "~" or "/" or "./")
            return "";
        return raw.Trim().TrimStart('/').TrimEnd('/');
    }

    private static bool IsDirectChildOf(string candidate, string parent)
    {
        if (parent.Length == 0)
            return !candidate.Contains('/');
        if (!candidate.StartsWith(parent, StringComparison.Ordinal))
            return false;
        if (candidate.Length <= parent.Length + 1 || candidate[parent.Length] != '/')
            return false;
        return candidate[(parent.Length + 1)..].IndexOf('/') < 0;
    }

    private static string SegmentName(string fullPath)
    {
        var slash = fullPath.LastIndexOf('/');
        return slash < 0 ? fullPath : fullPath[(slash + 1)..];
    }
}
