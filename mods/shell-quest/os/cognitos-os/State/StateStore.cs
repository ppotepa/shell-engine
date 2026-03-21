using System.IO.Compression;
using System.Text.Json;

namespace CognitosOs.State;

internal sealed class ZipStateStore : CognitosOs.Core.IMachineStart
{
    private const int CurrentSchemaVersion = 1;

    private readonly string _path;
    private readonly string _legacyJsonPath;

    public ZipStateStore(string path)
    {
        _path = path;
        _legacyJsonPath = Path.Combine(Path.GetDirectoryName(path) ?? Environment.CurrentDirectory, ".cognitos-state.json");
    }

    public MachineState LoadOrCreate()
    {
        if (!File.Exists(_path))
        {
            if (TryLoadLegacyJson(out var migrated))
            {
                Persist(migrated);
                return migrated;
            }
            return new MachineState();
        }

        try
        {
            using var archive = ZipFile.OpenRead(_path);
            var manifest = ReadJson<StateManifest>(archive, "meta.json") ?? new StateManifest();
            MigrateIfNeeded(manifest);
            var profile = ReadJson<UserProfile>(archive, "users/linus/profile.json");
            var clock = ReadJson<ClockState>(archive, "system/clock.json");
            var processes = ReadJson<List<ProcessEntry>>(archive, "system/processes.json") ?? new List<ProcessEntry>();
            var services = ReadJson<List<ServiceEntry>>(archive, "system/services.json") ?? new List<ServiceEntry>();
            var mailMessages = ReadJson<List<MailMessage>>(archive, "users/linus/mail-index.json") ?? new List<MailMessage>();
            var unread = ReadJson<int>(archive, "users/linus/unread-count.json");

            return new MachineState
            {
                UserName = profile?.UserName,
                Password = profile?.Password,
                LastLogin = profile?.LastLogin,
                UptimeMs = clock?.UptimeMs ?? 0,
                Mode = SessionMode.Booting,
                Processes = processes,
                Services = services,
                MailMessages = mailMessages,
                UnreadMailCount = unread == 0 ? mailMessages.Count(m => !m.IsRead) : unread,
            };
        }
        catch
        {
            return new MachineState();
        }
    }

    public void Persist(MachineState state)
    {
        Directory.CreateDirectory(Path.GetDirectoryName(_path) ?? Environment.CurrentDirectory);
        var previousManifest = ReadExistingManifest();
        if (File.Exists(_path))
        {
            File.Delete(_path);
        }

        using var archive = ZipFile.Open(_path, ZipArchiveMode.Create);

        var now = DateTime.UtcNow;
        WriteJson(archive, "meta.json", new StateManifest
        {
            SchemaVersion = CurrentSchemaVersion,
            OperatingSystem = "minix",
            CreatedUtc = previousManifest?.CreatedUtc ?? now,
            UpdatedUtc = now,
        });

        WriteJson(archive, "users/linus/profile.json", new UserProfile
        {
            UserName = state.UserName ?? "linus",
            Password = state.Password ?? "",
            LastLogin = state.LastLogin,
        });

        foreach (var mail in state.MailMessages)
        {
            WriteText(archive, $"users/linus/home/mail/{mail.FileName}", mail.Content);
        }

        WriteText(archive, "users/linus/home/notes/starter.txt",
            "- type ls to look around\n" +
            "- type cat mail/welcome.txt to read your mail\n" +
            "- try top to inspect machine status\n");

        WriteJson(archive, "system/clock.json", new ClockState { UptimeMs = state.UptimeMs });

        WriteJson(archive, "system/processes.json", state.Processes);
        WriteJson(archive, "system/services.json", state.Services);
        WriteJson(archive, "users/linus/mail-index.json", state.MailMessages);
        WriteJson(archive, "users/linus/unread-count.json", state.UnreadMailCount);
    }

    private StateManifest? ReadExistingManifest()
    {
        if (!File.Exists(_path))
        {
            return null;
        }

        try
        {
            using var archive = ZipFile.OpenRead(_path);
            return ReadJson<StateManifest>(archive, "meta.json");
        }
        catch
        {
            return null;
        }
    }

    private bool TryLoadLegacyJson(out MachineState state)
    {
        state = new MachineState();
        if (!File.Exists(_legacyJsonPath))
        {
            return false;
        }

        try
        {
            using var stream = File.OpenRead(_legacyJsonPath);
            using var doc = JsonDocument.Parse(stream);
            var root = doc.RootElement;
            state = new MachineState
            {
                UserName = ReadString(root, "UserName"),
                Password = ReadString(root, "Password"),
                LastLogin = ReadDateTime(root, "LastLogin"),
                UptimeMs = ReadUInt64(root, "UptimeMs"),
                Mode = SessionMode.Booting,
            };

            var backupPath = $"{_legacyJsonPath}.migrated";
            if (File.Exists(backupPath))
            {
                File.Delete(backupPath);
            }
            File.Move(_legacyJsonPath, backupPath);
            return true;
        }
        catch
        {
            return false;
        }
    }

    private static void MigrateIfNeeded(StateManifest manifest)
    {
        if (manifest.SchemaVersion > CurrentSchemaVersion)
        {
            throw new InvalidOperationException($"Unsupported state schema version: {manifest.SchemaVersion}");
        }
    }

    private static T? ReadJson<T>(ZipArchive archive, string entryPath)
    {
        var entry = archive.GetEntry(entryPath);
        if (entry is null)
        {
            return default;
        }

        using var stream = entry.Open();
        return JsonSerializer.Deserialize<T>(stream);
    }

    private static void WriteJson<T>(ZipArchive archive, string entryPath, T payload)
    {
        var entry = archive.CreateEntry(entryPath, CompressionLevel.Optimal);
        using var stream = entry.Open();
        JsonSerializer.Serialize(stream, payload, new JsonSerializerOptions { WriteIndented = true });
    }

    private static void WriteText(ZipArchive archive, string entryPath, string text)
    {
        var entry = archive.CreateEntry(entryPath, CompressionLevel.Optimal);
        using var stream = entry.Open();
        using var writer = new StreamWriter(stream);
        writer.Write(text);
    }

    private static string? ReadString(JsonElement root, string name)
        => root.TryGetProperty(name, out var value) && value.ValueKind == JsonValueKind.String
            ? value.GetString()
            : null;

    private static DateTime? ReadDateTime(JsonElement root, string name)
    {
        if (!root.TryGetProperty(name, out var value) || value.ValueKind != JsonValueKind.String)
        {
            return null;
        }

        var raw = value.GetString();
        return DateTime.TryParse(raw, out var parsed) ? parsed : null;
    }

    private static ulong ReadUInt64(JsonElement root, string name)
        => root.TryGetProperty(name, out var value) && value.TryGetUInt64(out var number)
            ? number
            : 0;
}
