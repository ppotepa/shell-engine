using System.Text.Json;
using System.Text.Json.Serialization;
using CognitosOs.Framework.Transport;

namespace CognitosOs.Core;

internal static class Protocol
{
    private static readonly JsonSerializerOptions JsonOpts = new()
    {
        PropertyNamingPolicy = null,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
        WriteIndented = false,
    };

    public static void Send(IOutputSink sink, object payload)
    {
        sink.WriteProtocolLine(JsonSerializer.Serialize(payload, JsonOpts));
        sink.Flush();
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
