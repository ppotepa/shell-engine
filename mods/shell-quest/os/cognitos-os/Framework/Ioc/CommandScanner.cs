namespace CognitosOs.Framework.Ioc;

using System.Reflection;
using CognitosOs.Commands;
using CognitosOs.Core;

internal static class CommandScanner
{
    /// Returns all types in the current assembly decorated with [CommandAttribute].
    /// Filtered by osTag — "universal" always included.
    public static IEnumerable<(CommandAttribute Attr, Type Type)> Scan(string osTag)
    {
        return Assembly.GetExecutingAssembly()
            .GetTypes()
            .Where(t => !t.IsAbstract && !t.IsInterface)
            .Select(t => (Attr: t.GetCustomAttribute<CommandAttribute>(), CommandType: t))
            .Where(x => x.Attr is not null)
            .Select(x => (Attr: x.Attr!, Type: x.CommandType))
            .Where(x => x.Attr.OsTag == osTag || x.Attr.OsTag == "universal");
    }

    public static IReadOnlyDictionary<string, IKernelCommand> BuildCommandIndex(ServiceContainer container, string osTag)
    {
        var index = new Dictionary<string, IKernelCommand>(StringComparer.Ordinal);

        foreach (var (_, type) in Scan(osTag))
        {
            foreach (var command in CreateCommands(container, type))
            {
                index[command.Name] = command;
                foreach (var alias in command.Aliases)
                    index[alias] = command;
            }
        }

        return index;
    }

    private static IEnumerable<IKernelCommand> CreateCommands(ServiceContainer container, Type type)
    {
        if (!typeof(IKernelCommand).IsAssignableFrom(type))
            yield break;

        if (type == typeof(HeadTailCommand))
        {
            yield return new HeadTailCommand(isHead: true);
            yield return new HeadTailCommand(isHead: false);
            yield break;
        }

        if (type == typeof(HistoryCommand))
        {
            yield return (IKernelCommand)container.Resolve(type);
            yield break;
        }

        yield return (IKernelCommand)container.Construct(type);
    }
}
