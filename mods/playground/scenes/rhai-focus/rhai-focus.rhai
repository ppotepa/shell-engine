let phase = (time.scene_elapsed_ms / 650) % 3;

scene.object("focus-tag-a").set("visible", phase == 0);
scene.object("focus-tag-b").set("visible", phase == 1);
scene.object("focus-tag-c").set("visible", phase == 2);
let blink = (time.scene_elapsed_ms / 250) % 2 == 0;
scene.object("focus-cursor-a").set("visible", phase == 0 && blink);
scene.object("focus-cursor-b").set("visible", phase == 1 && blink);
scene.object("focus-cursor-c").set("visible", phase == 2 && blink);
