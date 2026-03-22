namespace CognitosOs.Kernel.Disk;

using CognitosOs.State;

/// <summary>
/// Disk subsystem. Wraps <see cref="VirtualFileSystem"/> with timing,
/// buffer cache, FD allocation, and disk space accounting.
/// </summary>
internal interface IDisk
{
    /// <summary>Read file contents. Delays: FD alloc → cache check → (miss: seek+transfer) → close FD.</summary>
    string ReadFile(string path);

    /// <summary>Write file. Delays: disk space check → seek+transfer → invalidate cache.</summary>
    void WriteFile(string path, string content);

    /// <summary>Append to file. Creates if missing.</summary>
    void AppendFile(string path, string content);

    /// <summary>List directory entries. Each entry incurs a small read delay.</summary>
    IReadOnlyList<string> ReadDir(string path);

    /// <summary>Stat a file. May hit cache.</summary>
    FileStat Stat(string path);

    /// <summary>Create a directory.</summary>
    void Mkdir(string path);

    /// <summary>Delete a file. Frees disk space.</summary>
    void Unlink(string path);

    /// <summary>Check existence without full stat cost.</summary>
    bool Exists(string path);

    /// <summary>Raw file read — no timing, for internal use (boot, VFS seed).</summary>
    string? RawRead(string path);

    /// <summary>Raw directory list — no timing.</summary>
    IReadOnlyList<string>? RawReadDir(string path);
}
