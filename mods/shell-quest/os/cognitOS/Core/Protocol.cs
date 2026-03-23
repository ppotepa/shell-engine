using System.Text.Json;
using System.Text.Json.Serialization;
using CognitOS.Framework.Transport;

namespace CognitOS.Core;

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

    /// <summary>
    /// Send a single line with optional delay in milliseconds.
    /// Engine queues and displays after delay for realistic timing simulation.
    /// </summary>
    public static void EmitLine(IOutputSink sink, string text, ulong? delayMs = null)
    {
        Send(sink, new
        {
            type = "emit-line",
            text,
            delay_ms = delayMs
        });
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
