namespace CognitosOs.Kernel.Resources;

/// <summary>
/// Per-file LRU buffer cache. Tracks cached file paths and their sizes.
/// MaxKb is dynamically set by <see cref="ResourceState.Recalc"/> based on free RAM.
/// </summary>
internal sealed class BufferCache
{
    private readonly LinkedList<CacheEntry> _lru = new();
    private readonly Dictionary<string, LinkedListNode<CacheEntry>> _index = new();

    public int MaxKb { get; set; }
    public int UsedKb { get; private set; }
    public int Hits { get; private set; }
    public int Misses { get; private set; }
    public int EntryCount => _index.Count;

    public BufferCache(int maxKb)
    {
        MaxKb = maxKb;
    }

    /// <summary>
    /// Look up a path in the cache.
    /// On HIT: promotes entry to MRU, increments Hits, returns true.
    /// On MISS: increments Misses, returns false.
    /// </summary>
    public bool Lookup(string key)
    {
        if (_index.TryGetValue(key, out var node))
        {
            _lru.Remove(node);
            _lru.AddLast(node);
            Hits++;
            return true;
        }

        Misses++;
        return false;
    }

    /// <summary>
    /// Insert a file into cache. Evicts LRU entries if necessary.
    /// If single entry exceeds MaxKb, skip caching (file too large).
    /// </summary>
    public void Insert(string key, int sizeKb)
    {
        if (sizeKb <= 0) return;

        // Already cached — update size
        if (_index.TryGetValue(key, out var existing))
        {
            UsedKb -= existing.Value.SizeKb;
            _lru.Remove(existing);
            _index.Remove(key);
        }

        // Single entry larger than entire cache — don't cache
        if (sizeKb > MaxKb) return;

        // Evict LRU until room
        while (UsedKb + sizeKb > MaxKb && _lru.First is not null)
        {
            var victim = _lru.First!;
            _lru.RemoveFirst();
            _index.Remove(victim.Value.Key);
            UsedKb -= victim.Value.SizeKb;
        }

        var entry = new CacheEntry(key, sizeKb);
        var node = _lru.AddLast(entry);
        _index[key] = node;
        UsedKb += sizeKb;
    }

    /// <summary>Invalidate a cache entry (e.g. after file write).</summary>
    public void Invalidate(string key)
    {
        if (_index.TryGetValue(key, out var node))
        {
            UsedKb -= node.Value.SizeKb;
            _lru.Remove(node);
            _index.Remove(key);
        }
    }

    /// <summary>Shrink cache to <paramref name="newMaxKb"/> by evicting LRU entries.</summary>
    public void ShrinkTo(int newMaxKb)
    {
        MaxKb = newMaxKb;
        while (UsedKb > MaxKb && _lru.First is not null)
        {
            var victim = _lru.First!;
            _lru.RemoveFirst();
            _index.Remove(victim.Value.Key);
            UsedKb -= victim.Value.SizeKb;
        }
    }

    private readonly record struct CacheEntry(string Key, int SizeKb);
}
