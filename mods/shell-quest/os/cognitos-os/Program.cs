using System.Text.Json;
using CognitosOs.Boot;
using CognitosOs.Commands;
using CognitosOs.Core;
using CognitosOs.EasterEggs;
using CognitosOs.Framework.Ioc;
using CognitosOs.Framework.Kernel;
using CognitosOs.Minix;
using CognitosOs.Network;
using CognitosOs.State;

internal static class Program
{
    public static void Main()
    {
        var statePath = Path.Combine(Environment.CurrentDirectory, "state.obj");
        IMachineStart machineStart = new ZipStateStore(statePath);
        var state = machineStart.LoadOrCreate();

        var container = new ServiceContainer();
        var network = new NetworkRegistry();
        container.RegisterInstance(network);

        var historyCmd = new HistoryCommand();
        container.RegisterInstance(historyCmd);

        var eggs = new EasterEggRegistry();
        eggs.Register(new MinixEgg());
        eggs.Register(new LinuxEgg());
        eggs.Register(new OneLiners());
        IBootSequence boot = new MinixBootSequence();
        AppHost? host = null;
        var initialized = false;

        string? line;
        while ((line = Console.ReadLine()) != null)
        {
            line = line.TrimEnd('\r', '\n');
            if (string.IsNullOrWhiteSpace(line)) continue;

            try
            {
                using var doc = JsonDocument.Parse(line);
                var root = doc.RootElement;
                var type = Protocol.GetTypeTag(root);

                if (type == "tick")
                {
                    if (!initialized) continue;
                    host!.HandleTick((ulong)(root.TryGetProperty("dt_ms", out var dt) && dt.TryGetUInt64(out var ms) ? ms : 0));
                    continue;
                }

                if (type == "resize")
                {
                    if (!initialized) continue;
                    var cols = Protocol.GetInt(root, "cols") ?? 120;
                    var rows = Protocol.GetInt(root, "rows") ?? 40;
                    host!.HandleResize(cols, rows);
                    continue;
                }

                if (type == "hello")
                {
                    var cols = Protocol.GetInt(root, "cols") ?? 120;
                    var rows = Protocol.GetInt(root, "rows") ?? 40;

                    var difficultyLabel = Protocol.GetString(root, "difficulty");
                    var difficulty = MachineSpec.ParseLabel(difficultyLabel);
                    state.Spec = MachineSpec.FromDifficulty(difficulty);

                    var fileSystem = new ZipVirtualFileSystem(statePath);
                    var module = new MinixModule(state.Spec, fileSystem, network);
                    var osScope = container.LoadModule(module);
                    var kernel = osScope.Resolve<IKernel>();
                    var commandIndex = CommandScanner.BuildCommandIndex(osScope, "minix");

                    host = new AppHost(
                        kernel, state, machineStart, eggs, historyCmd, commandIndex,
                        reloadVfs: () => fileSystem.ReloadFromStateArchive());

                    host.HandleResize(cols, rows);

                    var bootScene = Protocol.GetBool(root, "boot_scene") ?? false;
                    if (bootScene)
                        host.EmitBoot(boot);
                    else
                        host.StartAtLogin();

                    initialized = true;
                    continue;
                }

                if (type == "key") continue;

                if (type == "set-input")
                {
                    if (!initialized) continue;
                    host!.HandleInputChange(Protocol.GetString(root, "text") ?? string.Empty);
                    continue;
                }

                if (type != "submit") continue;
                if (!initialized) continue;

                host!.HandleSubmit(Protocol.GetString(root, "line") ?? string.Empty);
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
