namespace CognitosOs.Framework.Ioc;

internal interface IOperatingSystemModule
{
    string Name { get; }
    void Register(ServiceContainer container);
}
