let commands = [];
let phase = (scene_elapsed_ms / 350) % 3;
commands.push(#{ op: "visibility", target: "time-pulse-a", visible: phase == 0 });
commands.push(#{ op: "visibility", target: "time-pulse-b", visible: phase == 1 });
commands.push(#{ op: "visibility", target: "time-pulse-c", visible: phase == 2 });
commands.push(#{ op: "visibility", target: "time-flash", visible: ((scene_elapsed_ms / 250) % 2) == 0 });

commands
