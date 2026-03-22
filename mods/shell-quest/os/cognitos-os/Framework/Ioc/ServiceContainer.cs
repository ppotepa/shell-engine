namespace CognitosOs.Framework.Ioc;

internal sealed class ServiceContainer
{
    private readonly ServiceContainer? _parent;
    private readonly ServiceLifetime _scopeLifetime;
    private readonly Dictionary<Type, ServiceDescriptor> _descriptors = new();
    private readonly Dictionary<Type, object> _instances = new();

    // Root container
    public ServiceContainer() { _scopeLifetime = ServiceLifetime.Singleton; }

    // Child scope
    private ServiceContainer(ServiceContainer parent, ServiceLifetime scopeLifetime)
    {
        _parent = parent;
        _scopeLifetime = scopeLifetime;
    }

    /// Register with explicit factory
    public ServiceContainer Register<TService>(ServiceLifetime lifetime, Func<ServiceContainer, TService> factory)
        where TService : class
    {
        _descriptors[typeof(TService)] = new ServiceDescriptor(
            typeof(TService), lifetime, c => factory(c));
        return this;
    }

    /// Register TImpl as TService — TImpl must have one public constructor
    public ServiceContainer Register<TService, TImpl>(ServiceLifetime lifetime)
        where TService : class
        where TImpl : class, TService
    {
        _descriptors[typeof(TService)] = new ServiceDescriptor(
            typeof(TService), lifetime, c => c.Construct(typeof(TImpl)));
        return this;
    }

    /// Register singleton instance directly
    public ServiceContainer RegisterInstance<TService>(TService instance)
        where TService : class
    {
        _descriptors[typeof(TService)] = new ServiceDescriptor(
            typeof(TService), ServiceLifetime.Singleton, _ => instance);
        _instances[typeof(TService)] = instance;
        return this;
    }

    public TService Resolve<TService>() where TService : class
        => (TService)Resolve(typeof(TService));

    public object Resolve(Type serviceType)
    {
        // Check own instances first
        if (_instances.TryGetValue(serviceType, out var inst)) return inst;

        // Check own descriptors
        if (_descriptors.TryGetValue(serviceType, out var desc))
        {
            // Singletons and PerOsStage are cached in the container that owns the descriptor
            if (desc.Lifetime is ServiceLifetime.Singleton or ServiceLifetime.PerOsStage)
            {
                var obj = desc.Factory(this);
                _instances[serviceType] = obj;
                return obj;
            }
            // PerSession: always create new, never cache
            return desc.Factory(this);
        }

        // Walk up to parent
        if (_parent is not null) return _parent.Resolve(serviceType);

        throw new InvalidOperationException($"Service not registered: {serviceType.FullName}");
    }

    /// Create a child scope for a new OS stage — inherits singletons from root
    public ServiceContainer CreateOsStageScope() => new(this, ServiceLifetime.PerOsStage);

    /// Create a per-session child scope
    public ServiceContainer CreateSessionScope() => new(this, ServiceLifetime.PerSession);

    /// Construct a type by reflecting its single public constructor and resolving each param
    internal object Construct(Type type)
    {
        var ctors = type.GetConstructors();
        if (ctors.Length == 0)
            throw new InvalidOperationException($"No public constructor on {type.FullName}");
        // Pick the constructor with the most parameters (greedy)
        var ctor = ctors.OrderByDescending(c => c.GetParameters().Length).First();
        var args = ctor.GetParameters()
            .Select(p => Resolve(p.ParameterType))
            .ToArray();
        return ctor.Invoke(args);
    }

    /// Load an OS module — calls module.Register(this) on a new PerOsStage scope
    public ServiceContainer LoadModule(IOperatingSystemModule module)
    {
        var scope = CreateOsStageScope();
        module.Register(scope);
        return scope;
    }
}
