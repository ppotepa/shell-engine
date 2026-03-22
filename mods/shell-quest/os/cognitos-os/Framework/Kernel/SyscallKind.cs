namespace CognitosOs.Framework.Kernel;

internal enum SyscallKind
{
    // Disk
    DiskRead,
    DiskWrite,
    DiskAppend,
    DiskStat,
    DiskUnlink,
    DiskMkdir,
    DiskListDir,

    // Network
    NetConnect,
    NetSend,
    NetRecv,
    NetClose,
    NetResolve,

    // Process
    ProcessFork,
    ProcessExec,
    ProcessExit,
    ProcessKill,
    ProcessList,

    // Memory
    MemAlloc,
    MemFree,

    // Clock
    ClockRead,

    // Mail
    MailRead,
    MailDeliver,

    // Journal
    JournalAppend,
}
