let phase = (time.scene_elapsed_ms / 350) % 3;
scene.set("time-pulse-a", "visible", phase == 0);
scene.set("time-pulse-b", "visible", phase == 1);
scene.set("time-pulse-c", "visible", phase == 2);
scene.set("time-flash", "visible", ((time.scene_elapsed_ms / 250) % 2) == 0);
