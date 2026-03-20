using System.Text.Json;

namespace CognitosOs.State;

internal sealed class JsonStateStore : CognitosOs.Core.IMachineStart
{
    private readonly string _path;

    public JsonStateStore(string path)
    {
        _path = path;
    }

    public MachineState LoadOrCreate()
    {
        if (!File.Exists(_path))
        {
            return new MachineState();
        }

        try
        {
            var json = File.ReadAllText(_path);
            return JsonSerializer.Deserialize<MachineState>(json) ?? new MachineState();
        }
        catch
        {
            return new MachineState();
        }
    }

    public void Persist(MachineState state)
    {
        var json = JsonSerializer.Serialize(state, new JsonSerializerOptions { WriteIndented = true });
        File.WriteAllText(_path, json);
    }
}
