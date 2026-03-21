using System.IO.Compression;

namespace CognitosOs.State;

internal interface IVirtualFileSystem
{
    IEnumerable<string> Ls(string? path);
    bool TryCat(string target, out string content);
    bool DirectoryExists(string path);

    /// <summary>
    /// Converts an absolute path (e.g. /home/linus/linux-0.01) to a VFS-relative
    /// key (e.g. linux-0.01). Paths outside /home/linus pass through stripped of
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
}

internal sealed class ZipVirtualFileSystem : IMutableFileSystem
{
    private const string HomeAbsolute = "/home/linus";

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
                    if (!entry.FullName.StartsWith("users/linus/home/", StringComparison.Ordinal))
                        continue;

                    var relative = entry.FullName["users/linus/home/".Length..].Trim('/');
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
