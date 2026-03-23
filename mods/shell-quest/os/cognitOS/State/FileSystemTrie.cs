namespace CognitOS.State;

/// <summary>
/// Trie-based file system for O(k) path lookup and efficient directory traversal.
/// k = path depth (typically 3-5 components)
/// </summary>
internal sealed class FileSystemTrie
{
    private class TrieNode
    {
        public string? FileContent { get; set; }  // Non-null if this node is a file
        public Dictionary<string, TrieNode> Children { get; } = new(StringComparer.Ordinal);
        public bool IsDirectory => FileContent == null && Children.Count > 0;
        public bool Exists => FileContent != null || IsDirectory;
    }

    private readonly TrieNode _root = new();

    /// <summary>
    /// Add or update a file at the given path.
    /// Automatically creates parent directories.
    /// </summary>
    public void SetFile(string path, string content)
    {
        var segments = PathToSegments(path);
        var node = _root;

        foreach (var segment in segments)
        {
            if (!node.Children.ContainsKey(segment))
                node.Children[segment] = new TrieNode();
            node = node.Children[segment];
        }

        node.FileContent = content;
    }

    /// <summary>
    /// Add a directory (ensure parent structure exists).
    /// </summary>
    public void AddDirectory(string path)
    {
        var segments = PathToSegments(path);
        var node = _root;

        foreach (var segment in segments)
        {
            if (!node.Children.ContainsKey(segment))
                node.Children[segment] = new TrieNode();
            node = node.Children[segment];
        }
    }

    /// <summary>
    /// Get file content. Returns null if not a file or doesn't exist.
    /// O(k) where k = path depth
    /// </summary>
    public string? GetFile(string path)
    {
        var node = TraversePath(path);
        return node?.FileContent;
    }

    /// <summary>
    /// Check if path is a directory.
    /// O(k)
    /// </summary>
    public bool IsDirectory(string path)
    {
        var node = TraversePath(path);
        return node?.IsDirectory ?? false;
    }

    /// <summary>
    /// Check if path exists (file or directory).
    /// O(k)
    /// </summary>
    public bool Exists(string path)
    {
        var node = TraversePath(path);
        return node?.Exists ?? false;
    }

    /// <summary>
    /// List direct children of a directory.
    /// Much faster than scanning all paths.
    /// O(m) where m = number of direct children
    /// </summary>
    public IEnumerable<string> ListDirectory(string path)
    {
        var node = TraversePath(path);
        if (node == null || !node.IsDirectory && node.FileContent != null)
            return Array.Empty<string>();

        // List all direct children
        var items = new List<string>();
        foreach (var (name, child) in node.Children)
        {
            if (child.FileContent != null)
                items.Add(name);  // File
            else if (child.IsDirectory)
                items.Add(name + "/");  // Directory
        }

        items.Sort(StringComparer.Ordinal);
        return items;
    }

    /// <summary>
    /// Delete a file or directory.
    /// O(k)
    /// </summary>
    public bool Delete(string path)
    {
        var segments = PathToSegments(path);
        if (segments.Length == 0)
            return false;

        var node = _root;
        var parents = new Stack<(TrieNode node, string segment)>();

        // Traverse to parent
        for (int i = 0; i < segments.Length - 1; i++)
        {
            if (!node.Children.ContainsKey(segments[i]))
                return false;
            parents.Push((node, segments[i]));
            node = node.Children[segments[i]];
        }

        var lastSegment = segments[^1];
        if (!node.Children.ContainsKey(lastSegment))
            return false;

        var target = node.Children[lastSegment];

        // Can't delete non-empty directories
        if (target.IsDirectory && target.Children.Count > 0)
            return false;

        node.Children.Remove(lastSegment);
        return true;
    }

    /// <summary>
    /// Get all files matching a prefix (for directory operations).
    /// Useful for cleanup/migration.
    /// </summary>
    public IEnumerable<(string path, string content)> GetFilesWithPrefix(string prefixPath)
    {
        var segments = PathToSegments(prefixPath);
        var node = TraversePath(prefixPath);
        if (node == null)
            yield break;

        // DFS to find all files under this path
        foreach (var (path, content) in GetFilesRecursive(node, prefixPath))
            yield return (path, content);
    }

    /// <summary>
    /// Traverse a path in the trie.
    /// O(k) where k = path depth
    /// </summary>
    private TrieNode? TraversePath(string path)
    {
        var segments = PathToSegments(path);
        var node = _root;

        foreach (var segment in segments)
        {
            if (!node.Children.TryGetValue(segment, out var child))
                return null;
            node = child;
        }

        return node;
    }

    /// <summary>
    /// Convert path string to segments.
    /// Handles absolute/relative paths, normalization.
    /// </summary>
    private static string[] PathToSegments(string path)
    {
        if (string.IsNullOrWhiteSpace(path) || path is "." or "/" or "./" or "~")
            return Array.Empty<string>();

        var trimmed = path.Trim().TrimStart('/').TrimEnd('/');
        if (trimmed.Length == 0)
            return Array.Empty<string>();

        // Handle home directory mapping
        const string homeRel = "usr/torvalds";
        if (trimmed == homeRel)
            return Array.Empty<string>();
        if (trimmed.StartsWith(homeRel + "/"))
            trimmed = trimmed[(homeRel.Length + 1)..];

        return trimmed.Split('/', StringSplitOptions.RemoveEmptyEntries);
    }

    /// <summary>
    /// Recursively find all files under a path.
    /// </summary>
    private IEnumerable<(string path, string content)> GetFilesRecursive(TrieNode node, string basePath)
    {
        if (node.FileContent != null)
        {
            yield return (basePath, node.FileContent);
        }

        foreach (var (name, child) in node.Children)
        {
            var childPath = basePath.EndsWith('/') ? basePath + name : basePath + "/" + name;
            foreach (var file in GetFilesRecursive(child, childPath))
                yield return file;
        }
    }
}
