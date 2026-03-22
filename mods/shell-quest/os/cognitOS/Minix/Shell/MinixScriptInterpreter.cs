namespace CognitOS.Minix.Shell;

using CognitOS.Framework.Execution;
using CognitOS.Kernel;

/// <summary>
/// Minimal prologue script support. The abstraction exists now so Linux can
/// replace it later with a richer interpreter without changing shell wiring.
/// </summary>
internal sealed class MinixScriptInterpreter : IScriptInterpreter
{
    public bool CanExecute(IUnitOfWork uow, string commandName)
    {
        var path = ResolveCandidatePath(uow, commandName);
        if (path is null)
            return false;

        var content = uow.Disk.RawRead(path);
        return content is not null && (path.EndsWith(".sh", StringComparison.Ordinal) || content.StartsWith("#!/bin/sh", StringComparison.Ordinal));
    }

    public int Execute(IUnitOfWork uow, string[] argv)
    {
        uow.Out.WriteLine("sh: script execution is not enabled in the prologue yet");
        return 126;
    }

    private static string? ResolveCandidatePath(IUnitOfWork uow, string commandName)
    {
        if (string.IsNullOrWhiteSpace(commandName))
            return null;

        if (commandName.Contains('/'))
            return uow.Session.ResolvePath(commandName);

        var cwdCandidate = uow.Session.ResolvePath(commandName);
        if (uow.Disk.RawRead(cwdCandidate) is not null)
            return cwdCandidate;

        var binCandidate = $"/bin/{commandName}";
        if (uow.Disk.RawRead(binCandidate) is not null)
            return binCandidate;

        return null;
    }
}
