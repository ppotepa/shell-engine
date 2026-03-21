using System.IO.Compression;

namespace CognitosOs.State;

internal interface IVirtualFileSystem
{
    IEnumerable<string> Ls(string? path);
    bool TryCat(string target, out string content);
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

        if (!File.Exists(_statePath))
        {
            return;
        }

        try
        {
            using var archive = ZipFile.OpenRead(_statePath);
            foreach (var entry in archive.Entries)
            {
                if (!entry.FullName.StartsWith("users/linus/home/", StringComparison.Ordinal))
                {
                    continue;
                }

                var relative = entry.FullName["users/linus/home/".Length..].Trim('/');
                if (relative.Length == 0)
                {
                    continue;
                }

                if (entry.FullName.EndsWith("/", StringComparison.Ordinal))
                {
                    RegisterDirectory(relative);
                    continue;
                }

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

    public IEnumerable<string> Ls(string? path)
    {
        var normalized = Normalize(path);
        if (!_directories.Contains(normalized))
        {
            return Array.Empty<string>();
        }

        var items = new List<string>();
        foreach (var dir in _directories)
        {
            if (dir.Length == 0 || !IsDirectChildOf(dir, normalized))
            {
                continue;
            }

            items.Add($"{SegmentName(dir)}/");
        }

        foreach (var file in _files.Keys)
        {
            if (IsDirectChildOf(file, normalized))
            {
                items.Add(SegmentName(file));
            }
        }

        items.Sort(StringComparer.Ordinal);
        return items;
    }

    public bool TryCat(string target, out string content)
    {
        return _files.TryGetValue(Normalize(target), out content!);
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
        if (normalized.Length == 0)
        {
            error = "invalid path";
            return false;
        }
        RegisterDirectory(normalized);
        error = "";
        return true;
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
        var parent = normalized;
        while (true)
        {
            var slash = parent.LastIndexOf('/');
            if (slash < 0)
            {
                break;
            }

            parent = parent[..slash];
            _directories.Add(parent);
        }

        using var stream = entry.Open();
        using var reader = new StreamReader(stream);
        _files[normalized] = reader.ReadToEnd();
    }

    private void RegisterDirectory(string relativePath)
    {
        var normalized = Normalize(relativePath);
        if (normalized.Length == 0)
        {
            return;
        }

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
        {
            return "";
        }

        return raw.Trim().TrimStart('/').TrimEnd('/');
    }

    private static bool IsDirectChildOf(string candidate, string parent)
    {
        if (parent.Length == 0)
        {
            return !candidate.Contains('/');
        }

        if (!candidate.StartsWith(parent, StringComparison.Ordinal))
        {
            return false;
        }

        if (candidate.Length <= parent.Length + 1 || candidate[parent.Length] != '/')
        {
            return false;
        }

        return candidate[(parent.Length + 1)..].IndexOf('/') < 0;
    }

    private static string SegmentName(string fullPath)
    {
        var slash = fullPath.LastIndexOf('/');
        return slash < 0 ? fullPath : fullPath[(slash + 1)..];
    }
}
