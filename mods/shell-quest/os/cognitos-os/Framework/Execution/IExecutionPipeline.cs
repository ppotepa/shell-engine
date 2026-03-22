namespace CognitosOs.Framework.Execution;

using CognitosOs.Core;
using CognitosOs.Kernel;

/// <summary>
/// Resolves shell input across builtins, commands, interactive apps and scripts.
/// This is the main OS-specific execution policy boundary.
/// </summary>
internal interface IExecutionPipeline
{
    ApplicationResult Execute(IUnitOfWork uow, string input);
}
