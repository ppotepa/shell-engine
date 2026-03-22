using CognitosOs.Core;
using CognitosOs.Kernel;

namespace CognitosOs.Commands;

internal sealed class UnameCommand : IKernelCommand
{
    public string Name => "uname";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        var flags = new HashSet<char>();
        foreach (var arg in argv.Skip(1))
        {
            if (arg.StartsWith('-') && arg.Length > 1)
                foreach (var c in arg[1..])
                    flags.Add(c);
        }

        if (flags.Contains('a'))
            flags.UnionWith(new[] { 's', 'n', 'r', 'v', 'm' });

        if (flags.Count == 0)
        {
            uow.Out.WriteLine("MINIX");
            return 0;
        }

        var parts = new List<string>();
        if (flags.Contains('s')) parts.Add("MINIX");
        if (flags.Contains('n')) parts.Add("kruuna");
        if (flags.Contains('r')) parts.Add("1.1");
        if (flags.Contains('v')) parts.Add("#1 Sep 17 1991");
        if (flags.Contains('m')) parts.Add("i386");
        if (flags.Contains('p')) parts.Add(uow.Spec.CpuModel);

        uow.Out.WriteLine(string.Join(" ", parts));
        return 0;
    }
}
