using System.Text.Json;
using CognitosOs.Boot;
using CognitosOs.Commands;
using CognitosOs.Core;
using CognitosOs.State;

internal static class Program
{
    public static void Main()
    {
        var statePath = Path.Combine(Environment.CurrentDirectory, ".cognitos-state.json");
        IMachineStart machineStart = new JsonStateStore(statePath);
        var state = machineStart.LoadOrCreate();

        var commands = new ICommand[]
        {
            new HelpCommand(),
            new LsCommand(),
            new CatCommand(),
            new TopCommand(),
            new ClearCommand(),
        };

        IOperatingSystem os = new MinixOperatingSystem(state, new VirtualFileSystem(), commands);
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

                if (type is "hello" or "resize" or "key")
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
