using System.Text.Json;
using System.Text.Json.Serialization;

namespace CognitosOs.Core;

internal static class Protocol
{
    private static readonly JsonSerializerOptions JsonOpts = new()
    {
        PropertyNamingPolicy = null,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
        WriteIndented = false,
    };

    public static void Send(object payload)
    {
        Console.Out.WriteLine(JsonSerializer.Serialize(payload, JsonOpts));
        Console.Out.Flush();
    }

    public static string? GetTypeTag(JsonElement root)
        => root.TryGetProperty("type", out var t) && t.ValueKind == JsonValueKind.String
            ? t.GetString()
            : null;

    public static string? GetString(JsonElement root, string name)
        => root.TryGetProperty(name, out var p) && p.ValueKind == JsonValueKind.String
            ? p.GetString()
            : null;

    public static int? GetInt(JsonElement root, string name)
        => root.TryGetProperty(name, out var p) && p.TryGetInt32(out var value)
            ? value
            : null;

    public static bool? GetBool(JsonElement root, string name)
        => root.TryGetProperty(name, out var p) && (p.ValueKind is JsonValueKind.True or JsonValueKind.False)
            ? p.GetBoolean()
            : null;
}
