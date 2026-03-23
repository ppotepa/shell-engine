using System.Text.Json;
using System.Net;
using System.Net.Sockets;
using CognitOS.Boot;
using CognitOS.Commands;
using CognitOS.Core;
using CognitOS.EasterEggs;
using CognitOS.Framework.Ioc;
using CognitOS.Framework.Kernel;
using CognitOS.Framework.Transport;
using CognitOS.Minix;
using CognitOS.Network;
using CognitOS.State;

internal static class Program
{
    public static void Main()
    {
        var transport = CreateTransport(Environment.GetCommandLineArgs().Skip(1).ToArray());
        Console.SetOut(new GameTextWriter(transport.Output));

        var statePath = Path.Combine(Environment.CurrentDirectory, "state.obj");
        IMachineStart machineStart = new ZipStateStore(statePath);
        var state = machineStart.LoadOrCreate();

        var container = new ServiceContainer();
        var hostIndex = RemoteHostIndex.Build();
        container.RegisterInstance(hostIndex);

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
        while ((line = transport.Input.ReadProtocolLine()) != null)
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
                    var module = new MinixModule(state.Spec, fileSystem, hostIndex);
                    var osScope = container.LoadModule(module);
                    var kernel = osScope.Resolve<IKernel>();
                    var commandIndex = CommandScanner.BuildCommandIndex(osScope, "minix");

                    host = new AppHost(
                        kernel, state, machineStart, transport.Output, eggs, historyCmd, commandIndex,
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
                Protocol.Send(transport.Output, new
                {
                    type = "out",
                    lines = new[] { Style.Fg(Style.Error, $"[cognitOS] parse error: {ex.Message}"), "" }
                });
            }
        }
    }

    private static TransportContext CreateTransport(string[] args)
    {
        if (args.Length >= 2 && args[0] == "--game-port" && int.TryParse(args[1], out var port))
            return TransportContext.CreateTcp(port);

        return TransportContext.CreateConsole(Console.In, Console.Out);
    }

    private sealed class TransportContext : IDisposable
    {
        public IInputSource Input { get; }
        public IOutputSink Output { get; }
        private readonly TcpClient? _client;
        private readonly TcpListener? _listener;

        private TransportContext(IInputSource input, IOutputSink output, TcpClient? client = null, TcpListener? listener = null)
        {
            Input = input;
            Output = output;
            _client = client;
            _listener = listener;
        }

        public static TransportContext CreateConsole(TextReader input, TextWriter output)
        {
            return new TransportContext(
                new ConsoleInputSource(input),
                new ConsoleOutputSink(output));
        }

        public static TransportContext CreateTcp(int port)
        {
            var listener = new TcpListener(IPAddress.Loopback, port);
            listener.Start();
            var client = listener.AcceptTcpClient();
            var stream = client.GetStream();
            return new TransportContext(
                new TcpInputSource(stream),
                new TcpOutputSink(stream),
                client,
                listener);
        }

        public void Dispose()
        {
            Input.Dispose();
            Output.Dispose();
            _client?.Dispose();
            _listener?.Stop();
        }
    }
}
