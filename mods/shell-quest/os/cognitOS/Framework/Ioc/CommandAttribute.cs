namespace CognitOS.Framework.Ioc;

[AttributeUsage(AttributeTargets.Class, Inherited = false)]
internal sealed class CommandAttribute : Attribute
{
    public string Name { get; }
    public string OsTag { get; init; } = "minix";
    public string[] Aliases { get; init; } = [];

    public CommandAttribute(string name) => Name = name;
}
