namespace CognitOS.Core;

/// <summary>
/// Game difficulty levels. Each maps to a distinct <see cref="MachineSpec"/>
/// that constrains hardware capabilities and affects gameplay.
/// </summary>
internal enum Difficulty
{
    MouseEnjoyer  = 1,
    ScriptKiddie  = 2,
    ICanExitVim   = 3,
    Dvorak        = 4,
    Su            = 5,
}

/// <summary>
/// Hardware specification produced from a <see cref="Difficulty"/> selection.
/// Every subsystem (FS, Network, FTP, commands) reads this — nothing is hardcoded.
/// </summary>
internal sealed record MachineSpec
{
    public Difficulty Difficulty { get; init; }

    // CPU
    public string CpuModel { get; init; } = "Intel 386 DX-33";
    public int CpuMhz { get; init; } = 33;

    // Memory
    public int RamKb { get; init; } = 4096;

    // Disk
    public int DiskKb { get; init; } = 40960;
    public int DiskFreeKb { get; init; } = 20480;

    // Modem / serial line
    public string ModemModel { get; init; } = "US Robotics Courier 2400";
    public int ModemBaud { get; init; } = 2400;
    public int FtpTimeoutMs { get; init; } = 30000;

    // Gameplay multipliers — subsystems can use these for timing/limits
    public double OperationSpeedMultiplier { get; init; } = 1.0;
    public int MaxProcesses { get; init; } = 16;
    public int MaxOpenFiles { get; init; } = 32;

    /// <summary>
    /// Produce a <see cref="MachineSpec"/> from a difficulty level.
    /// This is the single source of truth for hardware-per-difficulty.
    /// </summary>
    public static MachineSpec FromDifficulty(Difficulty difficulty) => difficulty switch
    {
        Difficulty.MouseEnjoyer => new MachineSpec
        {
            Difficulty = difficulty,
            CpuModel = "Intel 486 DX2-66",
            CpuMhz = 66,
            RamKb = 8192,
            DiskKb = 81920,
            DiskFreeKb = 52000,
            ModemModel = "US Robotics Courier 2400",
            ModemBaud = 2400,
            FtpTimeoutMs = 60000,
            OperationSpeedMultiplier = 0.6,
            MaxProcesses = 32,
            MaxOpenFiles = 64,
        },
        Difficulty.ScriptKiddie => new MachineSpec
        {
            Difficulty = difficulty,
            CpuModel = "Intel 486 DX-33",
            CpuMhz = 33,
            RamKb = 4096,
            DiskKb = 40960,
            DiskFreeKb = 28000,
            ModemModel = "Hayes Smartmodem 1200",
            ModemBaud = 1200,
            FtpTimeoutMs = 45000,
            OperationSpeedMultiplier = 0.8,
            MaxProcesses = 24,
            MaxOpenFiles = 48,
        },
        Difficulty.ICanExitVim => new MachineSpec
        {
            Difficulty = difficulty,
            CpuModel = "Intel 386 DX-33",
            CpuMhz = 33,
            RamKb = 4096,
            DiskKb = 40960,
            DiskFreeKb = 20480,
            ModemModel = "Hayes Smartmodem 1200",
            ModemBaud = 1200,
            FtpTimeoutMs = 30000,
            OperationSpeedMultiplier = 1.0,
            MaxProcesses = 16,
            MaxOpenFiles = 32,
        },
        Difficulty.Dvorak => new MachineSpec
        {
            Difficulty = difficulty,
            CpuModel = "Intel 386 SX-16",
            CpuMhz = 16,
            RamKb = 2048,
            DiskKb = 20480,
            DiskFreeKb = 8000,
            ModemModel = "Hayes Smartmodem 300",
            ModemBaud = 300,
            FtpTimeoutMs = 20000,
            OperationSpeedMultiplier = 1.4,
            MaxProcesses = 12,
            MaxOpenFiles = 20,
        },
        Difficulty.Su => new MachineSpec
        {
            Difficulty = difficulty,
            CpuModel = "Intel 386 SX-16",
            CpuMhz = 16,
            RamKb = 1024,
            DiskKb = 10240,
            DiskFreeKb = 3200,
            ModemModel = "Generic 300 baud serial",
            ModemBaud = 300,
            FtpTimeoutMs = 12000,
            OperationSpeedMultiplier = 2.0,
            MaxProcesses = 8,
            MaxOpenFiles = 12,
        },
        _ => FromDifficulty(Difficulty.ICanExitVim),
    };

    /// <summary>
    /// Parse difficulty from the string label used in menu YAML.
    /// Falls back to <see cref="Difficulty.ICanExitVim"/> on unknown input.
    /// </summary>
    public static Difficulty ParseLabel(string? label) => label?.Trim().ToUpperInvariant() switch
    {
        "MOUSE ENJOYER" => Difficulty.MouseEnjoyer,
        "SCRIPT KIDDIE" => Difficulty.ScriptKiddie,
        "I CAN EXIT VIM" => Difficulty.ICanExitVim,
        "DVORAK" => Difficulty.Dvorak,
        "SU" => Difficulty.Su,
        _ => Difficulty.ICanExitVim,
    };
}
