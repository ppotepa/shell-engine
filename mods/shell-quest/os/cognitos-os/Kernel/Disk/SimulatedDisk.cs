namespace CognitosOs.Kernel.Disk;

using CognitosOs.Framework.Kernel;
using CognitosOs.Kernel.Hardware;
using CognitosOs.Kernel.Resources;
using CognitosOs.State;

/// <summary>
/// Simulated disk operations with realistic timing.
/// All I/O goes through <see cref="ISyscallGate"/> for unified latency + resource management.
/// </summary>
internal sealed class SimulatedDisk : IDisk
{
    private readonly IMutableFileSystem _storage;
    private readonly ResourceState _res;
    private readonly HardwareProfile _hw;
    private readonly ISyscallGate _gate;

    public SimulatedDisk(IMutableFileSystem storage, ResourceState res, HardwareProfile hw, ISyscallGate gate)
    {
        _storage = storage;
        _res = res;
        _hw = hw;
        _gate = gate;
    }

    public string ReadFile(string path)
    {
        if (!_storage.TryCat(path, out string content))
            throw new FileNotFoundException(path);

        int sizeKb = Math.Max(1, (content.Length + 1023) / 1024);

        if (!_res.Cache.Lookup(path))
        {
            // Cache miss — disk I/O via gate
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
}
