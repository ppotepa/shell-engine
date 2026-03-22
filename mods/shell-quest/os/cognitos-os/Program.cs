using System.Text.Json;
using CognitosOs.Boot;
using CognitosOs.Commands;
using CognitosOs.Core;
using CognitosOs.EasterEggs;
using CognitosOs.Network;
using CognitosOs.State;

internal static class Program
{
    public static void Main()
    {
        var statePath = Path.Combine(Environment.CurrentDirectory, "state.obj");
        IMachineStart machineStart = new ZipStateStore(statePath);
        var state = machineStart.LoadOrCreate();

        var network = new NetworkRegistry();
        var historyCmd = new HistoryCommand();

        var commands = new ICommand[]
        {
            new HelpCommand(),
            new LsCommand(),
            new CatCommand(),
            new TopCommand(),
            new PsCommand(),
            new ServicesCommand(),
            new ClearCommand(),
            new CdCommand(),
            new PwdCommand(),
            new CpCommand(),
            new FtpCommand(),
            // Environment
            new DateCommand(),
            new UptimeCommand(),
            new WhoamiCommand(),
            new WhoCommand(),
            new UnameCommand(),
            new HostnameCommand(),
            new FortuneCommand(),
            new EchoCommand(),
            new EnvCommand(),
            historyCmd,
            new IdCommand(),
            // Filesystem
            new DfCommand(),
            new GrepCommand(),
            new HeadTailCommand(isHead: true),
            new HeadTailCommand(isHead: false),
            new WcCommand(),
            new FileCommand(),
            new ManCommand(),
            new FingerCommand(),
            new MountCommand(),
            new FreeCommand(),
            new SyncCommand(),
            new KillCommand(),
            // Network
            new PingCommand(network),
            new NslookupCommand(network),
            new NetstatCommand(),
            new IfconfigCommand(),
            new DmesgCommand(),
        };

        var eggs = new EasterEggRegistry();
        eggs.Register(new MinixEgg());
        eggs.Register(new LinuxEgg());
        eggs.Register(new OneLiners());

        var fileSystem = new ZipVirtualFileSystem(statePath);

        IOperatingSystem os = new MinixOperatingSystem(state, fileSystem, commands);
        IBootSequence boot = new MinixBootSequence();
        var host = new AppHost(os, machineStart, eggs, historyCmd);
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
                    host.HandleTick((ulong)(root.TryGetProperty("dt_ms", out var dt) && dt.TryGetUInt64(out var ms) ? ms : 0));
                    continue;
                }

                if (type == "resize")
                {
                    if (!initialized) continue;
                    var cols = Protocol.GetInt(root, "cols") ?? 120;
                    var rows = Protocol.GetInt(root, "rows") ?? 40;
                    host.HandleResize(cols, rows);
                    continue;
                }

                if (type == "hello")
                {
                    host.HandleResize(
                        Protocol.GetInt(root, "cols") ?? 120,
                        Protocol.GetInt(root, "rows") ?? 40
                    );
                    var difficultyLabel = Protocol.GetString(root, "difficulty");
                    var difficulty = MachineSpec.ParseLabel(difficultyLabel);
                    state.Spec = MachineSpec.FromDifficulty(difficulty);
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
                    host.HandleInputChange(Protocol.GetString(root, "text") ?? string.Empty);
                    continue;
                }

                if (type != "submit") continue;
                if (!initialized) continue;

                host.HandleSubmit(Protocol.GetString(root, "line") ?? string.Empty);
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
