let commands = [];
let phase = (scene_elapsed_ms / 300) % 2;
commands.push(#{ op: "visibility", target: "object-halo-a", visible: phase == 0 });
commands.push(#{ op: "visibility", target: "object-halo-b", visible: phase == 1 });
commands.push(#{ op: "visibility", target: "object-spark", visible: (scene_elapsed_ms / 120) % 2 == 0 });
commands.push(#{ op: "visibility", target: "object-shadow-a", visible: phase == 0 });
commands.push(#{ op: "visibility", target: "object-shadow-b", visible: phase == 1 });

commands
