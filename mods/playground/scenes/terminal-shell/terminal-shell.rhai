let commands = [];

let blink_on = ((scene_elapsed_ms / 450) % 2) == 0;
commands.push(#{ op: "visibility", target: "ui-shell-status-dot", visible: blink_on });

commands
