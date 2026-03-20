let phase = (time.scene_elapsed_ms / 650) % 3;

scene.set("focus-tag-a", "visible", phase == 0);
scene.set("focus-tag-b", "visible", phase == 1);
scene.set("focus-tag-c", "visible", phase == 2);
let blink = (time.scene_elapsed_ms / 250) % 2 == 0;
scene.set("focus-cursor-a", "visible", phase == 0 && blink);
scene.set("focus-cursor-b", "visible", phase == 1 && blink);
scene.set("focus-cursor-c", "visible", phase == 2 && blink);
