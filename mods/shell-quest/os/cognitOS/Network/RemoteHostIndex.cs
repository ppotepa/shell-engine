using System.Reflection;
using CognitOS.Core;

namespace CognitOS.Network;

/// <summary>
/// Built once at startup by scanning the executing assembly for all types marked
/// with <see cref="RemoteHostAttribute"/> that implement <see cref="IRemoteHost"/>.
/// Keyed case-insensitively by hostname and all aliases.
/// </summary>
internal sealed class RemoteHostIndex
{
    private readonly Dictionary<string, IRemoteHost> _hosts =
        new(StringComparer.OrdinalIgnoreCase);

    private RemoteHostIndex() { }

    public static RemoteHostIndex Build()
    {
        var index = new RemoteHostIndex();
        foreach (var type in Assembly.GetExecutingAssembly().GetTypes())
        {
            if (!typeof(IRemoteHost).IsAssignableFrom(type) || !type.IsClass || type.IsAbstract)
                continue;
            var attr = type.GetCustomAttribute<RemoteHostAttribute>();
            if (attr is null) continue;

            var host = (IRemoteHost)Activator.CreateInstance(type)!;
            index._hosts[attr.Hostname] = host;
            foreach (var alias in attr.Aliases)
                index._hosts[alias] = host;
        }
        return index;
    }

    /// <summary>Resolves a hostname (or alias) to its <see cref="IRemoteHost"/>, or null.</summary>
    public IRemoteHost? Resolve(string hostname)
        => _hosts.GetValueOrDefault(hostname);

    /// <summary>Returns the IP address for a known host, or null.</summary>
    public string? ResolveIp(string hostname)
        => _hosts.TryGetValue(hostname, out var h) ? h.IpAddress : null;

    public bool IsKnown(string hostname) => _hosts.ContainsKey(hostname);

    /// <summary>Simulate a ping RTT with jitter scaled by modem baud rate.</summary>
    public static int JitteredPing(int baseMs, MachineSpec spec)
    {
        if (baseMs <= 0) return 0;
        var factor = 1200.0 / Math.Max(spec.ModemBaud / 8.0, 1);
        var scaled = (int)(baseMs * factor);
        var jitter = Random.Shared.Next(-(scaled / 7), scaled / 7 + 1);
        return Math.Max(1, scaled + jitter);
    }
}
