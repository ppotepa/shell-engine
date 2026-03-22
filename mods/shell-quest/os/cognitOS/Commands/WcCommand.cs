using CognitOS.Core;
using CognitOS.Kernel;

namespace CognitOS.Commands;

[CognitOS.Framework.Ioc.Command("wc", OsTag = "universal")]
internal sealed class WcCommand : IKernelCommand
{
    public string Name => "wc";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        if (argv.Length < 2)
        {
            uow.Err.WriteLine("usage: wc <file>");
            return 1;
        }

        var path = uow.Session.ResolvePath(argv[1]);

        try
        {
            var content = uow.Disk.ReadFile(path);
            var lines = content.Replace("\r\n", "\n").Split('\n').Length;
            var words = content.Split(new[] { ' ', '\n', '\r', '\t' }, StringSplitOptions.RemoveEmptyEntries).Length;
            var bytes = content.Length;

            uow.Out.WriteLine($"  {lines}  {words}  {bytes} {argv[1]}");
            return 0;
        }
        catch (FileNotFoundException)
        {
            uow.Err.WriteLine($"wc: {argv[1]}: No such file or directory");
            return 1;
        }
    }
}
