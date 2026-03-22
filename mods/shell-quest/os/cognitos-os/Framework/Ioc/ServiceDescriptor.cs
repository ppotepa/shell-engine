namespace CognitosOs.Framework.Ioc;

internal sealed record ServiceDescriptor(
    Type ServiceType,
    ServiceLifetime Lifetime,
    Func<ServiceContainer, object> Factory);
