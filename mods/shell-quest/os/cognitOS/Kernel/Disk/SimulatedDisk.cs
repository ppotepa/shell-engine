namespace CognitOS.Kernel.Disk;

using CognitOS.Framework.Kernel;
using CognitOS.Kernel.Clock;
using CognitOS.Kernel.Hardware;
using CognitOS.Kernel.Resources;
using CognitOS.State;

/// <summary>
/// Simulated disk operations with realistic timing.
/// All I/O goes through <see cref="ISyscallGate"/> for unified latency + resource management.
/// Inode metadata (permissions, ownership, timestamps) lives in <see cref="InodeTable"/>.
/// </summary>
internal sealed class SimulatedDisk : IDisk
{
    private readonly IMutableFileSystem _storage;
    private readonly ResourceState _res;
    private readonly HardwareProfile _hw;
    private readonly ISyscallGate _gate;
    private readonly IClock _clock;
    private readonly InodeTable _inodes;

    public SimulatedDisk(IMutableFileSystem storage, ResourceState res, HardwareProfile hw,
                         ISyscallGate gate, IClock clock)
    {
        _storage = storage;
        _res = res;
        _hw = hw;
        _gate = gate;
        _clock = clock;
        _inodes = new InodeTable();
    }

    public string ReadFile(string path)
    {
        if (!_storage.TryCat(path, out string content))
            throw new FileNotFoundException(path);

        int sizeKb = Math.Max(1, (content.Length + 1023) / 1024);

        if (!_res.Cache.Lookup(path))
        {
            _gate.Dispatch(
                SyscallRequest.For(SyscallKind.DiskRead, sizeKb * 1024L),
                () => _res.Cache.Insert(path, sizeKb)
            ).ThrowIfFailed();
        }

        return content;
    }

    public void WriteFile(string path, string content)
    {
        int sizeKb = Math.Max(1, (content.Length + 1023) / 1024);

        int oldSizeKb = 0;
        if (_storage.TryCat(path, out string existing))
            oldSizeKb = Math.Max(1, (existing.Length + 1023) / 1024);

        int deltaKb = sizeKb - oldSizeKb;

        var result = _gate.Dispatch(
            SyscallRequest.For(SyscallKind.DiskWrite, sizeKb * 1024L),
            () =>
            {
                _storage.TryWrite(path, content, out _);

                if (deltaKb > 0)
                    _res.Ram.ConsumeDisk(deltaKb);
                else if (deltaKb < 0)
                    _res.Ram.ReleaseDisk(-deltaKb);

                _res.Cache.Invalidate(path);

                // Update inode mtime (or create default inode for new files)
                var key = NormalizeKey(path);
                _inodes.Touch(key, _clock.Now());
            });

        if (!result.Success)
            throw new IOException("No space left on device");
    }

    public void AppendFile(string path, string content)
    {
        _storage.TryCat(path, out string existing);
        WriteFile(path, (existing ?? "") + content);
    }

    public IReadOnlyList<string> ReadDir(string path)
    {
        var entries = _storage.Ls(path).ToList();
        if (entries.Count == 0 && !_storage.DirectoryExists(path))
            throw new DirectoryNotFoundException(path);

        string cacheKey = "dir:" + path;
        if (!_res.Cache.Lookup(cacheKey))
        {
            _gate.Dispatch(
                SyscallRequest.For(SyscallKind.DiskListDir, entries.Count),
                () => _res.Cache.Insert(cacheKey, Math.Max(1, entries.Count / 5))
            ).ThrowIfFailed();
        }

        return entries;
    }

    public FileStat Stat(string path)
    {
        string cacheKey = "stat:" + path;
        if (!_res.Cache.Lookup(cacheKey))
        {
            _gate.Dispatch(
                SyscallRequest.For(SyscallKind.DiskStat),
                () => _res.Cache.Insert(cacheKey, 1)
            ).ThrowIfFailed();
        }

        var key = NormalizeKey(path);
        bool isDir = _storage.DirectoryExists(path);
        bool isFile = _storage.TryCat(path, out string content);
        if (!isDir && !isFile) throw new FileNotFoundException(path);

        int size = isFile ? content.Length : 512;

        var inode = _inodes.Get(key);
        if (inode is not null)
            return new FileStat(inode.Mode, inode.Nlinks, inode.Owner, inode.Group, size, inode.Mtime);

        // Fallback: derive metadata from path prefix (for unseeded paths)
        return _storage.GetStat(path) ?? throw new FileNotFoundException(path);
    }

    public void Mkdir(string path)
    {
        if (!_res.Ram.CheckDiskFree(1))
            throw new IOException("No space left on device");

        _gate.Dispatch(
            SyscallRequest.For(SyscallKind.DiskMkdir),
            () =>
            {
                _storage.TryMkdir(path, out _);
                _res.Ram.ConsumeDisk(1);
                _res.Cache.Invalidate("dir:" + System.IO.Path.GetDirectoryName(path));

                var key = NormalizeKey(path);
                _inodes.CreateDir(key, _clock.Now());
            }).ThrowIfFailed();
    }

    public void Unlink(string path)
    {
        if (!_storage.TryCat(path, out string content))
            throw new FileNotFoundException(path);

        int sizeKb = Math.Max(1, (content.Length + 1023) / 1024);

        _gate.Dispatch(
            SyscallRequest.For(SyscallKind.DiskUnlink),
            () =>
            {
                _storage.TryDelete(path);
                _res.Ram.ReleaseDisk(sizeKb);
                _res.Cache.Invalidate(path);
                _res.Cache.Invalidate("dir:" + System.IO.Path.GetDirectoryName(path));

                _inodes.Remove(NormalizeKey(path));
            }).ThrowIfFailed();
    }

    public bool Exists(string path) =>
        _storage.TryCat(path, out _) || _storage.DirectoryExists(path);

    public string? RawRead(string path) =>
        _storage.TryCat(path, out string content) ? content : null;

    public IReadOnlyList<string>? RawReadDir(string path)
    {
        if (!_storage.DirectoryExists(path)) return null;
        return _storage.Ls(path).ToList();
    }

    public void Chmod(string path, string mode)
    {
        var key = NormalizeKey(path);
        _inodes.Chmod(key, mode);
        _res.Cache.Invalidate("stat:" + path);
    }

    /// <summary>
    /// Convert an absolute or relative path to the normalized VFS key
    /// used as the inode table key (same format as ZipVirtualFileSystem internal keys).
    /// </summary>
    private static string NormalizeKey(string path)
        => path.Trim().TrimStart('/').TrimEnd('/');
}
