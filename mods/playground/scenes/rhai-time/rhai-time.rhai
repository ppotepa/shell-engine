let phase = (time.scene_elapsed_ms / 350) % 3;
scene.object("time-pulse-a").set("visible", phase == 0);
scene.object("time-pulse-b").set("visible", phase == 1);
scene.object("time-pulse-c").set("visible", phase == 2);
scene.object("time-flash").set("visible", ((time.scene_elapsed_ms / 250) % 2) == 0);
