namespace CognitosOs.Framework.Ioc;

internal enum ServiceLifetime
{
    Singleton,   // one per process
    PerOsStage,  // replaced on MINIX→Linux swap
    PerSession,  // one per command execution
}
