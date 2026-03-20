namespace CognitosOs.State;

internal enum SessionMode
{
    Booting,
    LoginUser,
    LoginPassword,
    Shell,
}

internal sealed class MachineState
{
    public string? UserName { get; set; }
    public string? Password { get; set; }
    public DateTime? LastLogin { get; set; }
    public ulong UptimeMs { get; set; }
    public string Cwd { get; set; } = "~";
    public SessionMode Mode { get; set; } = SessionMode.Booting;
    public string PendingLoginUser { get; set; } = "";

    public bool HasAccount => !string.IsNullOrWhiteSpace(UserName) && !string.IsNullOrWhiteSpace(Password);
}
