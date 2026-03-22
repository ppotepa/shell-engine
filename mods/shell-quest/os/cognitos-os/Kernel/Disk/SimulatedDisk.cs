namespace CognitosOs.Kernel.Disk;

using CognitosOs.Kernel.Hardware;
using CognitosOs.Kernel.Resources;
using CognitosOs.State;

/// <summary>
/// Simulated disk operations with realistic timing.
/// Every read/write goes through cache + disk controller + hardware profile delays.
/// </summary>
internal sealed class SimulatedDisk : IDisk
{
    private readonly IMutableFileSystem _storage;
    private readonly ResourceState _res;
    private readonly HardwareProfile _hw;

    public SimulatedDisk(IMutableFileSystem storage, ResourceState res, HardwareProfile hw)
    {
        _storage = storage;
        _res = res;
        _hw = hw;
    }

    public string ReadFile(string path)
    {
        if (!_storage.TryCat(path, out string content))
            throw new FileNotFoundException(path);

        int sizeKb = Math.Max(1, (content.Length + 1023) / 1024);

        if (!_res.Cache.Lookup(path))
        {
            // Cache miss — disk I/O
            double contention = _res.DiskCtrl.Acquire();
            double transferMs = _hw.DiskTransferMs(sizeKb);
            _hw.BlockFor(_hw.DiskAccessMs + transferMs + contention + _res.Cpu.OverheadMs());
            _res.DiskCtrl.Release();
            _res.Cache.Insert(path, sizeKb);
        }
        else
        {
            // Cache hit — minimal CPU overhead only
            _hw.BlockFor(_res.Cpu.OverheadMs());
        }

        return content;
    }

    public void WriteFile(string path, string content)
    {
        int sizeKb = Math.Max(1, (content.Length + 1023) / 1024);

        // Check existing file size for delta
        int oldSizeKb = 0;
        if (_storage.TryCat(path, out string existing))
            oldSizeKb = Math.Max(1, (existing.Length + 1023) / 1024);

        int deltaKb = sizeKb - oldSizeKb;

        if (deltaKb > 0 && !_res.Ram.CheckDiskFree(deltaKb))
            throw new IOException("No space left on device");

        double contention = _res.DiskCtrl.Acquire();
        double transferMs = _hw.DiskTransferMs(sizeKb);
        _hw.BlockFor(_hw.DiskAccessMs + transferMs + contention + _res.Cpu.OverheadMs());
        _res.DiskCtrl.Release();

        _storage.TryWrite(path, content, out _);

        if (deltaKb > 0)
            _res.Ram.ConsumeDisk(deltaKb);
        else if (deltaKb < 0)
            _res.Ram.ReleaseDisk(-deltaKb);

        _res.Cache.Invalidate(path);
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
            double contention = _res.DiskCtrl.Acquire();
            _hw.BlockFor(_hw.DiskAccessMs + contention + _res.Cpu.OverheadMs());
            _res.DiskCtrl.Release();

            // Per-entry delay for reading directory entries
            _hw.BlockFor(entries.Count * _hw.DiskDirEntryMs);
            _res.Cache.Insert(cacheKey, Math.Max(1, entries.Count / 5));
        }

        return entries;
    }

    public FileStat Stat(string path)
    {
        string cacheKey = "stat:" + path;
        if (!_res.Cache.Lookup(cacheKey))
        {
            double contention = _res.DiskCtrl.Acquire();
            _hw.BlockFor(_hw.DiskSeekMs + contention + _res.Cpu.OverheadMs());
            _res.DiskCtrl.Release();
            _res.Cache.Insert(cacheKey, 1);
        }

        return _storage.GetStat(path) ?? throw new FileNotFoundException(path);
    }

    public void Mkdir(string path)
    {
        if (!_res.Ram.CheckDiskFree(1))
            throw new IOException("No space left on device");

        double contention = _res.DiskCtrl.Acquire();
        _hw.BlockFor(_hw.DiskAccessMs + contention);
        _res.DiskCtrl.Release();

        _storage.TryMkdir(path, out _);
        _res.Ram.ConsumeDisk(1);
        _res.Cache.Invalidate("dir:" + System.IO.Path.GetDirectoryName(path));
    }

    public void Unlink(string path)
    {
        if (!_storage.TryCat(path, out string content))
            throw new FileNotFoundException(path);

        int sizeKb = Math.Max(1, (content.Length + 1023) / 1024);

        double contention = _res.DiskCtrl.Acquire();
        _hw.BlockFor(_hw.DiskSeekMs + contention);
        _res.DiskCtrl.Release();

        _storage.TryDelete(path);
        _res.Ram.ReleaseDisk(sizeKb);
        _res.Cache.Invalidate(path);
        _res.Cache.Invalidate("dir:" + System.IO.Path.GetDirectoryName(path));
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
