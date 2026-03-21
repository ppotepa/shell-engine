using CognitosOs.State;

namespace CognitosOs.Core;

internal sealed class MinixOperatingSystem : IOperatingSystem
{
    private static readonly DateTime Epoch = new(1991, 9, 17, 21, 12, 0, DateTimeKind.Utc);

    public MachineState State { get; }
    public MachineSpec Spec => State.Spec;
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
        if (State.Processes.Count == 0)
        {
            State.Processes = BuildDefaultProcesses();
        }
        if (State.Services.Count == 0)
        {
            State.Services = BuildDefaultServices();
        }
        if (State.MailMessages.Count == 0)
        {
            State.MailMessages = BuildDefaultMail();
            State.UnreadMailCount = State.MailMessages.Count(m => !m.IsRead);
        }
    }

    public void Tick(ulong dtMs)
    {
        State.UptimeMs = State.UptimeMs + dtMs;
        UpdateProcessTable();
        TickServices();
    }

    public DateTime SimulatedNow()
        => Epoch.AddMilliseconds(State.UptimeMs);

    public (double CpuPercent, double MemoryPercent) UsageSnapshot()
    {
        var cpu = State.Processes.Sum(p => p.CpuPercent);
        var mem = State.Processes.Sum(p => p.MemoryPercent);
        return (cpu, mem);
    }

    public IReadOnlyList<ProcessEntry> ProcessSnapshot()
        => State.Processes;

    public IReadOnlyList<ServiceEntry> ServiceSnapshot()
        => State.Services;

    public int UnreadMailCount()
        => State.UnreadMailCount;

    public void MarkMailRead(string targetPath)
    {
        var normalized = targetPath.Trim().TrimStart('/').ToLowerInvariant();
        if (!normalized.StartsWith("mail/"))
        {
            return;
        }

        var fileName = normalized["mail/".Length..];
        var match = State.MailMessages.FirstOrDefault(m => m.FileName.Equals(fileName, StringComparison.OrdinalIgnoreCase));
        if (match is null || match.IsRead)
        {
            return;
        }

        match.IsRead = true;
        State.UnreadMailCount = State.MailMessages.Count(m => !m.IsRead);
    }

    private void UpdateProcessTable()
    {
        var t = State.UptimeMs / 1000.0;
        foreach (var process in State.Processes)
        {
            var phase = process.Pid / 10.0;
            process.CpuPercent = process.Name == "shell"
                ? 1.0 + Math.Abs(Math.Sin((t + phase) / 2.0)) * 22.0
                : 0.2 + Math.Abs(Math.Sin((t + phase) / 5.0)) * 3.0;
            process.MemoryPercent = process.Name == "shell"
                ? 3.0 + Math.Abs(Math.Cos((t + phase) / 3.0)) * 1.6
                : 1.0 + Math.Abs(Math.Cos((t + phase) / 7.0)) * 1.4;
            process.State = process.CpuPercent > 1.4 ? "running" : "sleeping";
        }
    }

    private static List<ProcessEntry> BuildDefaultProcesses()
        => new()
        {
            new ProcessEntry { Pid = 1, Name = "init", User = "root", State = "running", CpuPercent = 0.5, MemoryPercent = 2.1 },
            new ProcessEntry { Pid = 17, Name = "netd", User = "root", State = "sleeping", CpuPercent = 1.1, MemoryPercent = 1.8 },
            new ProcessEntry { Pid = 21, Name = "maild", User = "root", State = "sleeping", CpuPercent = 0.6, MemoryPercent = 1.4 },
            new ProcessEntry { Pid = 42, Name = "shell", User = "linus", State = "running", CpuPercent = 2.4, MemoryPercent = 3.2 },
        };

    private void TickServices()
    {
        foreach (var service in State.Services)
        {
            service.LastTickUtc = SimulatedNow();
        }

        var minute = State.UptimeMs / 60000;
        if (minute > 0 && minute % 2 == 0 && State.MailMessages.All(m => m.FileName != $"mail-{minute:000}.txt"))
        {
            var fileName = $"mail-{minute:000}.txt";
            State.MailMessages.Add(new MailMessage
            {
                FileName = fileName,
                Content =
                    "From: netd@kruuna\n" +
                    $"Subject: heartbeat {minute}\n\n" +
                    "network link stable.\n",
                IsRead = false,
            });
            State.UnreadMailCount = State.MailMessages.Count(m => !m.IsRead);
        }
    }

    private static List<ServiceEntry> BuildDefaultServices()
        => new()
        {
            new ServiceEntry { Name = "netd", Status = "active" },
            new ServiceEntry { Name = "maild", Status = "active" },
            new ServiceEntry { Name = "cron", Status = "active" },
        };

    private static List<MailMessage> BuildDefaultMail()
        => new()
        {
            new MailMessage
            {
                FileName = "welcome.txt",
                Content =
                    "From: Operator <op@kruuna>\n" +
                    "Subject: Welcome\n\n" +
                    "you made it in. good.\n" +
                    "read the notes when you get a chance.\n",
                IsRead = false,
            },
        };
}
