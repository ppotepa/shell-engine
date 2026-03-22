namespace CognitOS.Kernel.Mount;

internal sealed class MountTable : IMountTable
{
    private readonly List<MountEntry> _mounts = new();

    public IReadOnlyList<MountEntry> GetMounts() => _mounts;

    public void AddMount(MountEntry entry)
    {
        _mounts.RemoveAll(m => m.MountPoint == entry.MountPoint);
        _mounts.Add(entry);
    }
}
