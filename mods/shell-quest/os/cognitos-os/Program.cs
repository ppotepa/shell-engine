using System.Text.Json;
using CognitosOs.Boot;
using CognitosOs.Commands;
using CognitosOs.Core;
using CognitosOs.State;

internal static class Program
{
    public static void Main()
    {
        var statePath = Path.Combine(Environment.CurrentDirectory, "state.obj");
        IMachineStart machineStart = new ZipStateStore(statePath);
        var state = machineStart.LoadOrCreate();

        var commands = new ICommand[]
        {
            new HelpCommand(),
            new LsCommand(),
            new CatCommand(),
            new TopCommand(),
            new PsCommand(),
            new ServicesCommand(),
            new ClearCommand(),
        };

        var fileSystem = new ZipVirtualFileSystem(statePath);
        IOperatingSystem os = new MinixOperatingSystem(state, fileSystem, commands);
        IBootSequence boot = new MinixBootSequence();
        var host = new AppHost(os, machineStart);

        host.EmitBoot(boot);

        string? line;
        while ((line = Console.ReadLine()) != null)
        {
            line = line.TrimEnd('\r', '\n');
            if (string.IsNullOrWhiteSpace(line))
            {
                continue;
            }

            try
            {
                using var doc = JsonDocument.Parse(line);
                var root = doc.RootElement;
                var type = Protocol.GetTypeTag(root);

                if (type == "tick")
                {
                    host.HandleTick((ulong)(root.TryGetProperty("dt_ms", out var dt) && dt.TryGetUInt64(out var ms) ? ms : 0));
                    continue;
                }

                if (type == "resize")
                {
                    var rows = Protocol.GetInt(root, "rows") ?? 40;
                    host.HandleResize(rows);
                    continue;
                }

                if (type == "hello")
                {
                    host.HandleResize(Protocol.GetInt(root, "rows") ?? 40);
                    continue;
                }

                if (type == "key")
                {
                    continue;
                }

                if (type != "submit")
                {
                    continue;
                }

                host.HandleSubmit(Protocol.GetString(root, "line") ?? string.Empty);
                host.ApplyPrompt();
            }
            catch (Exception ex)
            {
                Protocol.Send(new
                {
                    type = "out",
                    lines = new[] { Style.Fg(Style.Error, $"[cognitos-os] parse error: {ex.Message}"), "" }
                });
            }
        }
    }
}
