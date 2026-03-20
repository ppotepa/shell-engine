using CognitosOs.State;

namespace CognitosOs.Core;

internal sealed class MinixOperatingSystem : IOperatingSystem
{
    private static readonly DateTime Epoch = new(1991, 9, 17, 21, 12, 0, DateTimeKind.Utc);

    public MachineState State { get; }
    public IReadOnlyDictionary<string, ICommand> CommandIndex { get; }
    public IVirtualFileSystem FileSystem { get; }

    public MinixOperatingSystem(MachineState state, IVirtualFileSystem fileSystem, IEnumerable<ICommand> commands)
    {
        State = state;
        FileSystem = fileSystem;
        var index = new Dictionary<string, ICommand>(StringComparer.Ordinal);
        foreach (var command in commands)
        {
            index[command.Name] = command;
            foreach (var alias in command.Aliases)
            {
                index[alias] = command;
            }
        }

        CommandIndex = index;
    }

    public void Tick(ulong dtMs)
    {
        State.UptimeMs = State.UptimeMs + dtMs;
    }

    public DateTime SimulatedNow()
        => Epoch.AddMilliseconds(State.UptimeMs);

    public (double CpuPercent, double MemoryPercent) UsageSnapshot()
    {
        var t = State.UptimeMs / 1000.0;
        var cpu = 12.0 + Math.Abs(Math.Sin(t / 4.0)) * 22.0;
        var mem = 28.0 + Math.Abs(Math.Cos(t / 7.0)) * 16.0;
        return (cpu, mem);
    }
}
