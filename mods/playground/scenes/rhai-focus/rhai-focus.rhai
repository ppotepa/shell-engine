let commands = [];
let phase = (scene_elapsed_ms / 650) % 3;

commands.push(#{ op: "visibility", target: "focus-tag-a", visible: phase == 0 });
commands.push(#{ op: "visibility", target: "focus-tag-b", visible: phase == 1 });
commands.push(#{ op: "visibility", target: "focus-tag-c", visible: phase == 2 });
let blink = (scene_elapsed_ms / 250) % 2 == 0;
commands.push(#{ op: "visibility", target: "focus-cursor-a", visible: phase == 0 && blink });
commands.push(#{ op: "visibility", target: "focus-cursor-b", visible: phase == 1 && blink });
commands.push(#{ op: "visibility", target: "focus-cursor-c", visible: phase == 2 && blink });

commands
