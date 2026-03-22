namespace CognitosOs.Framework.Ioc;

using System.Reflection;

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
}
