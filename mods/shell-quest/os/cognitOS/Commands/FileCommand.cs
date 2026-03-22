using CognitOS.Core;
using CognitOS.Kernel;

namespace CognitOS.Commands;

[CognitOS.Framework.Ioc.Command("file", OsTag = "minix")]
internal sealed class FileCommand : IKernelCommand
{
    public string Name => "file";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        if (argv.Length < 2)
        {
            uow.Err.WriteLine("usage: file <path>");
            return 1;
        }

        var userPath = argv[1];
        var path = uow.Session.ResolvePath(userPath);

        var raw = uow.Disk.RawRead(path);
        if (raw == null)
        {
            // Maybe a directory?
            var dir = uow.Disk.RawReadDir(path);
            if (dir != null)
            {
                uow.Out.WriteLine($"{userPath}: directory");
                return 0;
            }
            uow.Err.WriteLine($"{userPath}: cannot open");
            return 1;
        }

        var type = userPath switch
        {
            _ when userPath.EndsWith(".tar.Z") => "compressed data (compress'd)",
            _ when userPath.EndsWith(".Z") => "compressed data",
            _ when userPath.EndsWith(".tar") => "POSIX tar archive",
            _ when raw.StartsWith("[COMPRESSED") => "compressed data",
            _ when raw.StartsWith("[binary") || raw.StartsWith("[core") => "data",
            _ => "ASCII text",
        };

        uow.Out.WriteLine($"{userPath}: {type}");
        return 0;
    }
}
